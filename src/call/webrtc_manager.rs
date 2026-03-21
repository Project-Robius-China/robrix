//! Central WebRTC coordinator and session manager.
//!
//! This module manages multiple WebRTC sessions for a call, coordinating
//! between participants and handling the overall call lifecycle.

use std::collections::HashMap;
use std::sync::Arc;

use makepad_widgets::{log, error, SignalToUI};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedRoomId, OwnedUserId};
use tokio::sync::{mpsc, Mutex, RwLock};
use webrtc::api::{
    API, APIBuilder,
    interceptor_registry::register_default_interceptors,
    media_engine::MediaEngine,
};
use webrtc::interceptor::registry::Registry;

use crate::call::{
    call_state::{CallState, set_call_state},
    matrixrtc::{MatrixRTCMembership, FocusActive, generate_call_id},
    webrtc_session::{WebRTCSession, WebRTCSessionConfig, WebRTCSessionEvent},
};

/// Global WebRTC manager instance.
static WEBRTC_MANAGER: std::sync::OnceLock<Arc<WebRTCManager>> = std::sync::OnceLock::new();

/// Get or initialize the global WebRTC manager.
pub fn webrtc_manager() -> Arc<WebRTCManager> {
    WEBRTC_MANAGER
        .get_or_init(|| Arc::new(WebRTCManager::new()))
        .clone()
}

/// Events from the WebRTC manager to be handled by the UI.
#[derive(Debug)]
pub enum WebRTCManagerEvent {
    /// Call state changed.
    CallStateChanged {
        room_id: OwnedRoomId,
        state: CallState,
    },
    /// Need to send a MatrixRTC membership event.
    SendMembershipEvent {
        room_id: OwnedRoomId,
        membership: MatrixRTCMembership,
    },
    /// Error occurred.
    Error {
        room_id: Option<OwnedRoomId>,
        message: String,
    },
}

/// Central coordinator for all WebRTC call sessions.
#[allow(dead_code)]
pub struct WebRTCManager {
    /// The WebRTC API instance.
    api: RwLock<Option<API>>,
    /// Active call sessions, keyed by room ID.
    calls: RwLock<HashMap<OwnedRoomId, CallContext>>,
    /// Event sender for communicating with the UI thread.
    event_sender: Mutex<Option<mpsc::UnboundedSender<WebRTCManagerEvent>>>,
    /// Event receiver for the UI thread.
    event_receiver: Mutex<Option<mpsc::UnboundedReceiver<WebRTCManagerEvent>>>,
}

/// Context for a single call.
struct CallContext {
    /// The room this call is in.
    room_id: OwnedRoomId,
    /// Our call ID.
    call_id: String,
    /// Our device ID.
    local_device_id: OwnedDeviceId,
    /// Our user ID.
    local_user_id: OwnedUserId,
    /// Active peer sessions, keyed by (user_id, device_id).
    sessions: HashMap<(OwnedUserId, OwnedDeviceId), WebRTCSession>,
    /// Whether this is a video call.
    is_video_call: bool,
    /// Call start time.
    start_time_ms: u64,
    /// Current call state.
    state: CallState,
    /// Session configuration.
    session_config: WebRTCSessionConfig,
}

impl WebRTCManager {
    /// Create a new WebRTC manager.
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            api: RwLock::new(None),
            calls: RwLock::new(HashMap::new()),
            event_sender: Mutex::new(Some(event_sender)),
            event_receiver: Mutex::new(Some(event_receiver)),
        }
    }

    /// Initialize the WebRTC API. Must be called before starting calls.
    pub async fn initialize(&self) -> Result<(), webrtc::Error> {
        let mut api_guard = self.api.write().await;
        if api_guard.is_some() {
            return Ok(());
        }

        // Create a MediaEngine with default codecs
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        // Create an interceptor registry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)?;

        // Build the API
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        *api_guard = Some(api);
        log!("WebRTC API initialized");
        Ok(())
    }

    /// Start a new call in the given room.
    pub async fn start_call(
        &self,
        room_id: OwnedRoomId,
        local_user_id: OwnedUserId,
        local_device_id: OwnedDeviceId,
        is_video_call: bool,
        session_config: WebRTCSessionConfig,
    ) -> Result<MatrixRTCMembership, String> {
        // Ensure API is initialized
        self.initialize().await.map_err(|e| e.to_string())?;

        let call_id = generate_call_id();
        let start_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let context = CallContext {
            room_id: room_id.clone(),
            call_id: call_id.clone(),
            local_device_id: local_device_id.clone(),
            local_user_id: local_user_id.clone(),
            sessions: HashMap::new(),
            is_video_call,
            start_time_ms,
            state: CallState::Connecting { room_id: room_id.clone() },
            session_config,
        };

        // Create the membership event
        let membership = MatrixRTCMembership::new(local_device_id.clone(), call_id)
            .with_created_ts(start_time_ms)
            .with_focus_active(FocusActive::livekit());

        // Store the call context
        {
            let mut calls = self.calls.write().await;
            calls.insert(room_id.clone(), context);
        }

        // Update global call state
        set_call_state(
            room_id.clone(),
            CallState::Connecting { room_id: room_id.clone() },
        );

        // Signal UI update
        SignalToUI::set_ui_signal();

        log!("Started call in room {}", room_id);
        Ok(membership)
    }

    /// Join an existing call in the given room.
    pub async fn join_call(
        &self,
        room_id: OwnedRoomId,
        local_user_id: OwnedUserId,
        local_device_id: OwnedDeviceId,
        existing_memberships: Vec<MatrixRTCMembership>,
        session_config: WebRTCSessionConfig,
    ) -> Result<MatrixRTCMembership, String> {
        // Ensure API is initialized
        self.initialize().await.map_err(|e| e.to_string())?;

        let call_id = existing_memberships
            .first()
            .map(|m| m.call_id.clone())
            .unwrap_or_else(generate_call_id);

        let start_time_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let context = CallContext {
            room_id: room_id.clone(),
            call_id: call_id.clone(),
            local_device_id: local_device_id.clone(),
            local_user_id: local_user_id.clone(),
            sessions: HashMap::new(),
            is_video_call: true, // Assume video for now
            start_time_ms,
            state: CallState::Connecting { room_id: room_id.clone() },
            session_config,
        };

        // Create our membership event
        let membership = MatrixRTCMembership::new(local_device_id.clone(), call_id)
            .with_created_ts(start_time_ms)
            .with_focus_active(FocusActive::livekit());

        // Store the call context
        {
            let mut calls = self.calls.write().await;
            calls.insert(room_id.clone(), context);
        }

        // Update global call state
        set_call_state(
            room_id.clone(),
            CallState::Connecting { room_id: room_id.clone() },
        );

        // Signal UI update
        SignalToUI::set_ui_signal();

        log!("Joined call in room {}", room_id);
        Ok(membership)
    }

    /// Leave the call in the given room.
    pub async fn leave_call(&self, room_id: &OwnedRoomId) -> Result<(), String> {
        let context = {
            let mut calls = self.calls.write().await;
            calls.remove(room_id)
        };

        if let Some(mut context) = context {
            // Close all peer sessions
            for (_, session) in context.sessions.drain() {
                if let Err(e) = session.close().await {
                    error!("Error closing WebRTC session: {}", e);
                }
            }

            // Update state
            set_call_state(
                room_id.clone(),
                CallState::Idle,
            );

            // Signal UI update
            SignalToUI::set_ui_signal();

            log!("Left call in room {}", room_id);
        }

        Ok(())
    }

    /// Handle a new participant joining the call.
    pub async fn handle_participant_joined(
        &self,
        room_id: &OwnedRoomId,
        user_id: OwnedUserId,
        device_id: OwnedDeviceId,
        _membership: &MatrixRTCMembership,
    ) -> Result<(), String> {
        let api_guard = self.api.read().await;
        let api = api_guard.as_ref().ok_or("WebRTC API not initialized")?;

        let mut calls = self.calls.write().await;
        let context = calls.get_mut(room_id).ok_or("Call not found")?;

        // Don't create a session with ourselves
        if user_id == context.local_user_id && device_id == context.local_device_id {
            return Ok(());
        }

        // Check if we already have a session with this participant
        let key = (user_id.clone(), device_id.clone());
        if context.sessions.contains_key(&key) {
            return Ok(());
        }

        // Create a new WebRTC session
        let session = WebRTCSession::new(
            api,
            user_id.clone(),
            device_id.clone(),
            context.session_config.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;

        context.sessions.insert(key, session);

        log!("Created WebRTC session for participant {}:{}", user_id, device_id);
        Ok(())
    }

    /// Handle a participant leaving the call.
    pub async fn handle_participant_left(
        &self,
        room_id: &OwnedRoomId,
        user_id: &OwnedUserId,
        device_id: &OwnedDeviceId,
    ) -> Result<(), String> {
        let mut calls = self.calls.write().await;
        let context = calls.get_mut(room_id).ok_or("Call not found")?;

        let key = (user_id.clone(), device_id.clone());
        if let Some(session) = context.sessions.remove(&key) {
            if let Err(e) = session.close().await {
                error!("Error closing WebRTC session: {}", e);
            }
            log!("Removed WebRTC session for participant {}:{}", user_id, device_id);
        }

        Ok(())
    }

    /// Toggle local audio mute.
    pub async fn toggle_audio(&self, room_id: &OwnedRoomId) -> Result<bool, String> {
        // TODO: Implement audio track muting
        log!("Toggle audio for room {}", room_id);
        Ok(true)
    }

    /// Toggle local video.
    pub async fn toggle_video(&self, room_id: &OwnedRoomId) -> Result<bool, String> {
        // TODO: Implement video track toggling
        log!("Toggle video for room {}", room_id);
        Ok(true)
    }

    /// Get the current call state for a room.
    pub async fn get_call_state(&self, room_id: &OwnedRoomId) -> Option<CallState> {
        let calls = self.calls.read().await;
        calls.get(room_id).map(|c| c.state.clone())
    }

    /// Check if there's an active call in the given room.
    pub async fn has_active_call(&self, room_id: &OwnedRoomId) -> bool {
        let calls = self.calls.read().await;
        calls.get(room_id).map(|c| c.state.is_active()).unwrap_or(false)
    }

    /// Get the number of participants in a call.
    pub async fn participant_count(&self, room_id: &OwnedRoomId) -> usize {
        let calls = self.calls.read().await;
        calls.get(room_id)
            .map(|c| c.sessions.len() + 1) // +1 for ourselves
            .unwrap_or(0)
    }

    /// Poll for events from all active sessions.
    pub async fn poll_events(&self) -> Vec<(OwnedRoomId, WebRTCSessionEvent)> {
        let mut events = Vec::new();
        let mut calls = self.calls.write().await;

        for (room_id, context) in calls.iter_mut() {
            for ((_user_id, _device_id), session) in context.sessions.iter_mut() {
                while let Some(event) = session.try_recv_event() {
                    events.push((room_id.clone(), event));
                }
            }
        }

        events
    }
}

impl Default for WebRTCManager {
    fn default() -> Self {
        Self::new()
    }
}
