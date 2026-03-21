//! Call module for WebRTC video/audio calling support.
//!
//! This module implements native WebRTC calling using the MatrixRTC protocol,
//! webrtc-rs crate, and LiveKit SFU for scalable multi-party calls.

use makepad_widgets::*;

pub mod call_state;
pub mod matrixrtc;
pub mod webrtc_session;
pub mod webrtc_manager;
pub mod media_capture;
pub mod livekit_client;
pub mod call_screen;
pub mod video_preview;
pub mod call_controls;
pub mod call_preview_modal;

// Re-export commonly used types
pub use call_state::{CallState, CallParticipant};
pub use matrixrtc::{MatrixRTCMemberEvent, MatrixRTCMembership};
pub use call_preview_modal::{CallPreviewModalAction, CallPreviewModalRef};

/// Register call module widgets with the script VM.
pub fn script_mod(vm: &mut ScriptVm) {
    call_screen::script_mod(vm);
    video_preview::script_mod(vm);
    call_controls::script_mod(vm);
    call_preview_modal::script_mod(vm);
}
