//! A view that displays when the user is no longer a member of a room.
//!
//! This is shown when the user has left, been kicked from, or banned from a room.
//! It provides a clear indication of the membership state and optionally allows rejoining.

use makepad_widgets::*;
use makepad_widgets::ActionDefaultRef;
use matrix_sdk::RoomState;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ICON_LEAVE = crate_resource("self://resources/icons/right_from_bracket.svg")
    mod.widgets.ICON_FORBIDDEN = crate_resource("self://resources/icons/forbidden.svg")
    mod.widgets.ICON_ENTER = crate_resource("self://resources/icons/right_to_bracket.svg")

    // A view shown when the user is no longer a member of a room.
    mod.widgets.NoLongerMemberView = #(NoLongerMemberView::register_widget(vm)) {
        width: Fill, height: Fill,
        flow: Overlay,
        align: Align{x: 0.5, y: 0.5}
        visible: false

        show_bg: true
        draw_bg.color: #00000080

        content := RoundedView {
            width: 400, height: Fit,
            flow: Down,
            padding: 30,
            spacing: 15,
            align: Align{x: 0.5, y: 0.5}

            show_bg: true
            draw_bg +: {
                color: (COLOR_PRIMARY)
                border_radius: 8.0
            }

            icon_view := View {
                width: Fill, height: Fit,
                align: Align{x: 0.5, y: 0.5}

                state_icon := Icon {
                    draw_icon +: {
                        svg_file: (mod.widgets.ICON_LEAVE),
                        color: (COLOR_TEXT),
                    }
                    icon_walk: Walk{width: 48, height: 48}
                }
            }

            title := Label {
                width: Fill, height: Fit,
                align: Align{x: 0.5, y: 0.5}
                draw_text +: {
                    text_style: TITLE_TEXT{font_size: 16},
                    color: (COLOR_TEXT)
                    flow: Flow.Right{wrap: true}
                }
                text: "You are no longer a member of this room"
            }

            description := Label {
                width: Fill, height: Fit,
                align: Align{x: 0.5, y: 0.5}
                draw_text +: {
                    text_style: REGULAR_TEXT{font_size: 12},
                    color: #666666
                    flow: Flow.Right{wrap: true}
                }
                text: ""
            }

            reason_view := View {
                width: Fill, height: Fit,
                visible: false
                padding: 10,
                margin: Inset{top: 5}

                show_bg: true
                draw_bg.color: #f5f5f5
                draw_bg.border_radius: 4.0

                reason_label := Label {
                    width: Fill, height: Fit,
                    draw_text +: {
                        text_style: REGULAR_TEXT{font_size: 11},
                        color: #888888
                        flow: Flow.Right{wrap: true}
                    }
                    text: ""
                }
            }

            buttons_view := View {
                width: Fill, height: Fit,
                flow: Right,
                align: Align{x: 0.5, y: 0.5}
                spacing: 15
                padding: Inset{top: 10}

                rejoin_button := RobrixIconButton {
                    visible: false
                    width: 140,
                    align: Align{x: 0.5, y: 0.5}
                    padding: 12,
                    draw_icon +: {
                        svg_file: (mod.widgets.ICON_ENTER)
                        color: (COLOR_FG_ACCEPT_GREEN),
                    }
                    icon_walk: Walk{width: 16, height: 16, margin: Inset{left: -2, right: 2}}

                    draw_bg +: {
                        border_color: (COLOR_FG_ACCEPT_GREEN),
                        color: (COLOR_BG_ACCEPT_GREEN)
                    }
                    draw_text +: {
                        color: (COLOR_FG_ACCEPT_GREEN),
                    }
                    text: "Rejoin Room"
                }

                close_button := RobrixIconButton {
                    width: 100,
                    align: Align{x: 0.5, y: 0.5}
                    padding: 12,
                    icon_walk: Walk{width: 0, height: 0}

                    draw_bg +: {
                        border_size: 0.75
                        border_color: (COLOR_BG_DISABLED),
                        color: (COLOR_SECONDARY)
                    }
                    draw_text +: {
                        color: (COLOR_TEXT),
                    }
                    text: "Close"
                }
            }
        }
    }
}

/// The reason why the user is no longer a member of the room.
#[derive(Clone, Debug, Default)]
pub enum NoLongerMemberReason {
    /// The user voluntarily left the room.
    #[default]
    Left,
    /// The user was kicked from the room, optionally with a reason.
    Kicked(Option<String>),
    /// The user was banned from the room, optionally with a reason.
    Banned(Option<String>),
}

impl NoLongerMemberReason {
    /// Returns `true` if the user can attempt to rejoin the room.
    pub fn can_rejoin(&self) -> bool {
        // Users who were banned cannot rejoin
        !matches!(self, NoLongerMemberReason::Banned(_))
    }

    /// Returns the title text for this reason.
    pub fn title(&self) -> &'static str {
        match self {
            NoLongerMemberReason::Left => "You left this room",
            NoLongerMemberReason::Kicked(_) => "You were removed from this room",
            NoLongerMemberReason::Banned(_) => "You are banned from this room",
        }
    }

    /// Returns the description text for this reason.
    pub fn description(&self) -> &'static str {
        match self {
            NoLongerMemberReason::Left => {
                "You are no longer a member of this room. If this is a public room, you can rejoin at any time."
            }
            NoLongerMemberReason::Kicked(_) => {
                "A room administrator removed you from this room. You may be able to rejoin if the room is public."
            }
            NoLongerMemberReason::Banned(_) => {
                "A room administrator has banned you from this room. You cannot rejoin until you are unbanned."
            }
        }
    }

    /// Returns the optional reason string if one was provided.
    pub fn reason_text(&self) -> Option<&str> {
        match self {
            NoLongerMemberReason::Left => None,
            NoLongerMemberReason::Kicked(reason) | NoLongerMemberReason::Banned(reason) => {
                reason.as_deref()
            }
        }
    }

    /// Creates from a RoomState.
    pub fn from_room_state(state: RoomState) -> Option<Self> {
        match state {
            RoomState::Left => Some(NoLongerMemberReason::Left),
            RoomState::Banned => Some(NoLongerMemberReason::Banned(None)),
            _ => None,
        }
    }
}

/// Actions emitted by the NoLongerMemberView widget.
#[derive(Clone, Debug, Default)]
pub enum NoLongerMemberAction {
    /// The user clicked the rejoin button.
    Rejoin,
    /// The user clicked the close button.
    Close,
    #[default]
    None,
}

impl ActionDefaultRef for NoLongerMemberAction {
    fn default_ref() -> &'static Self {
        static DEFAULT: NoLongerMemberAction = NoLongerMemberAction::None;
        &DEFAULT
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct NoLongerMemberView {
    #[deref] view: View,
    #[live(false)] visible: bool,
    #[rust] reason: NoLongerMemberReason,
}

impl Widget for NoLongerMemberView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.visible {
            self.view.draw_walk(cx, scope, walk)
        } else {
            DrawStep::done()
        }
    }
}

impl WidgetMatchEvent for NoLongerMemberView {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let rejoin_button = self.button(cx, ids!(rejoin_button));
        let close_button = self.button(cx, ids!(close_button));

        if rejoin_button.clicked(actions) {
            cx.widget_action(
                self.widget_uid(),
                NoLongerMemberAction::Rejoin,
            );
        }

        if close_button.clicked(actions) {
            cx.widget_action(
                self.widget_uid(),
                NoLongerMemberAction::Close,
            );
        }
    }
}

impl NoLongerMemberView {
    /// Shows this view with the specified reason for why the user is no longer a member.
    pub fn show(&mut self, cx: &mut Cx, reason: NoLongerMemberReason) {
        self.reason = reason.clone();

        // Update the title
        self.label(cx, ids!(title)).set_text(cx, reason.title());

        // Update the description
        self.label(cx, ids!(description)).set_text(cx, reason.description());

        // Show/hide the reason view if there's a specific reason
        if let Some(reason_text) = reason.reason_text() {
            self.view(cx, ids!(reason_view)).set_visible(cx, true);
            self.label(cx, ids!(reason_label)).set_text(cx, &format!("Reason: {}", reason_text));
        } else {
            self.view(cx, ids!(reason_view)).set_visible(cx, false);
        }

        // Show/hide the rejoin button based on whether rejoin is possible
        self.button(cx, ids!(rejoin_button)).set_visible(cx, reason.can_rejoin());

        self.visible = true;
        self.redraw(cx);
    }

    /// Hides this view.
    pub fn hide(&mut self, cx: &mut Cx) {
        self.visible = false;
        self.redraw(cx);
    }
}

impl NoLongerMemberViewRef {
    /// See [`NoLongerMemberView::show()`].
    pub fn show(&self, cx: &mut Cx, reason: NoLongerMemberReason) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show(cx, reason);
        }
    }

    /// See [`NoLongerMemberView::hide()`].
    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }

    /// Returns the action if the rejoin button was clicked.
    pub fn rejoin_clicked(&self, actions: &Actions) -> bool {
        matches!(
            actions.find_widget_action(self.widget_uid()).cast_ref(),
            NoLongerMemberAction::Rejoin
        )
    }

    /// Returns the action if the close button was clicked.
    pub fn close_clicked(&self, actions: &Actions) -> bool {
        matches!(
            actions.find_widget_action(self.widget_uid()).cast_ref(),
            NoLongerMemberAction::Close
        )
    }
}
