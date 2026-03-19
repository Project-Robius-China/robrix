//! A modal dialog for creating a Matrix child bot through BotFather slash commands.

use makepad_widgets::*;

use crate::{shared::styles::*, utils::RoomNameId};

live_design! {
    use link::theme::*;
    use link::widgets::*;

    use crate::shared::helpers::*;
    use crate::shared::icon_button::RobrixIconButton;
    use crate::shared::styles::*;

    ModalLabel = <Label> {
        width: Fill,
        height: Fit,
        draw_text: {
            text_style: <REGULAR_TEXT>{font_size: 10.5},
            color: #333,
            wrap: Word,
        }
        text: ""
    }

    pub CreateBotModal = {{CreateBotModal}} {
        width: Fit
        height: Fit

        <RoundedView> {
            width: 448
            height: Fit
            align: {x: 0.5}
            flow: Down
            padding: {top: 28, right: 24, bottom: 20, left: 24}
            spacing: 16

            show_bg: true
            draw_bg: {
                color: (COLOR_PRIMARY)
                border_radius: 6.0
            }

            title = <Label> {
                width: Fill,
                height: Fit,
                draw_text: {
                    text_style: <TITLE_TEXT>{font_size: 13},
                    color: #000,
                    wrap: Word,
                }
                text: "Create Bot"
            }

            body = <ModalLabel> {
                text: ""
            }

            form = <RoundedView> {
                width: Fill,
                height: Fit,
                flow: Down
                spacing: 12
                padding: 14

                show_bg: true
                draw_bg: {
                    color: (COLOR_SECONDARY)
                    border_radius: 4.0
                }

                username_label = <ModalLabel> {
                    text: "Username"
                }

                username_input = <RobrixTextInput> {
                    width: Fill,
                    height: Fit,
                    padding: 10,
                    draw_text: {
                        text_style: <REGULAR_TEXT>{font_size: 11.5},
                        color: #000,
                    }
                    empty_text: "weather"
                }

                username_hint = <ModalLabel> {
                    draw_text: {
                        text_style: <REGULAR_TEXT>{font_size: 9.5},
                        color: #666,
                    }
                    text: "Lowercase letters, digits, and underscores only. BotFather will create @bot_<username>:server."
                }

                display_name_label = <ModalLabel> {
                    text: "Display Name"
                }

                display_name_input = <RobrixTextInput> {
                    width: Fill,
                    height: Fit,
                    padding: 10,
                    draw_text: {
                        text_style: <REGULAR_TEXT>{font_size: 11.5},
                        color: #000,
                    }
                    empty_text: "Weather Bot"
                }

                prompt_label = <ModalLabel> {
                    text: "System Prompt (Optional)"
                }

                prompt_input = <RobrixTextInput> {
                    width: Fill,
                    height: Fit,
                    padding: 10,
                    draw_text: {
                        text_style: <REGULAR_TEXT>{font_size: 11.5},
                        color: #000,
                    }
                    empty_text: "You are a weather assistant."
                }
            }

            status_label = <Label> {
                width: Fill,
                height: Fit,
                draw_text: {
                    text_style: <REGULAR_TEXT>{font_size: 10.5},
                    color: #000,
                    wrap: Word,
                }
                text: ""
            }

            buttons = <View> {
                width: Fill,
                height: Fit
                flow: Right,
                align: {x: 1.0, y: 0.5}
                spacing: 16

                cancel_button = <RobrixIconButton> {
                    width: 110,
                    align: {x: 0.5, y: 0.5}
                    padding: 12,
                    draw_icon: {
                        svg_file: (ICON_FORBIDDEN)
                        color: (COLOR_TEXT),
                    }
                    icon_walk: {width: 16, height: 16, margin: {left: -2, right: -1}}
                    draw_bg: {
                        border_size: 0.75
                        border_color: (COLOR_BG_DISABLED),
                        color: (COLOR_SECONDARY)
                    }
                    draw_text: {
                        color: (COLOR_TEXT),
                    }
                    text: "Cancel"
                }

                create_button = <RobrixIconButton> {
                    width: 130,
                    align: {x: 0.5, y: 0.5}
                    padding: 12,
                    draw_icon: {
                        svg_file: (ICON_CHECKMARK)
                        color: (COLOR_FG_ACCEPT_GREEN),
                    }
                    icon_walk: {width: 16, height: 16, margin: {left: -2, right: -1}}
                    draw_bg: {
                        border_size: 0.75
                        border_color: (COLOR_FG_ACCEPT_GREEN),
                        color: (COLOR_BG_ACCEPT_GREEN)
                    }
                    draw_text: {
                        color: (COLOR_FG_ACCEPT_GREEN),
                    }
                    text: "Create Bot"
                }
            }
        }
    }
}

fn is_valid_bot_username(username: &str) -> bool {
    !username.is_empty()
        && username
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateBotRequest {
    pub username: String,
    pub display_name: String,
    pub system_prompt: Option<String>,
}

#[derive(Clone, Debug)]
pub enum CreateBotModalAction {
    Close,
    Submit(CreateBotRequest),
}

#[derive(Live, LiveHook, Widget)]
pub struct CreateBotModal {
    #[deref]
    view: View,
    #[rust]
    room_name_id: Option<RoomNameId>,
    #[rust]
    is_showing_error: bool,
}

impl Widget for CreateBotModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for CreateBotModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let cancel_button = self.view.button(ids!(buttons.cancel_button));
        let create_button = self.view.button(ids!(buttons.create_button));
        let username_input = self.view.text_input(ids!(form.username_input));
        let display_name_input = self.view.text_input(ids!(form.display_name_input));
        let prompt_input = self.view.text_input(ids!(form.prompt_input));
        let status_label = self.view.label(ids!(status_label));

        if cancel_button.clicked(actions)
            || actions
                .iter()
                .any(|a| matches!(a.downcast_ref(), Some(ModalAction::Dismissed)))
        {
            cx.action(CreateBotModalAction::Close);
            return;
        }

        if self.is_showing_error
            && (username_input.changed(actions).is_some()
                || display_name_input.changed(actions).is_some()
                || prompt_input.changed(actions).is_some())
        {
            self.is_showing_error = false;
            status_label.set_text(cx, "");
            self.view.redraw(cx);
        }

        if create_button.clicked(actions) || prompt_input.returned(actions).is_some() {
            let username = username_input.text().trim().to_string();
            if !is_valid_bot_username(&username) {
                self.is_showing_error = true;
                status_label.apply_over(
                    cx,
                    live! {
                        text: "Username must use lowercase letters, digits, or underscores.",
                        draw_text: {
                            color: (COLOR_FG_DANGER_RED),
                        }
                    },
                );
                self.view.redraw(cx);
                return;
            }

            let display_name = display_name_input.text().trim().to_string();
            let system_prompt = prompt_input.text().trim().to_string();

            cx.action(CreateBotModalAction::Submit(CreateBotRequest {
                username: username.clone(),
                display_name: if display_name.is_empty() {
                    username
                } else {
                    display_name
                },
                system_prompt: (!system_prompt.is_empty()).then_some(system_prompt),
            }));
        }
    }
}

impl CreateBotModal {
    pub fn show(&mut self, cx: &mut Cx, room_name_id: RoomNameId) {
        self.room_name_id = Some(room_name_id.clone());
        self.is_showing_error = false;

        self.view.label(ids!(title)).set_text(cx, "Create Room Bot");
        self.view.label(ids!(body)).set_text(
            cx,
            &format!(
                "Robrix will send `/createbot` to BotFather in {}. The bot becomes available immediately after octos creates it.",
                room_name_id
            ),
        );
        self.view
            .text_input(ids!(form.username_input))
            .set_text(cx, "");
        self.view
            .text_input(ids!(form.display_name_input))
            .set_text(cx, "");
        self.view
            .text_input(ids!(form.prompt_input))
            .set_text(cx, "");
        self.view.label(ids!(status_label)).set_text(cx, "");
        self.view
            .button(ids!(buttons.create_button))
            .set_enabled(cx, true);
        self.view
            .button(ids!(buttons.cancel_button))
            .set_enabled(cx, true);
        self.view
            .button(ids!(buttons.create_button))
            .reset_hover(cx);
        self.view
            .button(ids!(buttons.cancel_button))
            .reset_hover(cx);
        self.view.redraw(cx);
    }
}

impl CreateBotModalRef {
    pub fn show(&self, cx: &mut Cx, room_name_id: RoomNameId) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.show(cx, room_name_id);
    }
}
