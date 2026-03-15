use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use matrix_sdk::{
    Client, SessionMeta, SessionTokens,
    authentication::matrix::MatrixSession,
    config::{RequestConfig, SyncSettings},
    room::reply::{EnforceThread, Reply},
};
use ruma::{
    OwnedDeviceId, OwnedEventId, OwnedRoomId, OwnedUserId,
    events::room::message::{ReplyWithinThread, RoomMessageEventContent},
};
use robrix_botfather::{SenderProfile, SenderProfileKind};

use crate::app_data_dir;

static BOT_SENDER_CLIENTS: LazyLock<Mutex<HashMap<String, Client>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug)]
pub struct VerifiedSenderSession {
    pub matrix_user_id: String,
    pub device_id: String,
    pub access_token: String,
}

pub async fn send_markdown_via_sender(
    profile: &SenderProfile,
    room_id: &str,
    thread_root_event_id: Option<&str>,
    markdown: String,
) -> Result<(), String> {
    if profile.kind == SenderProfileKind::CurrentUser {
        return Err("The current-user sender must use Robrix's primary Matrix client.".into());
    }

    let client = get_or_create_sender_client(profile).await?;
    client
        .sync_once(SyncSettings::default().timeout(Duration::from_secs(15)))
        .await
        .map_err(|error| format!("Failed to refresh sender session: {error}"))?;

    let room_id: OwnedRoomId = room_id
        .parse()
        .map_err(|error| format!("Invalid Matrix room id `{room_id}`: {error}"))?;
    let room = client
        .get_room(&room_id)
        .ok_or_else(|| {
            format!(
                "Sender \"{}\" is not aware of room `{}` yet. Make sure this bot account has joined the room.",
                profile.name, room_id
            )
        })?;
    let content = RoomMessageEventContent::text_markdown(markdown);

    if let Some(thread_root_event_id) = thread_root_event_id {
        let event_id: OwnedEventId = thread_root_event_id.parse().map_err(|error| {
            format!("Invalid thread root event id `{thread_root_event_id}`: {error}")
        })?;
        let reply = Reply {
            event_id,
            enforce_thread: EnforceThread::Threaded(ReplyWithinThread::No),
        };
        let reply_content = room
            .make_reply_event(content.into(), reply)
            .await
            .map_err(|error| format!("Failed to build threaded bot message: {error}"))?;
        room.send(reply_content)
            .await
            .map_err(|error| format!("Failed to send threaded bot message: {error}"))?;
    } else {
        room.send(content)
            .await
            .map_err(|error| format!("Failed to send bot message: {error}"))?;
    }

    Ok(())
}

pub async fn verify_sender_login(
    sender_profile_id: &str,
    homeserver_url: &str,
    matrix_user_id: &str,
    password: &str,
) -> Result<VerifiedSenderSession, String> {
    let homeserver_url = homeserver_url.trim();
    let matrix_user_id = matrix_user_id.trim();
    let password = password.trim();
    if homeserver_url.is_empty() {
        return Err("Sender homeserver URL cannot be empty.".into());
    }
    if matrix_user_id.is_empty() {
        return Err("Sender Matrix user ID cannot be empty.".into());
    }
    if password.is_empty() {
        return Err("Sender password cannot be empty for verification.".into());
    }

    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .sqlite_store(
            sender_store_dir(sender_profile_id),
            Some(&format!("robrix-bot-sender:{sender_profile_id}")),
        )
        .request_config(RequestConfig::new().timeout(Duration::from_secs(30)))
        .build()
        .await
        .map_err(|error| format!("Failed to create sender verification client: {error}"))?;
    client
        .matrix_auth()
        .login_username(matrix_user_id, password)
        .initial_device_display_name(&format!("robrix-bot-sender-{sender_profile_id}"))
        .send()
        .await
        .map_err(|error| format!("Failed to verify sender login: {error}"))?;

    let session = client
        .matrix_auth()
        .session()
        .ok_or_else(|| "Matrix login succeeded but no session was returned.".to_string())?;
    BOT_SENDER_CLIENTS
        .lock()
        .unwrap()
        .insert(sender_profile_id.to_string(), client);
    Ok(VerifiedSenderSession {
        matrix_user_id: session.meta.user_id.to_string(),
        device_id: session.meta.device_id.to_string(),
        access_token: session.tokens.access_token,
    })
}

async fn get_or_create_sender_client(profile: &SenderProfile) -> Result<Client, String> {
    if let Some(client) = BOT_SENDER_CLIENTS.lock().unwrap().get(&profile.id).cloned() {
        return Ok(client);
    }

    let homeserver_url = profile
        .homeserver_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Sender \"{}\" is missing a homeserver URL.", profile.name))?;
    let matrix_user_id: OwnedUserId = profile
        .matrix_user_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Sender \"{}\" is missing a Matrix user ID.", profile.name))?
        .parse()
        .map_err(|error| {
            format!(
                "Sender \"{}\" has an invalid Matrix user ID: {error}",
                profile.name
            )
        })?;
    let device_id: OwnedDeviceId = profile
        .device_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Sender \"{}\" is missing a device ID.", profile.name))?
        .into();
    let access_token = sender_access_token(profile)?;

    let db_dir = sender_store_dir(&profile.id);
    let passphrase = format!("robrix-bot-sender:{}", profile.id);
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .sqlite_store(&db_dir, Some(&passphrase))
        .request_config(RequestConfig::new().timeout(Duration::from_secs(30)))
        .build()
        .await
        .map_err(|error| {
            format!(
                "Failed to create Matrix client for sender \"{}\": {error}",
                profile.name
            )
        })?;
    let session = MatrixSession {
        meta: SessionMeta {
            user_id: matrix_user_id,
            device_id,
        },
        tokens: SessionTokens {
            access_token,
            refresh_token: None,
        },
    };
    client.restore_session(session).await.map_err(|error| {
        format!(
            "Failed to restore session for sender \"{}\": {error}",
            profile.name
        )
    })?;

    BOT_SENDER_CLIENTS
        .lock()
        .unwrap()
        .insert(profile.id.clone(), client.clone());
    Ok(client)
}

fn sender_access_token(profile: &SenderProfile) -> Result<String, String> {
    if let Some(token_env) = profile
        .access_token_env
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if let Ok(access_token) = std::env::var(token_env) {
            if !access_token.trim().is_empty() {
                return Ok(access_token);
            }
        }
    }
    profile
        .access_token
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            format!(
                "Sender \"{}\" has no usable access token. Verify the account in BotFather Settings or provide a token env var.",
                profile.name
            )
        })
}

fn sender_store_dir(sender_profile_id: &str) -> std::path::PathBuf {
    app_data_dir()
        .join("bot_senders")
        .join(sanitize_path_component(sender_profile_id))
}

fn sanitize_path_component(value: &str) -> String {
    value
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() || matches!(char, '-' | '_') {
                char
            } else {
                '_'
            }
        })
        .collect()
}
