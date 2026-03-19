use makepad_widgets::*;

use crate::{
    app::{AppState, BotSettingsState},
    shared::popup_list::{PopupKind, enqueue_popup_notification},
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::helpers::*;
    use crate::shared::icon_button::*;
    use crate::shared::styles::*;

    BotSettingsInfoLabel = <Label> {
        width: Fill, height: Fit
        margin: {left: 5, top: 2, bottom: 2}
        draw_text: {
            wrap: Word,
            color: (MESSAGE_TEXT_COLOR),
            text_style: <REGULAR_TEXT>{ font_size: 10.5 },
        }
        text: ""
    }

    BotEnableSwitch = {{BotEnableSwitch}} {
        width: 44,
        height: 24,
        flow: Overlay,

        track = <View> {
            width: Fill,
            height: Fill,
            show_bg: true
            draw_bg: {
                instance on: 0.0
                instance hover: 0.0

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let sz = self.rect_size;
                    let r = sz.y * 0.5;

                    sdf.circle(r, r, r);
                    sdf.rect(r, 0.0, sz.x - sz.y, sz.y);
                    sdf.circle(sz.x - r, r, r);

                    let bg_off = (COLOR_BG_DISABLED);
                    let bg_on = (COLOR_ACTIVE_PRIMARY);
                    let color = mix(bg_off, bg_on, self.on);
                    let color = mix(color, (COLOR_PRIMARY_DARKER), self.hover * 0.08);

                    sdf.fill(color);
                    return sdf.result;
                }
            }
        }

        thumb_wrap = <View> {
            width: Fill,
            height: Fill,
            align: {x: 0.0, y: 0.5}
            padding: {left: 3, right: 3}

            thumb = <View> {
                width: 18,
                height: 18,
                show_bg: true
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        let radius = self.rect_size.y * 0.5;

                        sdf.circle(radius, radius, radius);
                        sdf.fill((COLOR_PRIMARY));
                        return sdf.result;
                    }
                }
            }
        }

        animator: {
            hover = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.15}}
                    apply: {track = {draw_bg: {hover: 0.0}}}
                }
                on = {
                    from: {all: Forward {duration: 0.1}}
                    apply: {track = {draw_bg: {hover: 1.0}}}
                }
            }
            on = {
                default: off
                off = {
                    from: {all: Forward {duration: 0.2}}
                    apply: {
                        track = {draw_bg: {on: 0.0}}
                        thumb_wrap = {align: {x: 0.0}}
                    }
                }
                on = {
                    from: {all: Forward {duration: 0.2}}
                    apply: {
                        track = {draw_bg: {on: 1.0}}
                        thumb_wrap = {align: {x: 1.0}}
                    }
                }
            }
        }
    }

    pub BotSettings = {{BotSettings}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 10

        <TitleLabel> {
            text: "App Service"
        }

        description = <BotSettingsInfoLabel> {
            margin: {left: 5, right: 8, bottom: 4}
            text: "Enable Matrix app service support here. Robrix stays a normal Matrix client: it binds BotFather to a room and sends the matching slash commands."
        }

        enable_row = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 0.0, y: 0.5}
            spacing: 12
            margin: {left: 5, bottom: 2}

            enable_label = <SubsectionLabel> {
                width: Fit, height: Fit
                margin: 0
                text: "Enable App Service"
            }

            enable_switch = <BotEnableSwitch> {}
        }

        bot_details = <View> {
            visible: false
            width: Fill, height: Fit
            flow: Down

            <SubsectionLabel> {
                text: "BotFather User ID:"
            }

            bot_user_id_input = <SimpleTextInput> {
                margin: {top: 2, left: 5, right: 5, bottom: 8},
                width: 280, height: Fit
                empty_text: "bot or @bot:server"
            }

            save_button = <RobrixIconButton> {
                width: Fit, height: Fit
                padding: {top: 10, bottom: 10, left: 12, right: 15}
                margin: {left: 5}

                draw_bg: {
                    color: (COLOR_ACTIVE_PRIMARY)
                }
                draw_icon: {
                    svg_file: (ICON_CHECKMARK)
                    color: (COLOR_PRIMARY)
                }
                draw_text: {
                    color: (COLOR_PRIMARY)
                    text_style: <REGULAR_TEXT> {}
                }
                icon_walk: {width: 16, height: 16}
                text: "Save"
            }
        }
    }
}

#[derive(Clone, Debug, DefaultNone)]
pub enum BotEnableSwitchAction {
    Changed(bool),
    None,
}

#[derive(Live, LiveHook, Widget)]
pub struct BotEnableSwitch {
    #[deref]
    view: View,
    #[live]
    on: bool,
    #[live]
    disabled: bool,
    #[animator]
    animator: Animator,
}

impl Widget for BotEnableSwitch {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        let uid = self.widget_uid();

        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }

        self.view.handle_event(cx, event, scope);

        if self.disabled {
            return;
        }

        match event.hits(cx, self.view.area()) {
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                self.animator_play(cx, ids!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                cx.set_cursor(MouseCursor::Default);
                self.animator_play(cx, ids!(hover.off));
            }
            Hit::FingerUp(fe) => {
                if fe.is_over {
                    self.on = !self.on;
                    if self.on {
                        self.animator_play(cx, ids!(on.on));
                    } else {
                        self.animator_play(cx, ids!(on.off));
                    }
                    cx.widget_action(uid, &scope.path, BotEnableSwitchAction::Changed(self.on));
                    self.redraw(cx);
                }
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.on {
            self.view.view(ids!(track)).apply_over(
                cx,
                live! {
                    draw_bg: {on: 1.0}
                },
            );
            self.view.view(ids!(thumb_wrap)).apply_over(
                cx,
                live! {
                    align: {x: 1.0}
                },
            );
        } else {
            self.view.view(ids!(track)).apply_over(
                cx,
                live! {
                    draw_bg: {on: 0.0}
                },
            );
            self.view.view(ids!(thumb_wrap)).apply_over(
                cx,
                live! {
                    align: {x: 0.0}
                },
            );
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl BotEnableSwitch {
    pub fn set_on(&mut self, cx: &mut Cx, on: bool) {
        self.on = on;
        if on {
            self.animator_play(cx, ids!(on.on));
        } else {
            self.animator_play(cx, ids!(on.off));
        }
        self.redraw(cx);
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(action) = actions.find_widget_action(self.widget_uid()) {
            if let BotEnableSwitchAction::Changed(on) = action.cast() {
                return Some(on);
            }
        }
        None
    }
}

impl BotEnableSwitchRef {
    pub fn set_on(&self, cx: &mut Cx, on: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_on(cx, on);
        }
    }

    pub fn changed(&self, actions: &Actions) -> Option<bool> {
        if let Some(inner) = self.borrow() {
            inner.changed(actions)
        } else {
            None
        }
    }
}

/// The view containing bot-related settings.
#[derive(Live, LiveHook, Widget)]
pub struct BotSettings {
    #[deref]
    view: View,
}

impl Widget for BotSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for BotSettings {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, scope: &mut Scope) {
        let enable_switch = self.view.bot_enable_switch(ids!(enable_switch));
        let bot_details = self.view.view(ids!(bot_details));
        let bot_user_id_input = self.view.text_input(ids!(bot_user_id_input));
        let save_button = self.view.button(ids!(save_button));

        let Some(app_state) = scope.data.get_mut::<AppState>() else {
            return;
        };

        if let Some(enabled) = enable_switch.changed(actions) {
            app_state.bot_settings.enabled = enabled;
            bot_details.set_visible(cx, enabled);
            self.view.redraw(cx);
        }

        if save_button.clicked(actions) {
            app_state.bot_settings.botfather_user_id = bot_user_id_input.text().trim().to_string();
            enqueue_popup_notification(
                "Saved Matrix app service settings.",
                PopupKind::Success,
                Some(3.0),
            );
        }
    }
}

impl BotSettings {
    /// Populates the bot settings UI from the current persisted app state.
    pub fn populate(&mut self, cx: &mut Cx, bot_settings: &BotSettingsState) {
        self.view
            .bot_enable_switch(ids!(enable_switch))
            .set_on(cx, bot_settings.enabled);
        self.view
            .view(ids!(bot_details))
            .set_visible(cx, bot_settings.enabled);
        self.view
            .text_input(ids!(bot_user_id_input))
            .set_text(cx, &bot_settings.botfather_user_id);
        self.view.button(ids!(save_button)).reset_hover(cx);
        self.view.redraw(cx);
    }
}

impl BotSettingsRef {
    /// See [`BotSettings::populate()`].
    pub fn populate(&self, cx: &mut Cx, bot_settings: &BotSettingsState) {
        let Some(mut inner) = self.borrow_mut() else {
            return;
        };
        inner.populate(cx, bot_settings);
    }
}
