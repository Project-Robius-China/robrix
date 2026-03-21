//! Media capture integration with Makepad.
//!
//! This module provides integration between WebRTC media tracks and
//! Makepad's video/audio input APIs for camera and microphone access.

use std::sync::{Arc, Mutex};

use makepad_widgets::Cx;

/// State of media capture.
#[derive(Clone, Debug, Default)]
pub struct MediaCaptureState {
    /// Whether video capture is enabled.
    pub video_enabled: bool,
    /// Whether audio capture is enabled.
    pub audio_enabled: bool,
    /// Whether video capture is muted (paused but still captured).
    pub video_muted: bool,
    /// Whether audio capture is muted.
    pub audio_muted: bool,
    /// The currently selected video input device.
    pub video_device_id: Option<String>,
    /// The currently selected audio input device.
    pub audio_device_id: Option<String>,
    /// Error message if capture failed.
    pub error: Option<String>,
}

/// Configuration for media capture.
#[derive(Clone, Debug)]
pub struct MediaCaptureConfig {
    /// Whether to enable video capture.
    pub enable_video: bool,
    /// Whether to enable audio capture.
    pub enable_audio: bool,
    /// Preferred video width.
    pub video_width: u32,
    /// Preferred video height.
    pub video_height: u32,
    /// Preferred video frame rate.
    pub video_fps: u32,
}

impl Default for MediaCaptureConfig {
    fn default() -> Self {
        Self {
            enable_video: true,
            enable_audio: true,
            video_width: 640,
            video_height: 480,
            video_fps: 30,
        }
    }
}

/// Manages media capture from camera and microphone.
pub struct MediaCapture {
    /// Current capture state.
    state: Arc<Mutex<MediaCaptureState>>,
    /// Configuration.
    config: MediaCaptureConfig,
    /// Video frame callback for processing captured video.
    video_callback: Option<Box<dyn FnMut(&[u8], u32, u32) + Send>>,
    /// Audio samples callback for processing captured audio.
    audio_callback: Option<Box<dyn FnMut(&[f32]) + Send>>,
}

impl MediaCapture {
    /// Create a new media capture instance.
    pub fn new(config: MediaCaptureConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(MediaCaptureState::default())),
            config,
            video_callback: None,
            audio_callback: None,
        }
    }

    /// Request camera and microphone permissions.
    pub fn request_permissions(_cx: &mut Cx) {
        // TODO: Request camera and microphone permissions via platform APIs
    }

    /// Check if we have camera permission.
    pub fn has_camera_permission(_cx: &Cx) -> bool {
        // TODO: Check camera permission
        true
    }

    /// Check if we have microphone permission.
    pub fn has_microphone_permission(_cx: &Cx) -> bool {
        // TODO: Check microphone permission
        true
    }

    /// Start video capture.
    pub fn start_video(&mut self, _cx: &mut Cx) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.video_enabled = true;
        state.error = None;
        // TODO: Implement actual video capture via Makepad platform APIs
        Ok(())
    }

    /// Stop video capture.
    pub fn stop_video(&mut self, _cx: &mut Cx) {
        let mut state = self.state.lock().unwrap();
        state.video_enabled = false;
        state.video_device_id = None;
    }

    /// Start audio capture.
    pub fn start_audio(&mut self, _cx: &mut Cx) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        state.audio_enabled = true;
        state.error = None;
        // TODO: Implement actual audio capture via Makepad platform APIs
        Ok(())
    }

    /// Stop audio capture.
    pub fn stop_audio(&mut self, _cx: &mut Cx) {
        let mut state = self.state.lock().unwrap();
        state.audio_enabled = false;
        state.audio_device_id = None;
    }

    /// Mute/unmute video.
    pub fn set_video_muted(&mut self, muted: bool) {
        let mut state = self.state.lock().unwrap();
        state.video_muted = muted;
    }

    /// Mute/unmute audio.
    pub fn set_audio_muted(&mut self, muted: bool) {
        let mut state = self.state.lock().unwrap();
        state.audio_muted = muted;
    }

    /// Toggle video mute state.
    pub fn toggle_video_muted(&mut self) -> bool {
        let mut state = self.state.lock().unwrap();
        state.video_muted = !state.video_muted;
        state.video_muted
    }

    /// Toggle audio mute state.
    pub fn toggle_audio_muted(&mut self) -> bool {
        let mut state = self.state.lock().unwrap();
        state.audio_muted = !state.audio_muted;
        state.audio_muted
    }

    /// Get current capture state.
    pub fn state(&self) -> MediaCaptureState {
        self.state.lock().unwrap().clone()
    }

    /// Check if video is currently enabled.
    pub fn is_video_enabled(&self) -> bool {
        self.state.lock().unwrap().video_enabled
    }

    /// Check if audio is currently enabled.
    pub fn is_audio_enabled(&self) -> bool {
        self.state.lock().unwrap().audio_enabled
    }

    /// Check if video is muted.
    pub fn is_video_muted(&self) -> bool {
        self.state.lock().unwrap().video_muted
    }

    /// Check if audio is muted.
    pub fn is_audio_muted(&self) -> bool {
        self.state.lock().unwrap().audio_muted
    }

    /// Set the callback for processing video frames.
    pub fn set_video_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&[u8], u32, u32) + Send + 'static,
    {
        self.video_callback = Some(Box::new(callback));
    }

    /// Set the callback for processing audio samples.
    pub fn set_audio_callback<F>(&mut self, callback: F)
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        self.audio_callback = Some(Box::new(callback));
    }

    /// Start both video and audio capture.
    pub fn start(&mut self, cx: &mut Cx) -> Result<(), String> {
        if self.config.enable_video {
            self.start_video(cx)?;
        }
        if self.config.enable_audio {
            self.start_audio(cx)?;
        }
        Ok(())
    }

    /// Stop both video and audio capture.
    pub fn stop(&mut self, cx: &mut Cx) {
        self.stop_video(cx);
        self.stop_audio(cx);
    }
}

/// Helper struct for managing video frame data.
#[derive(Clone, Debug)]
pub struct VideoFrame {
    /// YUV or RGB data depending on format.
    pub data: Vec<u8>,
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Timestamp in milliseconds.
    pub timestamp_ms: u64,
    /// Format of the frame data.
    pub format: VideoFrameFormat,
}

/// Video frame format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoFrameFormat {
    /// YUV420 planar format.
    Yuv420,
    /// RGBA format.
    Rgba,
    /// NV12 format (Y plane + interleaved UV).
    Nv12,
}

impl VideoFrame {
    /// Create a new video frame.
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: VideoFrameFormat) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            data,
            width,
            height,
            timestamp_ms,
            format,
        }
    }

    /// Check if the frame data is valid for the given dimensions and format.
    pub fn is_valid(&self) -> bool {
        let expected_size = match self.format {
            VideoFrameFormat::Yuv420 => {
                (self.width * self.height * 3 / 2) as usize
            }
            VideoFrameFormat::Rgba => {
                (self.width * self.height * 4) as usize
            }
            VideoFrameFormat::Nv12 => {
                (self.width * self.height * 3 / 2) as usize
            }
        };
        self.data.len() >= expected_size
    }
}
