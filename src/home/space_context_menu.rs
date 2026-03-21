//! A context menu that appears when the user right-clicks
//! or long-presses on a space in the spaces bar.

use makepad_widgets::*;
use tokio::sync::mpsc::UnboundedSender;
use crate::{
    join_leave_room_modal::{JoinLeaveModalKind, JoinLeaveRoomModalAction},
    room::BasicRoomDetails,
    shared::popup_list::{PopupKind, enqueue_popup_notification},
    space_service_sync::SpaceRequest,
    utils::RoomNameId,
};

const BUTTON_HEIGHT: f64 = 35.0;
const MENU_WIDTH: f64 = 215.0;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.SPACE_CONTEXT_MENU_BUTTON_HEIGHT = 35
    mod.widgets.SPACE_CONTEXT_MENU_WIDTH = 215

    mod.widgets.SpaceContextMenuButton = RobrixIconButton {
        height: (mod.widgets.SPACE_CONTEXT_MENU_BUTTON_HEIGHT)
        width: Fill,
        margin: 0,
        icon_walk: Walk{width: 16, height: 16, margin: Inset{right: 3}}
        // Override the blue default back to neutral for context menu items
        draw_bg +: { color: (COLOR_PRIMARY), color_hover: #EBEBEB, color_down: #DCDCDC }
        draw_icon.color: #000
        draw_text +: { color: #000, color_hover: #000, color_down: #000 }
    }

    mod.widgets.SpaceContextMenu = set_type_default() do #(SpaceContextMenu::register_widget(vm)) {
        ..mod.widgets.SolidView

        visible: false,
        flow: Overlay,
        width: Fill,
        height: Fill,
        cursor: MouseCursor.Default,
        align: Align{x: 0, y: 0}

        show_bg: true
        draw_bg +: {
            color: #0000004d
        }

        main_content := RoundedView {
            flow: Down
            width: (mod.widgets.SPACE_CONTEXT_MENU_WIDTH),
            height: Fit,
            padding: 5
            spacing: 0,
            align: Align{x: 0, y: 0}

            show_bg: true
            draw_bg +: {
                color: (COLOR_PRIMARY)
                border_radius: 5.0
                border_size: 0.5
                border_color: #888
            }

            copy_link_button := mod.widgets.SpaceContextMenuButton {
                draw_icon +: { svg: (ICON_LINK) }
                text: "Copy Link to Space"
            }

            space_settings_button := mod.widgets.SpaceContextMenuButton {
                draw_icon +: { svg: (ICON_SETTINGS) }
                text: "Space Settings"
            }

            divider1 := LineH {
                margin: Inset{top: 3, bottom: 3}
                width: Fill,
            }

            leave_button := RobrixNegativeIconButton {
                height: (mod.widgets.SPACE_CONTEXT_MENU_BUTTON_HEIGHT)
                width: Fill,
                margin: 0,
                icon_walk: Walk{width: 16, height: 16, margin: Inset{right: 3}}
                draw_icon.svg: (ICON_LOGOUT)
                text: "Leave Space"
            }
        }
    }
}

/// Details needed to populate the space context menu.
#[derive(Clone, Debug)]
pub struct SpaceContextMenuDetails {
    pub space_name_id: RoomNameId,
    pub space_request_sender: UnboundedSender<SpaceRequest>,
}

/// Actions emitted from the SpaceContextMenu widget.
#[derive(Clone, Default, Debug)]
pub enum SpaceContextMenuAction {
    OpenSpaceSettings(RoomNameId),
    #[default]
    None,
}

#[derive(Script, ScriptHook, Widget)]
pub struct SpaceContextMenu {
    #[deref] view: View,
    #[source] source: ScriptObjectRef,
    #[rust] details: Option<SpaceContextMenuDetails>,
}

impl Widget for SpaceContextMenu {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if self.details.is_none() {
            self.visible = false;
        };
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if !self.visible { return; }
        self.view.handle_event(cx, event, scope);

        // Close logic similar to RoomContextMenu
        let area = self.view.area();
        let close_menu = {
            event.back_pressed()
            || match event.hits_with_capture_overload(cx, area, true) {
                Hit::KeyUp(key) => key.key_code == KeyCode::Escape,
                Hit::FingerUp(fue) if fue.is_over => {
                     !self.view(cx, ids!(main_content)).area().rect(cx).contains(fue.abs)
                }
                 Hit::FingerScroll(_) => true,
                _ => false,
            }
        };

        if close_menu {
            self.close(cx);
            return;
        }

        self.widget_match_event(cx, event, scope);
    }
}

impl WidgetMatchEvent for SpaceContextMenu {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        let Some(details) = self.details.as_ref() else { return };
        let mut close_menu = false;

        if self.button(cx, ids!(copy_link_button)).clicked(actions) {
            use crate::sliding_sync::{MatrixRequest, submit_async_request};
            submit_async_request(MatrixRequest::GenerateMatrixLink {
                room_id: details.space_name_id.room_id().clone(),
                event_id: None,
                use_matrix_scheme: false,
                join_on_click: false,
            });
            close_menu = true;
        }
        else if self.button(cx, ids!(space_settings_button)).clicked(actions) {
            enqueue_popup_notification(
                "The space settings page is not yet implemented.",
                PopupKind::Warning,
                Some(5.0),
            );
            close_menu = true;
        }
        else if self.button(cx, ids!(leave_button)).clicked(actions) {
            let room_details = BasicRoomDetails::Name(details.space_name_id.clone());
            cx.action(JoinLeaveRoomModalAction::Open {
                kind: JoinLeaveModalKind::LeaveSpace {
                    details: room_details,
                    space_request_sender: details.space_request_sender.clone(),
                },
                show_tip: false,
            });
            close_menu = true;
        }

        if close_menu {
            self.close(cx);
        }
    }
}

impl SpaceContextMenu {
    pub fn is_currently_shown(&self, _cx: &mut Cx) -> bool {
        self.visible
    }

    pub fn show(&mut self, cx: &mut Cx, details: SpaceContextMenuDetails) -> DVec2 {
        self.reset_button_states(cx);
        self.details = Some(details);
        self.visible = true;
        cx.set_key_focus(self.view.area());
        // Calculate height: 3 buttons * 35.0 + 1 divider * ~10.0 + padding
        let height = (3.0 * BUTTON_HEIGHT) + 10.0 + 10.0;
        dvec2(MENU_WIDTH, height)
    }

    fn reset_button_states(&mut self, cx: &mut Cx) {
        self.button(cx, ids!(copy_link_button)).reset_hover(cx);
        self.button(cx, ids!(space_settings_button)).reset_hover(cx);
        self.button(cx, ids!(leave_button)).reset_hover(cx);
        self.redraw(cx);
    }

    fn close(&mut self, cx: &mut Cx) {
        self.visible = false;
        self.details = None;
        cx.revert_key_focus();
        self.redraw(cx);
    }
}

impl SpaceContextMenuRef {
    pub fn is_currently_shown(&self, cx: &mut Cx) -> bool {
        let Some(inner) = self.borrow() else { return false };
        inner.is_currently_shown(cx)
    }

    pub fn show(&self, cx: &mut Cx, details: SpaceContextMenuDetails) -> DVec2 {
        let Some(mut inner) = self.borrow_mut() else { return DVec2::default()};
        inner.show(cx, details)
    }
}
