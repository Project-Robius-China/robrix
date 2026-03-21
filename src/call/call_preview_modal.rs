//! Call preview modal widget for the pre-join call screen.
//!
//! This modal is shown when the user clicks the call button, allowing them
//! to preview their camera/mic settings before joining the call.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::sliding_sync::{MatrixRequest, submit_async_request};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Call preview modal colors
    mod.widgets.COLOR_PREVIEW_BG = #f5f5f5
    mod.widgets.COLOR_PREVIEW_CARD_BG = #ffffff
    mod.widgets.COLOR_AVATAR_BG = #fce4ec
    mod.widgets.COLOR_AVATAR_TEXT = #7b1fa2
    mod.widgets.COLOR_JOIN_BUTTON_BG = #1f1f1f
    mod.widgets.COLOR_JOIN_BUTTON_TEXT = #ffffff
    mod.widgets.COLOR_CONTROL_BUTTON_BG = #e0e0e0
    mod.widgets.COLOR_CONTROL_BUTTON_HOVER = #d0d0d0

    // Call preview modal widget
    mod.widgets.CallPreviewModal = #(CallPreviewModal::register_widget(vm)) {
        width: Fill
        height: Fill
        flow: Down
        align: Align{x: 0.5, y: 0.5}
        show_bg: true
        draw_bg +: {
            color: #00000080
        }

        // Main card
        preview_card := RoundedView {
            width: 300
            height: 400
            flow: Down
            align: Align{x: 0.5, y: 0.5}
            padding: 30
            spacing: 20
            show_bg: true
            draw_bg +: {
                color: (mod.widgets.COLOR_PREVIEW_CARD_BG)
                border_radius: 12.0
                shadow_offset: Vec2{x: 0.0, y: 4.0}
            }

            // Avatar container
            avatar_container := View {
                width: Fill
                height: Fit
                align: Align{x: 0.5, y: 0.5}
                padding: Inset{top: 20, bottom: 20}

                // Pink circle with initial
                avatar_circle := RoundedView {
                    width: 120
                    height: 120
                    align: Align{x: 0.5, y: 0.5}
                    show_bg: true
                    draw_bg +: {
                        color: (mod.widgets.COLOR_AVATAR_BG)
                        border_radius: 60.0
                    }

                    avatar_initial := Label {
                        width: Fit
                        height: Fit
                        align: Align{x: 0.5, y: 0.5}
                        draw_text +: {
                            text_style: TextStyle{font_size: 48}
                            color: (mod.widgets.COLOR_AVATAR_TEXT)
                        }
                        text: "A"
                    }
                }
            }

            // Join call button
            join_button := RoundedView {
                width: 160
                height: 50
                align: Align{x: 0.5, y: 0.5}
                cursor: MouseCursor.Hand
                show_bg: true
                draw_bg +: {
                    color: (mod.widgets.COLOR_JOIN_BUTTON_BG)
                    border_radius: 25.0
                }

                animator: Animator {
                    hover: {
                        default: @off
                        off: AnimatorState{
                            from: {all: Forward {duration: 0.15}}
                            apply: {
                                draw_bg.color: (mod.widgets.COLOR_JOIN_BUTTON_BG)
                            }
                        }
                        on: AnimatorState{
                            from: {all: Forward {duration: 0.15}}
                            apply: {
                                draw_bg.color: #333333
                            }
                        }
                    }
                }

                join_button_label := Label {
                    width: Fit
                    height: Fit
                    align: Align{x: 0.5, y: 0.5}
                    draw_text +: {
                        text_style: TextStyle{font_size: 14}
                        color: (mod.widgets.COLOR_JOIN_BUTTON_TEXT)
                    }
                    text: "Join call"
                }
            }

            // Spacer to push controls to bottom
            View {
                width: Fill
                height: Fill
            }

            // Bottom controls
            controls_row := View {
                width: Fill
                height: Fit
                flow: Right
                align: Align{x: 0.5, y: 0.5}
                spacing: 15

                // Mute button
                mute_button := RoundedView {
                    width: 50
                    height: 50
                    align: Align{x: 0.5, y: 0.5}
                    cursor: MouseCursor.Hand
                    show_bg: true
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                                }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_HOVER)
                                }
                            }
                        }
                        muted: {
                            default: @off
                            off: AnimatorState{
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                                }
                            }
                            on: AnimatorState{
                                apply: {
                                    draw_bg.color: #ff4444
                                }
                            }
                        }
                    }

                    mute_icon := Icon {
                        width: 20
                        height: 20
                        align: Align{x: 0.5, y: 0.5}
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/microphone.svg")
                            color: #666666
                        }
                    }
                }

                // Camera button
                camera_button := RoundedView {
                    width: 50
                    height: 50
                    align: Align{x: 0.5, y: 0.5}
                    cursor: MouseCursor.Hand
                    show_bg: true
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                                }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_HOVER)
                                }
                            }
                        }
                        camera_off: {
                            default: @off
                            off: AnimatorState{
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                                }
                            }
                            on: AnimatorState{
                                apply: {
                                    draw_bg.color: #ff4444
                                }
                            }
                        }
                    }

                    camera_icon := Icon {
                        width: 20
                        height: 20
                        align: Align{x: 0.5, y: 0.5}
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/camera.svg")
                            color: #666666
                        }
                    }
                }

                // Settings button
                settings_button := RoundedView {
                    width: 50
                    height: 50
                    align: Align{x: 0.5, y: 0.5}
                    cursor: MouseCursor.Hand
                    show_bg: true
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_BG)
                                }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: {
                                    draw_bg.color: (mod.widgets.COLOR_CONTROL_BUTTON_HOVER)
                                }
                            }
                        }
                    }

                    settings_icon := Icon {
                        width: 20
                        height: 20
                        align: Align{x: 0.5, y: 0.5}
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/settings.svg")
                            color: #666666
                        }
                    }
                }
            }
        }
    }
}

/// Actions emitted by the call preview modal.
#[derive(Clone, Debug, Default)]
pub enum CallPreviewModalAction {
    /// No action.
    #[default]
    None,
    /// User wants to open the modal for a room.
    Open {
        room_id: OwnedRoomId,
        user_display_name: Option<String>,
    },
    /// User clicked join call.
    JoinCall {
        room_id: OwnedRoomId,
        is_video_call: bool,
    },
    /// User dismissed the modal.
    Close,
}

impl ActionDefaultRef for CallPreviewModalAction {
    fn default_ref() -> &'static Self {
        static DEFAULT: CallPreviewModalAction = CallPreviewModalAction::None;
        &DEFAULT
    }
}

/// Call preview modal widget.
#[derive(Script, ScriptHook, Widget)]
pub struct CallPreviewModal {
    #[deref] view: View,

    /// The room ID for the call.
    #[rust] room_id: Option<OwnedRoomId>,
    /// User's display name for the avatar initial.
    #[rust] user_display_name: Option<String>,
    /// Whether audio is muted.
    #[rust] is_muted: bool,
    /// Whether camera is off.
    #[rust] is_camera_off: bool,
}

impl Widget for CallPreviewModal {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Capture actions for button clicks
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Check for join button click
        if self.view.view(cx, ids!(join_button)).finger_up(&actions).is_some() {
            if let Some(room_id) = self.room_id.clone() {
                // Submit the start call request
                submit_async_request(MatrixRequest::StartCall {
                    room_id: room_id.clone(),
                    is_video_call: !self.is_camera_off,
                });

                // Emit action to close modal
                cx.widget_action(
                    self.view.widget_uid(),
                    CallPreviewModalAction::JoinCall {
                        room_id,
                        is_video_call: !self.is_camera_off,
                    },
                );
            }
        }

        // Check for mute button click
        if self.view.view(cx, ids!(mute_button)).finger_up(&actions).is_some() {
            self.is_muted = !self.is_muted;
            self.update_mute_state(cx);
        }

        // Check for camera button click
        if self.view.view(cx, ids!(camera_button)).finger_up(&actions).is_some() {
            self.is_camera_off = !self.is_camera_off;
            self.update_camera_state(cx);
        }

        // Handle modal dismissed action
        if actions.iter().any(|a| matches!(a.downcast_ref(), Some(ModalAction::Dismissed))) {
            cx.widget_action(
                self.view.widget_uid(),
                CallPreviewModalAction::Close,
            );
        }
    }
}

impl CallPreviewModal {
    /// Show the modal for a room.
    pub fn show(&mut self, cx: &mut Cx, room_id: OwnedRoomId, user_display_name: Option<String>) {
        self.room_id = Some(room_id);
        self.user_display_name = user_display_name.clone();
        self.is_muted = false;
        self.is_camera_off = false;

        // Update avatar initial
        let initial = user_display_name
            .as_ref()
            .and_then(|n| n.chars().next())
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        self.view.label(cx, ids!(avatar_initial)).set_text(cx, &initial);

        self.update_mute_state(cx);
        self.update_camera_state(cx);
        self.view.redraw(cx);
    }

    /// Update the mute button visual state.
    fn update_mute_state(&mut self, cx: &mut Cx) {
        let mute_button = self.view.view(cx, ids!(mute_button));
        if self.is_muted {
            mute_button.animator_play(cx, ids!(muted.on));
        } else {
            mute_button.animator_play(cx, ids!(muted.off));
        }
    }

    /// Update the camera button visual state.
    fn update_camera_state(&mut self, cx: &mut Cx) {
        let camera_button = self.view.view(cx, ids!(camera_button));
        if self.is_camera_off {
            camera_button.animator_play(cx, ids!(camera_off.on));
        } else {
            camera_button.animator_play(cx, ids!(camera_off.off));
        }
    }
}

impl CallPreviewModalRef {
    /// Show the modal for a room.
    pub fn show(&self, cx: &mut Cx, room_id: OwnedRoomId, user_display_name: Option<String>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx, room_id, user_display_name);
        }
    }

    /// Check if the modal was closed via join.
    pub fn joined(&self, actions: &Actions) -> Option<(OwnedRoomId, bool)> {
        if let CallPreviewModalAction::JoinCall { room_id, is_video_call } =
            actions.find_widget_action(self.widget_uid()).cast_ref()
        {
            Some((room_id.clone(), *is_video_call))
        } else {
            None
        }
    }

    /// Check if the modal was closed/dismissed.
    pub fn closed(&self, actions: &Actions) -> bool {
        matches!(
            actions.find_widget_action(self.widget_uid()).cast_ref(),
            CallPreviewModalAction::Close
        )
    }
}
