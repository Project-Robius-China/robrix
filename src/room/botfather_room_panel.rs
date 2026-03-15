use std::cell::RefCell;

use makepad_widgets::*;
use matrix_sdk::room::reply::{EnforceThread, Reply};
use ruma::events::room::message::{ReplyWithinThread, RoomMessageEventContent};

use crate::{
    app::PositiveConfirmationModalAction,
    botfather::{self, BotfatherAction},
    home::room_screen::RoomScreenProps,
    login::login_screen::LoginAction,
    logout::logout_confirm_modal::LogoutAction,
    shared::confirmation_modal::ConfirmationModalContent,
    sliding_sync::{MatrixRequest, spawn_on_tokio, submit_async_request},
};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::helpers::*;
    use crate::shared::styles::*;
    use crate::shared::icon_button::*;

    BotSelectorDropDown = <DropDown> {
        width: Fill
        height: Fit
        popup_menu_position: BelowInput
        labels: ["No bots configured"]
        draw_bg: {
            border_radius: 6.0
            color: (COLOR_PRIMARY)
            color_hover: (COLOR_BG_PREVIEW)
            color_focus: (COLOR_BG_PREVIEW)
            color_down: (COLOR_BG_PREVIEW)
            border_color: (COLOR_SECONDARY)
            border_color_hover: (COLOR_SECONDARY)
            border_color_focus: (COLOR_ACTIVE_PRIMARY)
            border_color_down: (COLOR_ACTIVE_PRIMARY)
            border_color_2: (COLOR_SECONDARY)
            border_color_2_hover: (COLOR_SECONDARY)
            border_color_2_focus: (COLOR_ACTIVE_PRIMARY)
            border_color_2_down: (COLOR_ACTIVE_PRIMARY)
        }
        draw_text: {
            color: (COLOR_TEXT)
        }
    }

    pub BotfatherRoomPanel = {{BotfatherRoomPanel}} {
        width: Fill, height: Fit
        flow: Down
        spacing: 10
        margin: {left: 6, right: 6, top: 6, bottom: 16}
        padding: 12

        header = <View> {
            width: Fill, height: Fit
            flow: Right
            align: {x: 1.0, y: 0.5}
            spacing: 10

            <Label> {
                width: Fill, height: Fit
                draw_text: {
                    text_style: <REGULAR_TEXT>{font_size: 11.0}
                    color: (COLOR_TEXT)
                }
                text: "Bot Room Panel"
            }

            close_button = <RobrixIconButton> {
                width: Fit
                padding: 9
                spacing: 0
                draw_bg: {
                    color: (COLOR_SECONDARY)
                }
                draw_icon: {
                    svg_file: (ICON_CLOSE)
                    color: (COLOR_TEXT)
                }
                icon_walk: { width: 12, height: 12 }
                text: ""
            }
        }

        show_bg: true
        draw_bg: {
            color: (COLOR_BG_PREVIEW)
            border_radius: 4.0
            border_size: 1.0
            border_color: (COLOR_SECONDARY)
        }

        <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.5}
                color: (COLOR_TEXT)
            }
            text: "This panel manages the room's active bot and sender binding. Pick the room bot and sender below, then click Bind Room to apply them. If both runtimes are available and you have not overridden the room, Crew still wins by default."
        }

        binding_summary_label = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.5}
                color: (COLOR_TEXT)
            }
            text: "No bot binding resolved yet."
        }

        <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            <SubsectionLabel> {
                text: "Room Bot"
            }

            bot_selector_dropdown = <BotSelectorDropDown> {}
        }

        <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            <SubsectionLabel> {
                text: "Room Sender"
            }

            sender_selector_dropdown = <BotSelectorDropDown> {
                labels: ["Current User"]
            }
        }

        <View> {
            width: Fill, height: Fit
            flow: RightWrap
            spacing: 10

            bind_room_button = <RobrixIconButton> {
                width: Fit
                padding: 12
                draw_bg: {
                    color: (COLOR_BG_ACCEPT_GREEN)
                    border_color: (COLOR_FG_ACCEPT_GREEN)
                }
                draw_icon: {
                    svg_file: (ICON_LINK)
                    color: (COLOR_FG_ACCEPT_GREEN)
                }
                draw_text: {
                    color: (COLOR_FG_ACCEPT_GREEN)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Bind Room"
            }

            unbind_room_button = <RobrixIconButton> {
                width: Fit
                padding: 12
                draw_bg: {
                    color: (COLOR_BG_DANGER_RED)
                    border_color: (COLOR_FG_DANGER_RED)
                }
                draw_icon: {
                    svg_file: (ICON_FORBIDDEN)
                    color: (COLOR_FG_DANGER_RED)
                }
                draw_text: {
                    color: (COLOR_FG_DANGER_RED)
                }
                icon_walk: { width: 14, height: 14 }
                text: "Unbind Room"
            }

        }

        preview_card = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8
            padding: 10
            show_bg: true
            draw_bg: {
                color: (COLOR_PRIMARY)
                border_radius: 4.0
                border_size: 1.0
                border_color: (COLOR_SECONDARY)
            }

            <SubsectionLabel> {
                text: "Latest Bot Stream"
            }

            preview_status_label = <Label> {
                width: Fill, height: Fit
                flow: RightWrap
                draw_text: {
                    wrap: Word
                    text_style: <REGULAR_TEXT>{font_size: 10.5}
                    color: (COLOR_TEXT)
                }
                text: "No streamed preview yet."
            }

            preview_body_label = <Label> {
                width: Fill, height: Fit
                flow: RightWrap
                draw_text: {
                    wrap: Word
                    text_style: <REGULAR_TEXT>{font_size: 10.5}
                    color: (COLOR_TEXT)
                }
                text: ""
            }

            <View> {
                width: Fill, height: Fit
                flow: RightWrap
                spacing: 10

                post_preview_button = <RobrixIconButton> {
                    width: Fit
                    padding: 12
                    enabled: false
                    draw_bg: {
                        color: (COLOR_ACTIVE_PRIMARY)
                    }
                    draw_icon: {
                        svg_file: (ICON_SEND)
                        color: (COLOR_PRIMARY)
                    }
                    draw_text: {
                        color: (COLOR_PRIMARY)
                    }
                    icon_walk: { width: 14, height: 14 }
                    text: "Post Preview"
                }

                clear_preview_button = <RobrixIconButton> {
                    width: Fit
                    padding: 12
                    enabled: false
                    draw_bg: {
                        color: (COLOR_SECONDARY)
                    }
                    draw_icon: {
                        svg_file: (ICON_CLOSE)
                        color: (COLOR_TEXT)
                    }
                    icon_walk: { width: 12, height: 12 }
                    text: "Clear Preview"
                }
            }
        }

        status_label = <Label> {
            width: Fill, height: Fit
            flow: RightWrap
            draw_text: {
                wrap: Word
                text_style: <REGULAR_TEXT>{font_size: 10.5}
                color: (COLOR_TEXT)
            }
            text: ""
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct BotfatherRoomPanel {
    #[deref]
    view: View,
    #[rust]
    bot_choice_ids: Vec<String>,
    #[rust]
    selected_bot_id: Option<String>,
    #[rust]
    sender_choice_ids: Vec<String>,
    #[rust]
    selected_sender_profile_id: Option<String>,
    #[rust]
    rendered_room_id: Option<String>,
}

#[derive(Clone, Debug, DefaultNone)]
pub enum BotfatherRoomPanelAction {
    CloseRequested,
    None,
}

impl Widget for BotfatherRoomPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if matches!(event, Event::Signal) {
            let _ = botfather::ensure_loaded_for_current_user();
            let _ = botfather::refresh_inventory_from_rooms_list(cx);
            self.apply_preview_visibility(cx);
            let current_room_id = current_room_id(scope);
            if self.rendered_room_id != current_room_id {
                self.refresh_room_state(cx, current_room_id.clone());
                self.rendered_room_id = current_room_id;
            }
            self.refresh_stream_preview(cx, scope);
        }

        if let Event::Actions(actions) = event {
            let current_room_id = current_room_id(scope);
            let bot_selector_dropdown = self.drop_down(ids!(bot_selector_dropdown));
            let sender_selector_dropdown = self.drop_down(ids!(sender_selector_dropdown));
            let bind_room_button = self.view.button(ids!(bind_room_button));
            let unbind_room_button = self.view.button(ids!(unbind_room_button));
            let post_preview_button = self.view.button(ids!(post_preview_button));
            let clear_preview_button = self.view.button(ids!(clear_preview_button));
            let close_button = self.view.button(ids!(close_button));

            if let Some(selected_index) = bot_selector_dropdown.selected(actions) {
                if let Some(bot_id) = self.bot_choice_ids.get(selected_index).cloned() {
                    self.selected_bot_id = Some(bot_id.clone());
                    self.set_status(
                        cx,
                        &format!("Selected \"{bot_id}\". Click Bind Room to apply it."),
                    );
                } else if self.bot_choice_ids.is_empty() {
                    self.set_status(
                        cx,
                        "Configure at least one bot in BotFather Settings first.",
                    );
                }
            }

            if let Some(selected_index) = sender_selector_dropdown.selected(actions) {
                if let Some(sender_profile_id) = self.sender_choice_ids.get(selected_index).cloned()
                {
                    self.selected_sender_profile_id = Some(sender_profile_id.clone());
                    self.set_status(
                        cx,
                        &format!(
                            "Selected sender \"{sender_profile_id}\". Click Bind Room to apply it."
                        ),
                    );
                }
            }

            if bind_room_button.clicked(actions) {
                match (
                    current_room_id.as_deref(),
                    self.selected_bot_id.as_deref(),
                    self.selected_sender_profile_id.as_deref(),
                ) {
                    (Some(room_id), Some(bot_id), sender_profile_id) => {
                        let selected_sender =
                            sender_profile_id.and_then(botfather::sender_profile_option);
                        let switching_to_shared = botfather::room_is_personal_assist(room_id)
                            && selected_sender.as_ref().is_some_and(|option| {
                                !matches!(
                                    option.kind,
                                    robrix_botfather::SenderProfileKind::CurrentUser
                                )
                            });

                        if let Some(selected_sender_id) = sender_profile_id {
                            if !botfather::sender_profile_is_ready(selected_sender_id) {
                                let guidance =
                                    botfather::sender_profile_setup_guidance(selected_sender_id);
                                cx.action(PositiveConfirmationModalAction::Show(RefCell::new(
                                    Some(ConfirmationModalContent {
                                        title_text: "Independent Bot Account Required".into(),
                                        body_text: guidance.clone().into(),
                                        accept_button_text: Some("Understood".into()),
                                        cancel_button_text: Some("Close".into()),
                                        ..Default::default()
                                    }),
                                )));
                                self.set_status(cx, &guidance);
                                return;
                            }
                        }

                        if switching_to_shared {
                            let room_id = room_id.to_string();
                            let bot_id = bot_id.to_string();
                            let sender_profile_id = sender_profile_id.map(ToOwned::to_owned);
                            cx.action(PositiveConfirmationModalAction::Show(RefCell::new(
                                Some(ConfirmationModalContent {
                                    title_text: "Switch Room to Shared Bot Mode".into(),
                                    body_text: "This room is currently using your own Matrix account as a personal assistant. Switching to a shared bot sender separates bot output from your account, makes attribution clear for everyone in the room, and gives higher-security rooms an isolated sender boundary.".into(),
                                    accept_button_text: Some("Switch Room".into()),
                                    cancel_button_text: Some("Keep Personal".into()),
                                    on_accept_clicked: Some(Box::new(move |_cx| {
                                        let status = bind_room_and_emit_status(
                                            &room_id,
                                            &bot_id,
                                            sender_profile_id.as_deref(),
                                        );
                                        Cx::post_action(BotfatherAction::Status(status));
                                    })),
                                    ..Default::default()
                                }),
                            )));
                            self.set_status(
                                cx,
                                "Confirm the sender switch to move this room from personal assist to shared bot mode.",
                            );
                            return;
                        }

                        let message = bind_room_and_emit_status(room_id, bot_id, sender_profile_id);
                        self.refresh_room_state(cx, current_room_id.clone());
                        self.sync_selected_bot(cx, current_room_id.as_deref());
                        self.sync_selected_sender(cx, current_room_id.as_deref());
                        self.rendered_room_id = current_room_id.clone();
                        self.refresh_stream_preview(cx, scope);
                        self.set_status(cx, &message);
                    }
                    (Some(_), None, _) => self.set_status(
                        cx,
                        "Pick a bot from the Room Bot dropdown before binding the room.",
                    ),
                    (None, _, _) => {
                        self.set_status(cx, "This room is not ready for BotFather yet.")
                    }
                }
            }

            if unbind_room_button.clicked(actions) {
                match current_room_id.as_deref() {
                    Some(room_id) => match botfather::unbind_room(room_id) {
                        Ok(()) => {
                            self.refresh_room_state(cx, current_room_id.clone());
                            self.sync_selected_bot(cx, current_room_id.as_deref());
                            self.sync_selected_sender(cx, current_room_id.as_deref());
                            self.rendered_room_id = current_room_id.clone();
                            self.refresh_stream_preview(cx, scope);
                            self.set_status(cx, "Removed the room-level bot override.");
                        }
                        Err(error) => self.set_status(cx, &error),
                    },
                    None => self.set_status(cx, "This room is not ready for BotFather yet."),
                }
            }

            if post_preview_button.clicked(actions) {
                match scope.props.get::<RoomScreenProps>() {
                    Some(room_props) => {
                        let thread_root_event_id = room_props
                            .timeline_kind
                            .thread_root_event_id()
                            .map(|event_id| event_id.to_string());
                        match botfather::room_stream_preview(
                            room_props.room_name_id.room_id().as_str(),
                            thread_root_event_id.as_deref(),
                        )
                        .map(|preview| preview.text)
                        .filter(|text| !text.trim().is_empty())
                        {
                            Some(text) => {
                                if botfather::room_uses_current_user_sender(
                                    room_props.room_name_id.room_id().as_str(),
                                ) {
                                    let replied_to = room_props
                                        .timeline_kind
                                        .thread_root_event_id()
                                        .map(|thread_root_event_id| Reply {
                                            event_id: thread_root_event_id.clone(),
                                            enforce_thread: EnforceThread::Threaded(
                                                ReplyWithinThread::No,
                                            ),
                                        });
                                    submit_async_request(MatrixRequest::SendMessage {
                                        timeline_kind: room_props.timeline_kind.clone(),
                                        message: RoomMessageEventContent::text_markdown(text),
                                        replied_to,
                                        bot_prompt: None,
                                        bot_dispatch_context: None,
                                        bot_stream_placeholder_context: None,
                                        #[cfg(feature = "tsp")]
                                        sign_with_tsp: false,
                                    });
                                    botfather::clear_room_stream_preview(
                                        room_props.room_name_id.room_id().as_str(),
                                        thread_root_event_id.as_deref(),
                                    );
                                    self.refresh_stream_preview(cx, scope);
                                    self.set_status(cx, "Posted the bot preview to Matrix.");
                                } else {
                                    let room_id = room_props.room_name_id.room_id().to_string();
                                    let thread_root_event_id = thread_root_event_id.clone();
                                    self.set_status(
                                        cx,
                                        "Posting bot preview via the resolved sender...",
                                    );
                                    spawn_on_tokio(async move {
                                        let status =
                                            match botfather::post_markdown_via_resolved_sender(
                                                &room_id,
                                                thread_root_event_id.as_deref(),
                                                text,
                                            )
                                            .await
                                            {
                                                Ok(message) => {
                                                    botfather::clear_room_stream_preview(
                                                        &room_id,
                                                        thread_root_event_id.as_deref(),
                                                    );
                                                    message
                                                }
                                                Err(error) => error,
                                            };
                                        Cx::post_action(BotfatherAction::Status(status));
                                    });
                                }
                            }
                            None => self.set_status(
                                cx,
                                "No finished bot preview is available for this scope yet.",
                            ),
                        }
                    }
                    None => self.set_status(cx, "This room is not ready for BotFather yet."),
                }
            }

            if clear_preview_button.clicked(actions) {
                match scope.props.get::<RoomScreenProps>() {
                    Some(room_props) => {
                        let thread_root_event_id = room_props
                            .timeline_kind
                            .thread_root_event_id()
                            .map(|event_id| event_id.to_string());
                        botfather::clear_room_stream_preview(
                            room_props.room_name_id.room_id().as_str(),
                            thread_root_event_id.as_deref(),
                        );
                        self.refresh_stream_preview(cx, scope);
                        self.set_status(cx, "Cleared the current bot preview buffer.");
                    }
                    None => self.set_status(cx, "This room is not ready for BotFather yet."),
                }
            }

            if close_button.clicked(actions) {
                cx.widget_action(
                    self.widget_uid(),
                    &scope.path,
                    BotfatherRoomPanelAction::CloseRequested,
                );
            }

            for action in actions {
                if let Some(LoginAction::LoginSuccess) = action.downcast_ref() {
                    let _ = botfather::ensure_loaded_for_current_user();
                    self.apply_preview_visibility(cx);
                    self.refresh_room_state(cx, current_room_id.clone());
                    self.refresh_stream_preview(cx, scope);
                    continue;
                }

                if let Some(LogoutAction::ClearAppState { .. }) = action.downcast_ref() {
                    self.clear(cx);
                    continue;
                }

                if let Some(BotfatherAction::StateChanged) = action.downcast_ref() {
                    self.apply_preview_visibility(cx);
                    self.refresh_room_state(cx, current_room_id.clone());
                    self.sync_selected_bot(cx, current_room_id.as_deref());
                    self.sync_selected_sender(cx, current_room_id.as_deref());
                    self.rendered_room_id = current_room_id.clone();
                    self.refresh_stream_preview(cx, scope);
                    continue;
                }

                if let Some(
                    BotfatherAction::StreamQueued { room_id, .. }
                    | BotfatherAction::StreamStarted { room_id, .. }
                    | BotfatherAction::StreamDelta { room_id, .. }
                    | BotfatherAction::StreamFinished { room_id, .. }
                    | BotfatherAction::StreamFailed { room_id, .. }
                    | BotfatherAction::StreamCancelled { room_id, .. },
                ) = action.downcast_ref()
                {
                    if current_room_id.as_deref() == Some(room_id.as_str()) {
                        self.refresh_stream_preview(cx, scope);
                    }
                    continue;
                }

                if let Some(BotfatherAction::Status(status)) = action.downcast_ref() {
                    self.set_status(cx, status);
                    continue;
                }

                if let Some(BotfatherAction::HealthcheckFinished { room_id, result }) =
                    action.downcast_ref()
                {
                    if current_room_id.as_deref() == Some(room_id.as_str()) {
                        match result {
                            Ok(message) => self.set_status(cx, message),
                            Err(error) => self.set_status(cx, error),
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl BotfatherRoomPanel {
    fn refresh_room_state(&mut self, cx: &mut Cx, room_id: Option<String>) {
        match room_id {
            Some(room_id) => {
                self.view
                    .label(ids!(binding_summary_label))
                    .set_text(cx, &botfather::describe_room_binding(&room_id));
                self.update_bot_selector(cx, Some(room_id.as_str()));
                self.update_sender_selector(cx, Some(room_id.as_str()));
            }
            None => {
                self.view.label(ids!(binding_summary_label)).set_text(
                    cx,
                    "This room is not ready yet. Open it again after Robrix finishes loading.",
                );
                self.update_bot_selector(cx, None);
                self.update_sender_selector(cx, None);
                self.clear_preview(cx);
            }
        }
    }

    fn clear(&mut self, cx: &mut Cx) {
        self.view
            .label(ids!(binding_summary_label))
            .set_text(cx, "No bot binding resolved yet.");
        self.view.label(ids!(status_label)).set_text(cx, "");
        self.selected_bot_id = None;
        self.selected_sender_profile_id = None;
        self.rendered_room_id = None;
        self.update_bot_selector(cx, None);
        self.update_sender_selector(cx, None);
        self.clear_preview(cx);
    }

    fn set_status(&mut self, cx: &mut Cx, status: &str) {
        self.view.label(ids!(status_label)).set_text(cx, status);
    }

    fn update_bot_selector(&mut self, cx: &mut Cx, room_id: Option<&str>) {
        let dropdown = self.drop_down(ids!(bot_selector_dropdown));
        let options = match room_id {
            Some(room_id) => botfather::room_bot_options(Some(room_id)),
            None => Vec::new(),
        };

        if options.is_empty() {
            self.bot_choice_ids.clear();
            self.selected_bot_id = None;
            let placeholder = if room_id.is_some() {
                "No bots configured"
            } else {
                "Room unavailable"
            };
            dropdown.set_labels(cx, vec![placeholder.to_string()]);
            dropdown.set_selected_item(cx, 0);
            return;
        }

        self.bot_choice_ids = options.iter().map(|option| option.bot_id.clone()).collect();
        dropdown.set_labels(
            cx,
            options
                .iter()
                .map(|option| option.label.clone())
                .collect::<Vec<_>>(),
        );

        let selected_index = room_id
            .and_then(botfather::room_primary_bot_id)
            .and_then(|bot_id| {
                self.bot_choice_ids
                    .iter()
                    .position(|candidate| candidate == &bot_id)
            })
            .unwrap_or(0);
        self.selected_bot_id = self.bot_choice_ids.get(selected_index).cloned();
        dropdown.set_selected_item(cx, selected_index);
    }

    fn update_sender_selector(&mut self, cx: &mut Cx, room_id: Option<&str>) {
        let dropdown = self.drop_down(ids!(sender_selector_dropdown));
        let options = botfather::sender_profile_options();

        if options.is_empty() {
            self.sender_choice_ids.clear();
            self.selected_sender_profile_id = None;
            dropdown.set_labels(cx, vec!["No senders configured".to_string()]);
            dropdown.set_selected_item(cx, 0);
            return;
        }

        self.sender_choice_ids = options
            .iter()
            .map(|option| option.sender_profile_id.clone())
            .collect();
        dropdown.set_labels(
            cx,
            options
                .iter()
                .map(|option| option.label.clone())
                .collect::<Vec<_>>(),
        );

        let selected_index = room_id
            .and_then(botfather::room_primary_sender_profile_id)
            .and_then(|sender_profile_id| {
                self.sender_choice_ids
                    .iter()
                    .position(|candidate| candidate == &sender_profile_id)
            })
            .unwrap_or(0);
        self.selected_sender_profile_id = self.sender_choice_ids.get(selected_index).cloned();
        dropdown.set_selected_item(cx, selected_index);
    }

    fn sync_selected_bot(&mut self, cx: &mut Cx, room_id: Option<&str>) {
        let Some(room_id) = room_id else {
            return;
        };
        let Some(bot_id) = botfather::room_primary_bot_id(room_id) else {
            self.selected_bot_id = self.bot_choice_ids.first().cloned();
            self.drop_down(ids!(bot_selector_dropdown))
                .set_selected_item(cx, 0);
            return;
        };
        if let Some(selected_index) = self
            .bot_choice_ids
            .iter()
            .position(|candidate| candidate == &bot_id)
        {
            self.selected_bot_id = Some(bot_id);
            self.drop_down(ids!(bot_selector_dropdown))
                .set_selected_item(cx, selected_index);
        }
    }

    fn sync_selected_sender(&mut self, cx: &mut Cx, room_id: Option<&str>) {
        let Some(room_id) = room_id else {
            return;
        };
        let Some(sender_profile_id) = botfather::room_primary_sender_profile_id(room_id) else {
            self.selected_sender_profile_id = self.sender_choice_ids.first().cloned();
            self.drop_down(ids!(sender_selector_dropdown))
                .set_selected_item(cx, 0);
            return;
        };
        if let Some(selected_index) = self
            .sender_choice_ids
            .iter()
            .position(|candidate| candidate == &sender_profile_id)
        {
            self.selected_sender_profile_id = Some(sender_profile_id);
            self.drop_down(ids!(sender_selector_dropdown))
                .set_selected_item(cx, selected_index);
        }
    }

    fn refresh_stream_preview(&mut self, cx: &mut Cx, scope: &mut Scope) {
        let Some(room_props) = scope.props.get::<RoomScreenProps>() else {
            self.clear_preview(cx);
            return;
        };
        self.refresh_stream_preview_for_room(
            cx,
            room_props.room_name_id.room_id().as_str(),
            room_props
                .timeline_kind
                .thread_root_event_id()
                .map(|event_id| event_id.to_string())
                .as_deref(),
        );
    }

    fn refresh_stream_preview_for_room(
        &mut self,
        cx: &mut Cx,
        room_id: &str,
        thread_root_event_id: Option<&str>,
    ) {
        let post_preview_button = self.view.button(ids!(post_preview_button));
        let clear_preview_button = self.view.button(ids!(clear_preview_button));

        if let Some(preview) = botfather::room_stream_preview(room_id, thread_root_event_id) {
            let status = match preview.status {
                botfather::BotStreamPreviewStatus::Idle => preview.detail,
                botfather::BotStreamPreviewStatus::Queued => format!("Queued. {}", preview.detail),
                botfather::BotStreamPreviewStatus::Streaming => {
                    format!("Streaming {}...", preview.bot_name)
                }
                botfather::BotStreamPreviewStatus::Finished => {
                    format!("Finished. {}", preview.detail)
                }
                botfather::BotStreamPreviewStatus::Failed => format!("Failed. {}", preview.detail),
                botfather::BotStreamPreviewStatus::Cancelled => preview.detail,
            };
            self.view
                .label(ids!(preview_status_label))
                .set_text(cx, &status);
            self.view
                .label(ids!(preview_body_label))
                .set_text(cx, &preview.text);
            post_preview_button.set_enabled(cx, preview.can_post);
            clear_preview_button.set_enabled(
                cx,
                !preview.text.trim().is_empty()
                    || preview.status != botfather::BotStreamPreviewStatus::Idle,
            );
        } else {
            self.clear_preview(cx);
        }
    }

    fn clear_preview(&mut self, cx: &mut Cx) {
        self.view
            .label(ids!(preview_status_label))
            .set_text(cx, "No streamed preview yet.");
        self.view.label(ids!(preview_body_label)).set_text(cx, "");
        self.view
            .button(ids!(post_preview_button))
            .set_enabled(cx, false);
        self.view
            .button(ids!(clear_preview_button))
            .set_enabled(cx, false);
    }

    fn apply_preview_visibility(&mut self, cx: &mut Cx) {
        let visible = botfather::room_stream_preview_enabled();
        self.view.view(ids!(preview_card)).set_visible(cx, visible);
        if !visible {
            self.clear_preview(cx);
        }
    }
}

fn current_room_id(scope: &mut Scope) -> Option<String> {
    scope
        .props
        .get::<RoomScreenProps>()
        .map(|props| props.room_name_id.room_id().to_string())
}

fn bind_room_and_emit_status(
    room_id: &str,
    bot_id: &str,
    sender_profile_id: Option<&str>,
) -> String {
    match botfather::bind_room_to_bot_and_sender(room_id, bot_id, sender_profile_id) {
        Ok(message) => message,
        Err(error) => error,
    }
}
