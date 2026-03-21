//! LiveKit SFU client for scalable multi-party calls.
//!
//! This module provides integration with LiveKit Selective Forwarding Unit (SFU)
//! for handling multi-party video/audio calls efficiently.

use std::collections::HashMap;
use std::sync::Arc;

use makepad_widgets::{log, SignalToUI};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use tokio::sync::{mpsc, RwLock};


/// JWT token for LiveKit authentication.
#[derive(Clone, Debug)]
pub struct LiveKitToken {
    /// The JWT token string.
    pub token: String,
    /// The LiveKit server URL.
    pub server_url: String,
    /// The room name in LiveKit.
    pub room_name: String,
}

/// Events from the LiveKit client.
#[derive(Clone, Debug)]
pub enum LiveKitEvent {
    /// Connected to the LiveKit room.
    Connected {
        room_name: String,
    },
    /// Disconnected from the LiveKit room.
    Disconnected {
        room_name: String,
        reason: String,
    },
    /// A participant joined the room.
    ParticipantJoined {
        participant_id: String,
        user_id: Option<OwnedUserId>,
    },
    /// A participant left the room.
    ParticipantLeft {
        participant_id: String,
    },
    /// A participant's track was subscribed.
    TrackSubscribed {
        participant_id: String,
        track_sid: String,
        track_kind: TrackKind,
    },
    /// A participant's track was unsubscribed.
    TrackUnsubscribed {
        participant_id: String,
        track_sid: String,
    },
    /// A participant muted/unmuted their track.
    TrackMuted {
        participant_id: String,
        track_sid: String,
        muted: bool,
    },
    /// Connection quality changed for a participant.
    ConnectionQualityChanged {
        participant_id: String,
        quality: ConnectionQuality,
    },
    /// Error occurred.
    Error {
        message: String,
    },
}

/// Type of media track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Audio,
    Video,
    ScreenShare,
    ScreenShareAudio,
}

/// Connection quality levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Poor,
    Lost,
    Unknown,
}

/// Configuration for the LiveKit client.
#[derive(Clone, Debug)]
pub struct LiveKitConfig {
    /// Whether to enable automatic reconnection.
    pub auto_reconnect: bool,
    /// Reconnection timeout in milliseconds.
    pub reconnect_timeout_ms: u64,
    /// Whether to enable adaptive streaming.
    pub adaptive_stream: bool,
    /// Whether to enable dynacast (dynamic track publishing).
    pub dynacast: bool,
}

impl Default for LiveKitConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            reconnect_timeout_ms: 30_000,
            adaptive_stream: true,
            dynacast: true,
        }
    }
}

/// LiveKit room state.
#[derive(Clone, Debug, Default)]
pub struct LiveKitRoomState {
    /// Whether we are connected to the room.
    pub is_connected: bool,
    /// The room name.
    pub room_name: Option<String>,
    /// Local participant identity.
    pub local_participant_id: Option<String>,
    /// Remote participants in the room.
    pub participants: HashMap<String, LiveKitParticipant>,
    /// Our local tracks that are published.
    pub local_tracks: Vec<LiveKitTrack>,
}

/// A participant in the LiveKit room.
#[derive(Clone, Debug)]
pub struct LiveKitParticipant {
    /// Unique participant ID (SID).
    pub participant_id: String,
    /// Participant identity (often the user ID).
    pub identity: String,
    /// Matrix user ID if available.
    pub user_id: Option<OwnedUserId>,
    /// Tracks from this participant.
    pub tracks: Vec<LiveKitTrack>,
    /// Connection quality.
    pub connection_quality: ConnectionQuality,
    /// Whether the participant is speaking.
    pub is_speaking: bool,
}

/// A media track in LiveKit.
#[derive(Clone, Debug)]
pub struct LiveKitTrack {
    /// Track SID.
    pub track_sid: String,
    /// Track kind.
    pub kind: TrackKind,
    /// Whether the track is muted.
    pub muted: bool,
    /// Whether we are subscribed to this track.
    pub subscribed: bool,
}

/// Client for connecting to LiveKit SFU.
#[allow(dead_code)]
pub struct LiveKitClient {
    /// Configuration.
    config: LiveKitConfig,
    /// Current room state.
    state: Arc<RwLock<LiveKitRoomState>>,
    /// Event sender.
    event_sender: mpsc::UnboundedSender<LiveKitEvent>,
    /// Event receiver.
    event_receiver: mpsc::UnboundedReceiver<LiveKitEvent>,
    /// Associated Matrix room ID.
    room_id: Option<OwnedRoomId>,
}

impl LiveKitClient {
    /// Create a new LiveKit client.
    pub fn new(config: LiveKitConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            state: Arc::new(RwLock::new(LiveKitRoomState::default())),
            event_sender,
            event_receiver,
            room_id: None,
        }
    }

    /// Connect to a LiveKit room using the given token.
    pub async fn connect(&mut self, token: LiveKitToken, room_id: OwnedRoomId) -> Result<(), String> {
        self.room_id = Some(room_id);

        // In a real implementation, this would use the livekit-api crate
        // to connect to the LiveKit server. For now, we'll simulate the connection.
        log!("Connecting to LiveKit room: {} at {}", token.room_name, token.server_url);

        // Update state
        {
            let mut state = self.state.write().await;
            state.is_connected = true;
            state.room_name = Some(token.room_name.clone());
            state.local_participant_id = Some("local".to_string());
        }

        // Emit connected event
        let _ = self.event_sender.send(LiveKitEvent::Connected {
            room_name: token.room_name,
        });

        SignalToUI::set_ui_signal();
        Ok(())
    }

    /// Disconnect from the current LiveKit room.
    pub async fn disconnect(&mut self) -> Result<(), String> {
        let room_name = {
            let state = self.state.read().await;
            state.room_name.clone()
        };

        if let Some(room_name) = room_name {
            log!("Disconnecting from LiveKit room: {}", room_name);

            // Clear state
            {
                let mut state = self.state.write().await;
                state.is_connected = false;
                state.room_name = None;
                state.local_participant_id = None;
                state.participants.clear();
                state.local_tracks.clear();
            }

            // Emit disconnected event
            let _ = self.event_sender.send(LiveKitEvent::Disconnected {
                room_name,
                reason: "User disconnected".to_string(),
            });

            SignalToUI::set_ui_signal();
        }

        self.room_id = None;
        Ok(())
    }

    /// Publish a local video track.
    pub async fn publish_video(&mut self) -> Result<String, String> {
        let track_sid = format!("video_{}", uuid_v4());

        {
            let mut state = self.state.write().await;
            state.local_tracks.push(LiveKitTrack {
                track_sid: track_sid.clone(),
                kind: TrackKind::Video,
                muted: false,
                subscribed: true,
            });
        }

        log!("Published video track: {}", track_sid);
        Ok(track_sid)
    }

    /// Publish a local audio track.
    pub async fn publish_audio(&mut self) -> Result<String, String> {
        let track_sid = format!("audio_{}", uuid_v4());

        {
            let mut state = self.state.write().await;
            state.local_tracks.push(LiveKitTrack {
                track_sid: track_sid.clone(),
                kind: TrackKind::Audio,
                muted: false,
                subscribed: true,
            });
        }

        log!("Published audio track: {}", track_sid);
        Ok(track_sid)
    }

    /// Unpublish a local track.
    pub async fn unpublish_track(&mut self, track_sid: &str) -> Result<(), String> {
        {
            let mut state = self.state.write().await;
            state.local_tracks.retain(|t| t.track_sid != track_sid);
        }

        log!("Unpublished track: {}", track_sid);
        Ok(())
    }

    /// Mute or unmute a local track.
    pub async fn set_track_muted(&mut self, track_sid: &str, muted: bool) -> Result<(), String> {
        {
            let mut state = self.state.write().await;
            if let Some(track) = state.local_tracks.iter_mut().find(|t| t.track_sid == track_sid) {
                track.muted = muted;
            }
        }

        log!("Set track {} muted: {}", track_sid, muted);
        Ok(())
    }

    /// Subscribe to a remote participant's track.
    pub async fn subscribe_track(&mut self, participant_id: &str, track_sid: &str) -> Result<(), String> {
        {
            let mut state = self.state.write().await;
            if let Some(participant) = state.participants.get_mut(participant_id) {
                if let Some(track) = participant.tracks.iter_mut().find(|t| t.track_sid == track_sid) {
                    track.subscribed = true;
                }
            }
        }

        // Emit event
        let _ = self.event_sender.send(LiveKitEvent::TrackSubscribed {
            participant_id: participant_id.to_string(),
            track_sid: track_sid.to_string(),
            track_kind: TrackKind::Video, // TODO: Get actual kind
        });

        log!("Subscribed to track {} from {}", track_sid, participant_id);
        Ok(())
    }

    /// Unsubscribe from a remote participant's track.
    pub async fn unsubscribe_track(&mut self, participant_id: &str, track_sid: &str) -> Result<(), String> {
        {
            let mut state = self.state.write().await;
            if let Some(participant) = state.participants.get_mut(participant_id) {
                if let Some(track) = participant.tracks.iter_mut().find(|t| t.track_sid == track_sid) {
                    track.subscribed = false;
                }
            }
        }

        // Emit event
        let _ = self.event_sender.send(LiveKitEvent::TrackUnsubscribed {
            participant_id: participant_id.to_string(),
            track_sid: track_sid.to_string(),
        });

        log!("Unsubscribed from track {} from {}", track_sid, participant_id);
        Ok(())
    }

    /// Get the current room state.
    pub async fn state(&self) -> LiveKitRoomState {
        self.state.read().await.clone()
    }

    /// Check if connected to a room.
    pub async fn is_connected(&self) -> bool {
        self.state.read().await.is_connected
    }

    /// Get the next event from the client.
    pub fn try_recv_event(&mut self) -> Option<LiveKitEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// Wait for the next event from the client.
    pub async fn recv_event(&mut self) -> Option<LiveKitEvent> {
        self.event_receiver.recv().await
    }

    /// Get the associated Matrix room ID.
    pub fn room_id(&self) -> Option<&OwnedRoomId> {
        self.room_id.as_ref()
    }

    /// Get the list of participants.
    pub async fn participants(&self) -> Vec<LiveKitParticipant> {
        let state = self.state.read().await;
        state.participants.values().cloned().collect()
    }
}

impl Default for LiveKitClient {
    fn default() -> Self {
        Self::new(LiveKitConfig::default())
    }
}

/// Generate a simple UUID v4-like string.
fn uuid_v4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.r#gen();

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Helper to create a LiveKit JWT token.
///
/// In production, this would be obtained from the Matrix homeserver
/// or a dedicated authentication service.
pub fn create_livekit_token(
    _api_key: &str,
    _api_secret: &str,
    room_name: &str,
    participant_identity: &str,
    _ttl_seconds: u64,
) -> Result<String, String> {
    // In a real implementation, this would use the livekit-api crate's
    // AccessToken to create a properly signed JWT.
    //
    // For now, return a placeholder that would need to be replaced
    // with actual token generation.
    log!(
        "Creating LiveKit token for room '{}' as '{}'",
        room_name, participant_identity
    );

    // This is a placeholder - real implementation would use:
    // livekit_api::access_token::AccessToken::new(api_key, api_secret)
    //     .with_identity(participant_identity)
    //     .with_name(participant_identity)
    //     .with_grants(VideoGrants { room_join: true, room: room_name.to_string(), ... })
    //     .to_jwt()

    Err("LiveKit token generation requires actual API keys".to_string())
}
