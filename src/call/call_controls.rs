//! Call controls widget for mute, camera toggle, and hang up buttons.
//!
//! This widget provides the control bar at the bottom of a call screen
//! with buttons for toggling audio/video and ending the call.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedRoomId;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Call control colors
    mod.widgets.COLOR_CONTROL_BG = #2d2d2d
    mod.widgets.COLOR_BUTTON_BG = #444444
    mod.widgets.COLOR_BUTTON_HOVER = #555555
    mod.widgets.COLOR_BUTTON_ACTIVE = #1a73e8
    mod.widgets.COLOR_BUTTON_MUTED = #ff4444
    mod.widgets.COLOR_END_CALL = #ff3b30

    // Call controls widget
    mod.widgets.CallControls = set_type_default() do #(CallControls::register_widget(vm)) {
        width: Fill,
        height: 100,
        flow: Right,
        align: Align{x: 0.5, y: 0.5},
        spacing: 20,
        padding: 20,
        show_bg: true,
        draw_bg +: {
            color: (mod.widgets.COLOR_CONTROL_BG)
        }

        // Mute/unmute audio button
        mute_button := RoundedView {
            width: 60,
            height: 60,
            align: Align{x: 0.5, y: 0.5},
            cursor: MouseCursor.Hand,
            show_bg: true,
            draw_bg +: {
                color: (mod.widgets.COLOR_BUTTON_BG)
                border_radius: 30.0
            }

            animator: Animator {
                hover: {
                    default: @off
                    off: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_BG)
                        }
                    }
                    on: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_HOVER)
                        }
                    }
                }
                muted: {
                    default: @off
                    off: AnimatorState{
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_BG)
                        }
                    }
                    on: AnimatorState{
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_MUTED)
                        }
                    }
                }
            }

            Icon {
                width: 24,
                height: 24,
                align: Align{x: 0.5, y: 0.5},
                draw_icon +: {
                    svg: crate_resource("self://resources/icons/microphone.svg")
                    color: #ffffff
                }
            }
        }

        // Toggle camera button
        camera_button := RoundedView {
            width: 60,
            height: 60,
            align: Align{x: 0.5, y: 0.5},
            cursor: MouseCursor.Hand,
            show_bg: true,
            draw_bg +: {
                color: (mod.widgets.COLOR_BUTTON_BG)
                border_radius: 30.0
            }

            animator: Animator {
                hover: {
                    default: @off
                    off: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_BG)
                        }
                    }
                    on: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_HOVER)
                        }
                    }
                }
                camera_off: {
                    default: @off
                    off: AnimatorState{
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_BG)
                        }
                    }
                    on: AnimatorState{
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_BUTTON_MUTED)
                        }
                    }
                }
            }

            Icon {
                width: 24,
                height: 24,
                align: Align{x: 0.5, y: 0.5},
                draw_icon +: {
                    svg: crate_resource("self://resources/icons/camera.svg")
                    color: #ffffff
                }
            }
        }

        // End call button
        end_call_button := RoundedView {
            width: 80,
            height: 60,
            align: Align{x: 0.5, y: 0.5},
            cursor: MouseCursor.Hand,
            show_bg: true,
            draw_bg +: {
                color: (mod.widgets.COLOR_END_CALL)
                border_radius: 30.0
            }

            animator: Animator {
                hover: {
                    default: @off
                    off: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: (mod.widgets.COLOR_END_CALL)
                        }
                    }
                    on: AnimatorState{
                        from: {all: Forward {duration: 0.15}}
                        apply: {
                            draw_bg.color: #ff5a52
                        }
                    }
                }
            }

            Icon {
                width: 24,
                height: 24,
                align: Align{x: 0.5, y: 0.5},
                draw_icon +: {
                    svg: crate_resource("self://resources/icons/phone_hangup.svg")
                    color: #ffffff
                }
            }
        }
    }
}

/// Actions emitted by call controls.
#[derive(Clone, Debug, Default)]
pub enum CallControlsAction {
    /// No action.
    #[default]
    None,
    /// Toggle audio mute.
    ToggleMute {
        room_id: OwnedRoomId,
    },
    /// Toggle camera.
    ToggleCamera {
        room_id: OwnedRoomId,
    },
    /// End the call.
    EndCall {
        room_id: OwnedRoomId,
    },
}

/// Call controls widget.
#[derive(Script, ScriptHook, Widget)]
pub struct CallControls {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,

    /// The room ID of the current call.
    #[rust] room_id: Option<OwnedRoomId>,
    /// Whether audio is muted.
    #[rust] is_muted: bool,
    /// Whether camera is off.
    #[rust] is_camera_off: bool,
}

impl Widget for CallControls {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle button clicks
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Check for mute button click
        if self.view.view(cx, ids!(mute_button)).finger_up(&actions).is_some() {
            if let Some(room_id) = &self.room_id {
                self.is_muted = !self.is_muted;
                cx.widget_action(
                    self.view.widget_uid(),
                    CallControlsAction::ToggleMute { room_id: room_id.clone() },
                );
            }
        }

        // Check for camera button click
        if self.view.view(cx, ids!(camera_button)).finger_up(&actions).is_some() {
            if let Some(room_id) = &self.room_id {
                self.is_camera_off = !self.is_camera_off;
                cx.widget_action(
                    self.view.widget_uid(),
                    CallControlsAction::ToggleCamera { room_id: room_id.clone() },
                );
            }
        }

        // Check for end call button click
        if self.view.view(cx, ids!(end_call_button)).finger_up(&actions).is_some() {
            if let Some(room_id) = &self.room_id {
                cx.widget_action(
                    self.view.widget_uid(),
                    CallControlsAction::EndCall { room_id: room_id.clone() },
                );
            }
        }
    }
}

impl CallControls {
    /// Set the room ID for this call.
    pub fn set_room_id(&mut self, room_id: OwnedRoomId) {
        self.room_id = Some(room_id);
    }

    /// Set the mute state.
    pub fn set_muted(&mut self, _cx: &mut Cx, muted: bool) {
        self.is_muted = muted;
    }

    /// Set the camera state.
    pub fn set_camera_off(&mut self, _cx: &mut Cx, off: bool) {
        self.is_camera_off = off;
    }
}

impl CallControlsRef {
    /// Set the room ID for this call.
    pub fn set_room_id(&self, room_id: OwnedRoomId) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_room_id(room_id);
        }
    }

    /// Set the mute state.
    pub fn set_muted(&self, cx: &mut Cx, muted: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_muted(cx, muted);
        }
    }

    /// Set the camera state.
    pub fn set_camera_off(&self, cx: &mut Cx, off: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_camera_off(cx, off);
        }
    }
}
