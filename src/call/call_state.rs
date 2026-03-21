//! Call state machine and participant management.
//!
//! This module defines the core state types for managing call sessions,
//! including call state transitions and participant tracking.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use matrix_sdk::ruma::{OwnedDeviceId, OwnedRoomId, OwnedUserId};

/// Global call states, keyed by room ID.
static CALL_STATES: OnceLock<Mutex<HashMap<OwnedRoomId, CallState>>> = OnceLock::new();

/// Get or initialize the global call states map.
fn call_states() -> &'static Mutex<HashMap<OwnedRoomId, CallState>> {
    CALL_STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Get the call state for a specific room.
pub fn get_call_state(room_id: &OwnedRoomId) -> Option<CallState> {
    call_states().lock().ok()?.get(room_id).cloned()
}

/// Set the call state for a specific room.
pub fn set_call_state(room_id: OwnedRoomId, state: CallState) {
    if let Ok(mut states) = call_states().lock() {
        states.insert(room_id, state);
    }
}

/// Remove the call state for a specific room.
pub fn remove_call_state(room_id: &OwnedRoomId) {
    if let Ok(mut states) = call_states().lock() {
        states.remove(room_id);
    }
}

/// Represents the current state of a call.
#[derive(Clone, Debug, Default)]
pub enum CallState {
    /// No active call in the room.
    #[default]
    Idle,
    /// An incoming call is ringing.
    Ringing {
        room_id: OwnedRoomId,
        caller: CallParticipant,
    },
    /// Connecting to the call (negotiating WebRTC).
    Connecting {
        room_id: OwnedRoomId,
    },
    /// Actively connected to the call.
    Connected {
        room_id: OwnedRoomId,
        participants: Vec<CallParticipant>,
        /// Our local participant info.
        local_participant: CallParticipant,
        /// Whether this is a video call (vs audio-only).
        is_video_call: bool,
        /// Call start time in milliseconds since Unix epoch.
        start_time_ms: u64,
    },
    /// Disconnecting from the call.
    Disconnecting {
        room_id: OwnedRoomId,
    },
    /// Call ended or failed.
    Ended {
        room_id: OwnedRoomId,
        reason: CallEndReason,
    },
}

impl CallState {
    /// Returns true if there is an active or connecting call.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            CallState::Ringing { .. }
                | CallState::Connecting { .. }
                | CallState::Connected { .. }
                | CallState::Disconnecting { .. }
        )
    }

    /// Returns the room ID if the call is in any active state.
    pub fn room_id(&self) -> Option<&OwnedRoomId> {
        match self {
            CallState::Idle => None,
            CallState::Ringing { room_id, .. } => Some(room_id),
            CallState::Connecting { room_id } => Some(room_id),
            CallState::Connected { room_id, .. } => Some(room_id),
            CallState::Disconnecting { room_id } => Some(room_id),
            CallState::Ended { room_id, .. } => Some(room_id),
        }
    }

    /// Returns true if connected to a call.
    pub fn is_connected(&self) -> bool {
        matches!(self, CallState::Connected { .. })
    }

    /// Returns the participants if connected.
    pub fn participants(&self) -> Option<&[CallParticipant]> {
        match self {
            CallState::Connected { participants, .. } => Some(participants),
            _ => None,
        }
    }
}

/// Reason why a call ended.
#[derive(Clone, Debug)]
pub enum CallEndReason {
    /// User hung up normally.
    HungUp,
    /// Call was rejected.
    Rejected,
    /// Call timed out (no answer).
    Timeout,
    /// Network or other error.
    Error(String),
    /// Remote party ended the call.
    RemoteHangup,
}

/// Information about a call participant.
#[derive(Clone, Debug)]
pub struct CallParticipant {
    /// The Matrix user ID of this participant.
    pub user_id: OwnedUserId,
    /// The device ID participating in the call.
    pub device_id: OwnedDeviceId,
    /// Display name of the participant.
    pub display_name: Option<String>,
    /// Video track identifier if video is enabled.
    pub video_track_id: Option<String>,
    /// Audio track identifier if audio is enabled.
    pub audio_track_id: Option<String>,
    /// Whether the participant's audio is muted.
    pub is_muted: bool,
    /// Whether the participant's video is off.
    pub is_video_off: bool,
    /// Whether this participant is sharing their screen.
    pub is_screen_sharing: bool,
    /// Whether this participant is currently speaking.
    pub is_speaking: bool,
}

impl CallParticipant {
    /// Create a new participant with the given user and device IDs.
    pub fn new(user_id: OwnedUserId, device_id: OwnedDeviceId) -> Self {
        Self {
            user_id,
            device_id,
            display_name: None,
            video_track_id: None,
            audio_track_id: None,
            is_muted: false,
            is_video_off: false,
            is_screen_sharing: false,
            is_speaking: false,
        }
    }

    /// Returns the displayable name for this participant.
    pub fn displayable_name(&self) -> &str {
        self.display_name
            .as_deref()
            .unwrap_or_else(|| self.user_id.as_str())
    }
}

/// Actions emitted by the call system to update the UI.
#[derive(Clone, Debug)]
pub enum CallAction {
    /// Call state changed for a room.
    StateChanged {
        room_id: OwnedRoomId,
        new_state: CallState,
    },
    /// Show the call screen for a room (used when joining a call).
    ShowCallScreen {
        room_id: OwnedRoomId,
        /// The display name of the local user.
        user_display_name: Option<String>,
    },
    /// Hide the call screen.
    HideCallScreen,
    /// A participant joined the call.
    ParticipantJoined {
        room_id: OwnedRoomId,
        participant: CallParticipant,
    },
    /// A participant left the call.
    ParticipantLeft {
        room_id: OwnedRoomId,
        user_id: OwnedUserId,
    },
    /// A participant's media state changed (mute, video, etc.).
    ParticipantMediaChanged {
        room_id: OwnedRoomId,
        user_id: OwnedUserId,
        is_muted: bool,
        is_video_off: bool,
    },
    /// Local media device error.
    MediaError {
        error: String,
    },
    /// Incoming call notification.
    IncomingCall {
        room_id: OwnedRoomId,
        caller: CallParticipant,
        is_video_call: bool,
    },
}
