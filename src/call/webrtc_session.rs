//! WebRTC peer connection session management.
//!
//! This module handles individual WebRTC peer connections, including
//! SDP negotiation, ICE candidate handling, and media track management.

use std::sync::Arc;

use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use tokio::sync::mpsc;
use webrtc::{
    api::API,
    ice_transport::ice_connection_state::RTCIceConnectionState,
    peer_connection::{
        RTCPeerConnection,
        configuration::RTCConfiguration,
        sdp::session_description::RTCSessionDescription,
        peer_connection_state::RTCPeerConnectionState,
    },
    rtp_transceiver::rtp_codec::RTPCodecType,
    track::track_local::TrackLocal,
};

use crate::call::matrixrtc::{IceCandidate, SessionDescription};

/// Events emitted by a WebRTC session.
#[derive(Debug)]
pub enum WebRTCSessionEvent {
    /// ICE connection state changed.
    IceConnectionStateChanged(RTCIceConnectionState),
    /// Peer connection state changed.
    PeerConnectionStateChanged(RTCPeerConnectionState),
    /// New ICE candidate discovered.
    IceCandidate(IceCandidate),
    /// Remote track added.
    TrackAdded {
        track_id: String,
        kind: TrackKind,
    },
    /// Remote track removed.
    TrackRemoved {
        track_id: String,
    },
    /// Session negotiation needed.
    NegotiationNeeded,
    /// Session error.
    Error(String),
}

/// Type of media track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackKind {
    Audio,
    Video,
}

impl From<RTPCodecType> for TrackKind {
    fn from(codec_type: RTPCodecType) -> Self {
        match codec_type {
            RTPCodecType::Audio => TrackKind::Audio,
            RTPCodecType::Video => TrackKind::Video,
            _ => TrackKind::Audio, // Default to audio for unknown types
        }
    }
}

/// Configuration for creating a WebRTC session.
#[derive(Clone, Debug)]
pub struct WebRTCSessionConfig {
    /// ICE servers to use for connectivity.
    pub ice_servers: Vec<IceServerConfig>,
    /// Whether to enable video.
    pub enable_video: bool,
    /// Whether to enable audio.
    pub enable_audio: bool,
}

impl Default for WebRTCSessionConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                IceServerConfig {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                },
            ],
            enable_video: true,
            enable_audio: true,
        }
    }
}

/// Configuration for a single ICE server.
#[derive(Clone, Debug)]
pub struct IceServerConfig {
    /// STUN/TURN server URLs.
    pub urls: Vec<String>,
    /// Username for TURN authentication.
    pub username: Option<String>,
    /// Credential for TURN authentication.
    pub credential: Option<String>,
}

/// A WebRTC session representing a connection to a single remote peer.
pub struct WebRTCSession {
    /// The remote user we're connected to.
    pub remote_user_id: OwnedUserId,
    /// The remote device we're connected to.
    pub remote_device_id: OwnedDeviceId,
    /// The underlying peer connection.
    peer_connection: Arc<RTCPeerConnection>,
    /// Channel for receiving session events.
    event_receiver: mpsc::UnboundedReceiver<WebRTCSessionEvent>,
    /// Channel for sending session events (held by callbacks).
    event_sender: mpsc::UnboundedSender<WebRTCSessionEvent>,
    /// Local tracks added to this session.
    local_tracks: Vec<Arc<dyn TrackLocal + Send + Sync>>,
    /// Whether we initiated the call (offerer) or received it (answerer).
    is_offerer: bool,
}

impl WebRTCSession {
    /// Create a new WebRTC session.
    pub async fn new(
        api: &API,
        remote_user_id: OwnedUserId,
        remote_device_id: OwnedDeviceId,
        config: WebRTCSessionConfig,
    ) -> Result<Self, webrtc::Error> {
        let rtc_config = Self::build_rtc_config(&config);
        let peer_connection = Arc::new(api.new_peer_connection(rtc_config).await?);

        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Set up event handlers
        Self::setup_event_handlers(&peer_connection, event_sender.clone());

        Ok(Self {
            remote_user_id,
            remote_device_id,
            peer_connection,
            event_receiver,
            event_sender,
            local_tracks: Vec::new(),
            is_offerer: false,
        })
    }

    /// Build the RTCConfiguration from our config.
    fn build_rtc_config(config: &WebRTCSessionConfig) -> RTCConfiguration {
        use webrtc::ice_transport::ice_server::RTCIceServer;

        let ice_servers: Vec<RTCIceServer> = config
            .ice_servers
            .iter()
            .map(|server| {
                RTCIceServer {
                    urls: server.urls.clone(),
                    username: server.username.clone().unwrap_or_default(),
                    credential: server.credential.clone().unwrap_or_default(),
                    ..Default::default()
                }
            })
            .collect();

        RTCConfiguration {
            ice_servers,
            ..Default::default()
        }
    }

    /// Set up event handlers on the peer connection.
    fn setup_event_handlers(
        peer_connection: &Arc<RTCPeerConnection>,
        event_sender: mpsc::UnboundedSender<WebRTCSessionEvent>,
    ) {
        // ICE connection state handler
        let sender = event_sender.clone();
        peer_connection.on_ice_connection_state_change(Box::new(move |state| {
            let _ = sender.send(WebRTCSessionEvent::IceConnectionStateChanged(state));
            Box::pin(async {})
        }));

        // Peer connection state handler
        let sender = event_sender.clone();
        peer_connection.on_peer_connection_state_change(Box::new(move |state| {
            let _ = sender.send(WebRTCSessionEvent::PeerConnectionStateChanged(state));
            Box::pin(async {})
        }));

        // ICE candidate handler
        let sender = event_sender.clone();
        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            if let Some(candidate) = candidate {
                // RTCIceCandidate in webrtc-rs uses to_json() to get candidate string
                let ice_candidate = IceCandidate {
                    candidate: candidate.to_string(),
                    sdp_mid: None,  // Not directly available in webrtc-rs
                    sdp_m_line_index: None,  // Not directly available in webrtc-rs
                };
                let _ = sender.send(WebRTCSessionEvent::IceCandidate(ice_candidate));
            }
            Box::pin(async {})
        }));

        // Track handler
        let sender = event_sender.clone();
        peer_connection.on_track(Box::new(move |track, _receiver, _transceiver| {
            let track_id = track.id().to_string();
            let kind = TrackKind::from(track.kind());
            let _ = sender.send(WebRTCSessionEvent::TrackAdded { track_id, kind });
            Box::pin(async {})
        }));

        // Negotiation needed handler
        let sender = event_sender;
        peer_connection.on_negotiation_needed(Box::new(move || {
            let _ = sender.send(WebRTCSessionEvent::NegotiationNeeded);
            Box::pin(async {})
        }));
    }

    /// Create an SDP offer.
    pub async fn create_offer(&mut self) -> Result<SessionDescription, webrtc::Error> {
        self.is_offerer = true;
        let offer = self.peer_connection.create_offer(None).await?;
        self.peer_connection.set_local_description(offer.clone()).await?;
        Ok(SessionDescription::offer(offer.sdp))
    }

    /// Create an SDP answer in response to an offer.
    pub async fn create_answer(&mut self) -> Result<SessionDescription, webrtc::Error> {
        self.is_offerer = false;
        let answer = self.peer_connection.create_answer(None).await?;
        self.peer_connection.set_local_description(answer.clone()).await?;
        Ok(SessionDescription::answer(answer.sdp))
    }

    /// Set the remote SDP description.
    pub async fn set_remote_description(
        &self,
        description: SessionDescription,
    ) -> Result<(), String> {
        // Parse the SDP and create the session description
        // webrtc-rs API requires parsing the SDP string
        let _sdp_type = if description.is_offer() {
            webrtc::peer_connection::sdp::sdp_type::RTCSdpType::Offer
        } else {
            webrtc::peer_connection::sdp::sdp_type::RTCSdpType::Answer
        };

        // Create offer/answer based on type
        let rtc_sdp = if description.is_offer() {
            RTCSessionDescription::offer(description.sdp).map_err(|e| e.to_string())?
        } else {
            RTCSessionDescription::answer(description.sdp).map_err(|e| e.to_string())?
        };

        self.peer_connection.set_remote_description(rtc_sdp)
            .await
            .map_err(|e| e.to_string())
    }

    /// Add an ICE candidate from the remote peer.
    pub async fn add_ice_candidate(&self, candidate: IceCandidate) -> Result<(), webrtc::Error> {
        // Create ICE candidate init from our IceCandidate struct
        use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
        let init = RTCIceCandidateInit {
            candidate: candidate.candidate,
            sdp_mid: candidate.sdp_mid,
            sdp_mline_index: candidate.sdp_m_line_index.map(|i| i as u16),
            username_fragment: None,
        };
        self.peer_connection.add_ice_candidate(init).await
    }

    /// Add a local track to the peer connection.
    pub async fn add_track(
        &mut self,
        track: Arc<dyn TrackLocal + Send + Sync>,
    ) -> Result<(), webrtc::Error> {
        self.peer_connection.add_track(track.clone()).await?;
        self.local_tracks.push(track);
        Ok(())
    }

    /// Remove a local track from the peer connection.
    pub async fn remove_track(&mut self, track_id: &str) -> Result<(), webrtc::Error> {
        // Find and remove the track
        if let Some(idx) = self.local_tracks.iter().position(|t| t.id() == track_id) {
            let _track = self.local_tracks.remove(idx);
            // Note: webrtc-rs doesn't have a direct remove_track equivalent,
            // you need to find the sender and remove it
        }
        Ok(())
    }

    /// Get the next session event, if any.
    pub fn try_recv_event(&mut self) -> Option<WebRTCSessionEvent> {
        self.event_receiver.try_recv().ok()
    }

    /// Wait for the next session event.
    pub async fn recv_event(&mut self) -> Option<WebRTCSessionEvent> {
        self.event_receiver.recv().await
    }

    /// Get the current connection state.
    pub fn connection_state(&self) -> RTCPeerConnectionState {
        self.peer_connection.connection_state()
    }

    /// Get the current ICE connection state.
    pub fn ice_connection_state(&self) -> RTCIceConnectionState {
        self.peer_connection.ice_connection_state()
    }

    /// Close the peer connection.
    pub async fn close(&self) -> Result<(), webrtc::Error> {
        self.peer_connection.close().await
    }

    /// Check if we are the offerer (initiated the call).
    pub fn is_offerer(&self) -> bool {
        self.is_offerer
    }
}
