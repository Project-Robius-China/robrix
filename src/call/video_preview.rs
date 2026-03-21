//! Video preview widget for displaying camera and remote video streams.
//!
//! This widget uses Makepad's Video widget with YUV rendering for
//! displaying both local camera preview and remote participant video.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Video preview colors
    mod.widgets.COLOR_VIDEO_BG = #000000
    mod.widgets.COLOR_VIDEO_PLACEHOLDER = #333333
    mod.widgets.COLOR_VIDEO_BORDER = #444444

    // Video preview widget
    mod.widgets.VideoPreview = set_type_default() do #(VideoPreview::register_widget(vm)) {
        width: Fill,
        height: Fill,
        flow: Overlay,
        show_bg: true,
        draw_bg +: {
            color: (mod.widgets.COLOR_VIDEO_BG)
        }

        // Placeholder when no video
        placeholder := View {
            visible: true,
            width: Fill,
            height: Fill,
            align: Align{x: 0.5, y: 0.5},
            show_bg: true,
            draw_bg +: {
                color: (mod.widgets.COLOR_VIDEO_PLACEHOLDER)
            }

            placeholder_icon := View {
                width: 64,
                height: 64,
                align: Align{x: 0.5, y: 0.5},
                show_bg: true,
                draw_bg +: {
                    color: #555555
                }
            }
        }

        // Video display area
        video_view := View {
            visible: false,
            width: Fill,
            height: Fill,
            align: Align{x: 0.5, y: 0.5},

            video := Video {
                width: Fill,
                height: Fill,
            }
        }

        // Participant name overlay
        name_overlay := View {
            width: Fill,
            height: Fit,
            align: Align{x: 0.0, y: 1.0},
            padding: 8,
            show_bg: true,
            draw_bg +: {
                color: #00000080
            }

            name_label := Label {
                width: Fill,
                height: Fit,
                draw_text +: {
                    text_style: TextStyle{font_size: 12},
                    color: #ffffff
                }
                text: ""
            }
        }

        // Muted indicator
        muted_indicator := View {
            visible: false,
            width: 24,
            height: 24,
            align: Align{x: 1.0, y: 0.0},
            margin: 8,
            show_bg: true,
            draw_bg +: {
                color: #ff4444
            }
        }
    }
}

/// Video preview widget for displaying a video stream.
#[derive(Script, ScriptHook, Widget)]
pub struct VideoPreview {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,

    /// Whether this is showing local camera video.
    #[rust] is_local: bool,
    /// Participant name to display.
    #[rust] participant_name: String,
    /// Whether the participant is muted.
    #[rust] is_muted: bool,
    /// Whether video is available.
    #[rust] has_video: bool,
}

impl Widget for VideoPreview {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl VideoPreview {
    /// Set whether this is a local camera preview.
    pub fn set_local(&mut self, is_local: bool) {
        self.is_local = is_local;
    }

    /// Set the participant name.
    pub fn set_participant_name(&mut self, cx: &mut Cx, name: &str) {
        self.participant_name = name.to_string();
        self.view.label(cx, ids!(name_label)).set_text(cx, name);
        // Hide name overlay for local preview
        if self.is_local && name.is_empty() {
            self.view.view(cx, ids!(name_overlay)).set_visible(cx, false);
        } else {
            self.view.view(cx, ids!(name_overlay)).set_visible(cx, true);
        }
    }

    /// Set whether the participant is muted.
    pub fn set_muted(&mut self, cx: &mut Cx, muted: bool) {
        self.is_muted = muted;
        self.view.view(cx, ids!(muted_indicator)).set_visible(cx, muted);
    }

    /// Set whether video is available.
    pub fn set_has_video(&mut self, cx: &mut Cx, has_video: bool) {
        self.has_video = has_video;
        self.view.view(cx, ids!(placeholder)).set_visible(cx, !has_video);
        self.view.view(cx, ids!(video_view)).set_visible(cx, has_video);
    }

    /// Update the video frame with new YUV data.
    pub fn update_frame(&mut self, cx: &mut Cx, _data: &[u8], _width: u32, _height: u32) {
        // In a real implementation, this would update the Video widget
        // with new YUV frame data for rendering.
        if !self.has_video {
            self.set_has_video(cx, true);
        }
        // TODO: Pass frame data to the Video widget
    }
}

impl VideoPreviewRef {
    /// Set whether this is a local camera preview.
    pub fn set_local(&self, is_local: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_local(is_local);
        }
    }

    /// Set the participant name.
    pub fn set_participant_name(&self, cx: &mut Cx, name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_participant_name(cx, name);
        }
    }

    /// Set whether the participant is muted.
    pub fn set_muted(&self, cx: &mut Cx, muted: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_muted(cx, muted);
        }
    }

    /// Set whether video is available.
    pub fn set_has_video(&self, cx: &mut Cx, has_video: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_has_video(cx, has_video);
        }
    }

    /// Update the video frame with new YUV data.
    pub fn update_frame(&self, cx: &mut Cx, data: &[u8], width: u32, height: u32) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_frame(cx, data, width, height);
        }
    }
}
