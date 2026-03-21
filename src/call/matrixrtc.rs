//! MatrixRTC protocol implementation.
//!
//! This module handles the Matrix-specific signaling for WebRTC calls,
//! including state events for call membership and SDP exchange.
//!
//! The MatrixRTC protocol (MSC3401) defines how WebRTC calls are signaled
//! over Matrix rooms using state events.

use makepad_widgets::{log, Cx, SignalToUI};
use matrix_sdk::ruma::OwnedDeviceId;
use matrix_sdk::Client;
use serde::{Deserialize, Serialize};

use crate::call::webrtc_manager::webrtc_manager;
use crate::call::call_state::{CallAction, CallParticipant};

/// The state event type for MatrixRTC call membership.
pub const MATRIXRTC_MEMBER_EVENT_TYPE: &str = "org.matrix.msc3401.call.member";

/// Application type for standard Matrix calls.
pub const MATRIXRTC_APPLICATION_CALL: &str = "m.call";

/// Focus type for LiveKit SFU.
pub const FOCUS_TYPE_LIVEKIT: &str = "livekit";

/// MatrixRTC member state event content.
///
/// This is the content of the `org.matrix.msc3401.call.member` state event
/// that each participant sends to indicate their call membership.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatrixRTCMemberEvent {
    /// List of active call memberships for this user.
    #[serde(default)]
    pub memberships: Vec<MatrixRTCMembership>,
}

impl MatrixRTCMemberEvent {
    /// Create a new empty member event.
    pub fn new() -> Self {
        Self {
            memberships: Vec::new(),
        }
    }

    /// Create a member event with a single membership.
    pub fn with_membership(membership: MatrixRTCMembership) -> Self {
        Self {
            memberships: vec![membership],
        }
    }

    /// Add a membership to this event.
    pub fn add_membership(&mut self, membership: MatrixRTCMembership) {
        self.memberships.push(membership);
    }

    /// Remove expired memberships.
    pub fn remove_expired(&mut self, current_time_ms: u64) {
        self.memberships.retain(|m| !m.is_expired(current_time_ms));
    }

    /// Find a membership for the given device.
    pub fn find_membership(&self, device_id: &OwnedDeviceId) -> Option<&MatrixRTCMembership> {
        self.memberships.iter().find(|m| &m.device_id == device_id)
    }
}

impl Default for MatrixRTCMemberEvent {
    fn default() -> Self {
        Self::new()
    }
}

/// A single call membership entry.
///
/// Each device that joins a call creates a membership entry with information
/// about how to connect to it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatrixRTCMembership {
    /// The device ID of the participant.
    pub device_id: OwnedDeviceId,
    /// Unique identifier for this call session.
    pub call_id: String,
    /// Application type (e.g., "m.call").
    pub application: String,
    /// Scope of the call ("m.room" for room calls).
    #[serde(default = "default_scope")]
    pub scope: String,
    /// The currently active focus (SFU) for this participant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_active: Option<FocusActive>,
    /// Preferred foci (SFUs) for this participant.
    #[serde(default)]
    pub foci_preferred: Vec<FocusInfo>,
    /// Timestamp when this membership was created (ms since epoch).
    #[serde(default)]
    pub created_ts: u64,
    /// Membership expiration time in milliseconds.
    /// If not specified, defaults to 1 hour.
    #[serde(default = "default_expires_ms")]
    pub expires: u64,
}

fn default_scope() -> String {
    "m.room".to_string()
}

fn default_expires_ms() -> u64 {
    3_600_000 // 1 hour
}

impl MatrixRTCMembership {
    /// Create a new membership for the given device and call.
    pub fn new(device_id: OwnedDeviceId, call_id: String) -> Self {
        Self {
            device_id,
            call_id,
            application: MATRIXRTC_APPLICATION_CALL.to_string(),
            scope: default_scope(),
            focus_active: None,
            foci_preferred: Vec::new(),
            created_ts: 0,
            expires: default_expires_ms(),
        }
    }

    /// Set the creation timestamp.
    pub fn with_created_ts(mut self, ts: u64) -> Self {
        self.created_ts = ts;
        self
    }

    /// Set the active focus.
    pub fn with_focus_active(mut self, focus: FocusActive) -> Self {
        self.focus_active = Some(focus);
        self
    }

    /// Add a preferred focus.
    pub fn add_preferred_focus(&mut self, focus: FocusInfo) {
        self.foci_preferred.push(focus);
    }

    /// Check if this membership has expired.
    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        self.created_ts > 0 && current_time_ms > self.created_ts + self.expires
    }
}

/// Information about the currently active focus (SFU).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusActive {
    /// Type of focus (e.g., "livekit").
    #[serde(rename = "type")]
    pub focus_type: String,
    /// Selection strategy for this focus.
    #[serde(default)]
    pub focus_selection: String,
}

impl FocusActive {
    /// Create a new LiveKit focus.
    pub fn livekit() -> Self {
        Self {
            focus_type: FOCUS_TYPE_LIVEKIT.to_string(),
            focus_selection: "oldest_membership".to_string(),
        }
    }
}

/// Information about a focus (SFU) that can be used for the call.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FocusInfo {
    /// Type of focus (e.g., "livekit").
    #[serde(rename = "type")]
    pub focus_type: String,
    /// LiveKit-specific information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livekit_alias: Option<String>,
    /// LiveKit service URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livekit_service_url: Option<String>,
}

impl FocusInfo {
    /// Create a new LiveKit focus info.
    pub fn livekit(service_url: String, alias: Option<String>) -> Self {
        Self {
            focus_type: FOCUS_TYPE_LIVEKIT.to_string(),
            livekit_alias: alias,
            livekit_service_url: Some(service_url),
        }
    }
}

/// SDP offer/answer for WebRTC signaling.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionDescription {
    /// Type of SDP ("offer" or "answer").
    #[serde(rename = "type")]
    pub sdp_type: String,
    /// The SDP content.
    pub sdp: String,
}

impl SessionDescription {
    /// Create a new offer.
    pub fn offer(sdp: String) -> Self {
        Self {
            sdp_type: "offer".to_string(),
            sdp,
        }
    }

    /// Create a new answer.
    pub fn answer(sdp: String) -> Self {
        Self {
            sdp_type: "answer".to_string(),
            sdp,
        }
    }

    /// Check if this is an offer.
    pub fn is_offer(&self) -> bool {
        self.sdp_type == "offer"
    }

    /// Check if this is an answer.
    pub fn is_answer(&self) -> bool {
        self.sdp_type == "answer"
    }
}

/// ICE candidate for WebRTC connectivity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IceCandidate {
    /// The ICE candidate string.
    pub candidate: String,
    /// SDP mid (media stream identification).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_mid: Option<String>,
    /// SDP m-line index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_m_line_index: Option<u32>,
}

/// Call configuration from the homeserver.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CallConfiguration {
    /// TURN server URIs.
    #[serde(default)]
    pub turn_uris: Vec<String>,
    /// TURN username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_username: Option<String>,
    /// TURN password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_password: Option<String>,
    /// TURN credential TTL in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_ttl: Option<u64>,
    /// LiveKit service URL if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub livekit_service_url: Option<String>,
}

/// Generate a unique call ID.
pub fn generate_call_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 8] = rng.r#gen();
    hex::encode(bytes)
}

/// Helper to encode bytes as hex string.
mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: [u8; 8]) -> String {
        let mut s = String::with_capacity(16);
        for b in bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        s
    }
}

/// Add MatrixRTC event handlers to the client.
///
/// This registers handlers for call membership state events, allowing
/// the app to track call participants joining and leaving.
///
/// Note: MatrixRTC uses custom state events (org.matrix.msc3401.call.member)
/// that aren't part of the standard ruma events. We handle these by
/// processing raw state events in the timeline updates or using raw
/// event parsing when state changes occur.
pub fn add_matrixrtc_event_handlers(client: Client) {
    // For MatrixRTC events, we need to poll for room state changes
    // since the SDK doesn't have typed handlers for custom state events.
    // The actual event handling happens in handle_call_member_event_raw
    // which is called from the room list service when state changes occur.
    log!("MatrixRTC event handlers registered for client {:?}", client.user_id());
}

/// Process a raw MatrixRTC call member state event.
///
/// This function is called when we receive a raw state event that
/// might be a MatrixRTC membership event. It parses the event and
/// updates the call state accordingly.
pub async fn handle_call_member_event_raw(
    room_id: matrix_sdk::ruma::OwnedRoomId,
    sender_user_id: matrix_sdk::ruma::OwnedUserId,
    member_event: &MatrixRTCMemberEvent,
    local_user_id: Option<&matrix_sdk::ruma::UserId>,
) {
    log!(
        "Processing call member event in room {} from user {}: {} memberships",
        room_id,
        sender_user_id,
        member_event.memberships.len()
    );

    // Check if this is our own event
    let is_local_user = local_user_id == Some(sender_user_id.as_ref());

    let manager = webrtc_manager();

    if member_event.memberships.is_empty() {
        // User has left the call
        log!("User {} has left the call in room {}", sender_user_id, room_id);

        // Post action to update UI about participant leaving
        Cx::post_action(CallAction::ParticipantLeft {
            room_id: room_id.clone(),
            user_id: sender_user_id.clone(),
        });
        SignalToUI::set_ui_signal();
    } else {
        // User has joined or updated their call membership
        for membership in &member_event.memberships {
            log!(
                "User {}:{} joined/updated call {} in room {}",
                sender_user_id,
                membership.device_id,
                membership.call_id,
                room_id
            );

            if !is_local_user {
                // Handle remote participant joining
                let _ = manager
                    .handle_participant_joined(
                        &room_id,
                        sender_user_id.clone(),
                        membership.device_id.clone(),
                        membership,
                    )
                    .await;

                // Post action to update UI about new participant
                let participant = CallParticipant::new(
                    sender_user_id.clone(),
                    membership.device_id.clone(),
                );
                Cx::post_action(CallAction::ParticipantJoined {
                    room_id: room_id.clone(),
                    participant,
                });
                SignalToUI::set_ui_signal();
            }
        }
    }
}
