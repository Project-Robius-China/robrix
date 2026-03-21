//! Call screen widget for displaying active video/audio calls.
//!
//! This widget provides the main UI for an active call, including
//! video preview, participant grid, and call controls.

use makepad_widgets::*;
use matrix_sdk::ruma::OwnedRoomId;

use crate::call::call_state::{CallState, CallAction, get_call_state};
use crate::sliding_sync::{MatrixRequest, submit_async_request};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Call screen colors
    mod.widgets.COLOR_CALL_BG = #e8e8e8
    mod.widgets.COLOR_CALL_AVATAR_BG = #fce4ec
    mod.widgets.COLOR_CALL_AVATAR_TEXT = #7b1fa2
    mod.widgets.COLOR_CALL_TEXT = #333333
    mod.widgets.COLOR_CALL_SUBTEXT = #666666
    mod.widgets.COLOR_CONTROL_BG = #e0e0e0
    mod.widgets.COLOR_CONTROL_HOVER = #d0d0d0
    mod.widgets.COLOR_HANGUP = #ff3b30

    // Main call screen widget
    mod.widgets.CallScreen = set_type_default() do #(CallScreen::register_widget(vm)) {
        width: Fill,
        height: Fill,
        flow: Overlay,
        show_bg: true,
        draw_bg +: {
            color: (mod.widgets.COLOR_CALL_BG)
        }

        // Main content area with avatar
        main_content := View {
            width: Fill,
            height: Fill,
            flow: Down,
            align: Align{x: 0.5, y: 0.5},

            // Large avatar circle
            avatar_container := View {
                width: Fit,
                height: Fit,
                align: Align{x: 0.5, y: 0.5},

                avatar_circle := RoundedView {
                    width: 200,
                    height: 200,
                    align: Align{x: 0.5, y: 0.5},
                    show_bg: true,
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CALL_AVATAR_BG)
                        border_radius: 100.0
                    }

                    avatar_initial := Label {
                        width: Fit,
                        height: Fit,
                        align: Align{x: 0.5, y: 0.5},
                        draw_text +: {
                            text_style: TextStyle{font_size: 72},
                            color: (mod.widgets.COLOR_CALL_AVATAR_TEXT)
                        }
                        text: "A"
                    }
                }
            }
        }

        // User name label at bottom left
        user_label_container := View {
            width: Fill,
            height: Fill,
            align: Align{x: 0.0, y: 1.0},
            padding: Inset{left: 20, bottom: 120},

            user_label_bg := RoundedView {
                width: Fit,
                height: Fit,
                padding: Inset{top: 6, bottom: 6, left: 10, right: 12},
                show_bg: true,
                draw_bg +: {
                    color: #ffffff
                    border_radius: 15.0
                }
                flow: Right,
                align: Align{x: 0.0, y: 0.5},
                spacing: 6,

                mic_icon := Icon {
                    width: 16,
                    height: 16,
                    draw_icon +: {
                        svg: crate_resource("self://resources/icons/microphone.svg")
                        color: #666666
                    }
                }

                user_name_label := Label {
                    width: Fit,
                    height: Fit,
                    draw_text +: {
                        text_style: TextStyle{font_size: 12},
                        color: (mod.widgets.COLOR_CALL_TEXT)
                    }
                    text: "User"
                }
            }
        }

        // Bottom controls bar
        controls_container := View {
            width: Fill,
            height: Fill,
            align: Align{x: 0.5, y: 1.0},
            padding: Inset{bottom: 30},

            controls_row := View {
                width: Fit,
                height: Fit,
                flow: Right,
                align: Align{x: 0.5, y: 0.5},
                spacing: 15,

                // Mute button
                mute_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CONTROL_BG)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_BG) }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_HOVER) }
                            }
                        }
                        muted: {
                            default: @off
                            off: AnimatorState{
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_BG) }
                            }
                            on: AnimatorState{
                                apply: { draw_bg.color: #ff4444 }
                            }
                        }
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/microphone.svg")
                            color: #666666
                        }
                    }
                }

                // Camera button
                camera_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: (mod.widgets.COLOR_CONTROL_BG)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_BG) }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_HOVER) }
                            }
                        }
                        camera_off: {
                            default: @off
                            off: AnimatorState{
                                apply: { draw_bg.color: (mod.widgets.COLOR_CONTROL_BG) }
                            }
                            on: AnimatorState{
                                apply: { draw_bg.color: #ff4444 }
                            }
                        }
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/camera.svg")
                            color: #666666
                        }
                    }
                }

                // Screen share button
                screen_share_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: #ffffff
                        border_radius: 25.0
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/upload.svg")
                            color: #333333
                        }
                    }
                }

                // Emoji/reactions button
                emoji_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: #ffffff
                        border_radius: 25.0
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/add_reaction.svg")
                            color: #333333
                        }
                    }
                }

                // Settings button
                settings_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: #ffffff
                        border_radius: 25.0
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/settings.svg")
                            color: #333333
                        }
                    }
                }

                // Hang up button (red)
                hangup_button := RoundedView {
                    width: 50,
                    height: 50,
                    align: Align{x: 0.5, y: 0.5},
                    cursor: MouseCursor.Hand,
                    show_bg: true,
                    draw_bg +: {
                        color: (mod.widgets.COLOR_HANGUP)
                        border_radius: 25.0
                    }

                    animator: Animator {
                        hover: {
                            default: @off
                            off: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: (mod.widgets.COLOR_HANGUP) }
                            }
                            on: AnimatorState{
                                from: {all: Forward {duration: 0.15}}
                                apply: { draw_bg.color: #ff5a52 }
                            }
                        }
                    }

                    Icon {
                        width: 22,
                        height: 22,
                        align: Align{x: 0.5, y: 0.5},
                        draw_icon +: {
                            svg: crate_resource("self://resources/icons/phone_hangup.svg")
                            color: #ffffff
                        }
                    }
                }
            }
        }
    }
}

/// The main call screen widget.
#[derive(Script, ScriptHook, Widget)]
pub struct CallScreen {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,

    /// The room ID of the current call.
    #[rust] room_id: Option<OwnedRoomId>,
    /// User's display name.
    #[rust] user_display_name: Option<String>,
    /// Call start time for duration display.
    #[rust] call_start_time: Option<std::time::Instant>,
    /// Whether audio is muted.
    #[rust] is_muted: bool,
    /// Whether camera is off.
    #[rust] is_camera_off: bool,
}

impl Widget for CallScreen {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Capture actions for button clicks
        let actions = cx.capture_actions(|cx| {
            self.view.handle_event(cx, event, scope);
        });

        // Handle mute button click
        if self.view.view(cx, ids!(mute_button)).finger_up(&actions).is_some() {
            self.is_muted = !self.is_muted;
            self.update_mute_state(cx);
            if let Some(room_id) = &self.room_id {
                submit_async_request(MatrixRequest::ToggleCallAudio { room_id: room_id.clone() });
            }
        }

        // Handle camera button click
        if self.view.view(cx, ids!(camera_button)).finger_up(&actions).is_some() {
            self.is_camera_off = !self.is_camera_off;
            self.update_camera_state(cx);
            if let Some(room_id) = &self.room_id {
                submit_async_request(MatrixRequest::ToggleCallVideo { room_id: room_id.clone() });
            }
        }

        // Handle hangup button click
        if self.view.view(cx, ids!(hangup_button)).finger_up(&actions).is_some() {
            if let Some(room_id) = &self.room_id {
                submit_async_request(MatrixRequest::LeaveCall { room_id: room_id.clone() });
                cx.action(CallAction::HideCallScreen);
            }
        }

        // Handle call state changes on signal
        if let Event::Signal = event {
            if let Some(room_id) = &self.room_id {
                if let Some(state) = get_call_state(room_id) {
                    self.update_ui_for_state(cx, &state);
                }
            }
        }
    }
}

impl CallScreen {
    /// Set the room for this call screen.
    pub fn set_room(&mut self, cx: &mut Cx, room_id: OwnedRoomId, user_display_name: &str) {
        self.room_id = Some(room_id);
        self.user_display_name = Some(user_display_name.to_string());

        // Set the user name label
        self.view.label(cx, ids!(user_name_label)).set_text(cx, user_display_name);

        // Set the avatar initial
        let initial = user_display_name
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string();
        self.view.label(cx, ids!(avatar_initial)).set_text(cx, &initial);

        // Reset mute states
        self.is_muted = false;
        self.is_camera_off = false;
        self.update_mute_state(cx);
        self.update_camera_state(cx);
    }

    /// Start the call timer.
    pub fn start_call_timer(&mut self, _cx: &mut Cx) {
        self.call_start_time = Some(std::time::Instant::now());
    }

    /// Stop the call timer.
    pub fn stop_call_timer(&mut self) {
        self.call_start_time = None;
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

    /// Update the UI based on call state.
    fn update_ui_for_state(&mut self, _cx: &mut Cx, _state: &CallState) {
        // Update UI based on call state if needed
    }

    /// Update local video mute indicator.
    pub fn set_local_muted(&mut self, cx: &mut Cx, muted: bool) {
        self.is_muted = muted;
        self.update_mute_state(cx);
    }
}

impl CallScreenRef {
    /// Set the room for this call screen.
    pub fn set_room(&self, cx: &mut Cx, room_id: OwnedRoomId, room_name: &str) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_room(cx, room_id, room_name);
        }
    }

    /// Start the call timer.
    pub fn start_call_timer(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.start_call_timer(cx);
        }
    }

    /// Stop the call timer.
    pub fn stop_call_timer(&self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.stop_call_timer();
        }
    }

    /// Update local video mute indicator.
    pub fn set_local_muted(&self, cx: &mut Cx, muted: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_local_muted(cx, muted);
        }
    }
}
