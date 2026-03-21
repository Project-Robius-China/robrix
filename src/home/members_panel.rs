//! A sliding panel that displays the list of members in a Matrix room.
//!
//! This panel can be opened from the room screen to view, search, and interact
//! with room members.

use std::collections::HashMap;
use std::sync::Arc;
use crossbeam_queue::SegQueue;
use makepad_widgets::*;
use matrix_sdk::room::RoomMember;
use ruma::{OwnedRoomId, OwnedUserId};

use crate::{
    profile::user_profile::{ShowUserProfileAction, UserProfile, UserProfileAndRoomId},
    shared::avatar::{AvatarState, AvatarWidgetRefExt},
    sliding_sync::{submit_async_request, MatrixRequest},
    utils,
};

/// The debounce delay before triggering a background search (in seconds).
const SEARCH_DEBOUNCE_DELAY: f64 = 0.3;

/// The minimum number of members before using async search.
/// For smaller lists, synchronous filtering is fast enough.
const ASYNC_SEARCH_THRESHOLD: usize = 100;

/// Maximum number of cached search results to prevent memory bloat.
const SEARCH_CACHE_LIMIT: usize = 20;

/// Search state for the members panel.
#[derive(Clone, Debug, Default)]
pub enum MemberSearchState {
    /// No search is in progress.
    #[default]
    Idle,
    /// Waiting for debounce timer to expire before searching.
    Debouncing,
    /// A background search is in progress.
    Searching {
        /// The query being searched.
        query: String,
        /// Unique ID to identify this search and discard stale results.
        search_id: u64,
    },
}

/// Result of a background member search operation.
pub struct MemberSearchResult {
    /// The unique ID of this search operation.
    pub search_id: u64,
    /// The room ID this search was for.
    pub room_id: OwnedRoomId,
    /// The search query that was used.
    pub query: String,
    /// Indices into the members list that matched the query.
    pub matched_indices: Vec<usize>,
}

/// The queue of member search results waiting to be processed by the UI thread.
static PENDING_MEMBER_SEARCH_RESULTS: SegQueue<MemberSearchResult> = SegQueue::new();

/// Enqueues a new member search result and signals the UI.
pub fn enqueue_member_search_result(result: MemberSearchResult) {
    PENDING_MEMBER_SEARCH_RESULTS.push(result);
    SignalToUI::set_ui_signal();
}

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.ICON_CLOSE = crate_resource("self://resources/icons/close.svg")

    // A single member entry in the members list - uses a Button base for click handling
    mod.widgets.MemberEntry = Button {
        width: Fill,
        height: Fit,
        flow: Right,
        align: Align{x: 0.0, y: 0.5},
        padding: Inset{left: 10, right: 10, top: 8, bottom: 8},
        spacing: 10,
        cursor: MouseCursor.Hand,

        draw_bg +: {
            fn pixel() -> vec4 {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
        }

        avatar := Avatar {
            width: 36,
            height: 36,
            text_view +: {
                text +: {
                    draw_text +: {
                        text_style: theme.font_regular { font_size: 14.0 }
                    }
                }
            }
        }

        info_column := View {
            width: Fill,
            height: Fit,
            flow: Down,
            spacing: 2,

            display_name := Label {
                width: Fill,
                height: Fit,
                draw_text +: {
                    text_style: USERNAME_TEXT_STYLE { font_size: 11 },
                    color: #000
                }
                text: "Display Name"
            }

            user_id_label := Label {
                width: Fill,
                height: Fit,
                draw_text +: {
                    text_style: MESSAGE_TEXT_STYLE { font_size: 9 },
                    color: (MESSAGE_TEXT_COLOR)
                }
                text: "@user:server.com"
            }
        }

        role_badge := RoundedView {
            visible: false,
            width: Fit,
            height: Fit,
            padding: Inset{left: 6, right: 6, top: 2, bottom: 2},
            show_bg: true,
            draw_bg +: {
                color: #E8E8E8,
                border_radius: 3.0,
            }

            role_label := Label {
                width: Fit,
                height: Fit,
                draw_text +: {
                    text_style: TEXT_SUB { font_size: 8 },
                    color: #666
                }
                text: "Admin"
            }
        }
    }

    mod.widgets.MembersPanel = #(MembersPanel::register_widget(vm)) {
        visible: false,
        flow: Overlay,
        width: Fill,
        height: Fill,
        align: Align{x: 1.0, y: 0}

        // Semi-transparent background overlay
        bg_view := SolidView {
            width: Fill
            height: Fill
            visible: false,
            show_bg: true
            draw_bg.color: #000000BB
        }

        // Main panel content
        main_content := SolidView {
            width: 300,
            height: Fill
            flow: Down,
            align: Align{x: 1.0}

            show_bg: true,
            draw_bg.color: (COLOR_PRIMARY)

            // Header with title and close button
            header := View {
                width: Fill,
                height: Fit,
                flow: Right,
                align: Align{x: 0.0, y: 0.5},
                padding: Inset{left: 15, right: 10, top: 10, bottom: 10},

                title := Label {
                    width: Fill,
                    height: Fit,
                    draw_text +: {
                        text_style: USERNAME_TEXT_STYLE { font_size: 13 },
                        color: #000
                    }
                    text: "Members"
                }

                member_count := Label {
                    width: Fit,
                    height: Fit,
                    margin: Inset{right: 10},
                    draw_text +: {
                        text_style: MESSAGE_TEXT_STYLE { font_size: 10 },
                        color: (MESSAGE_TEXT_COLOR)
                    }
                    text: "(0)"
                }

                close_button := RobrixNeutralIconButton {
                    width: Fit,
                    height: Fit,
                    spacing: 0,
                    padding: 10,
                    draw_icon.svg: (mod.widgets.ICON_CLOSE)
                    icon_walk: Walk{width: 14, height: 14}
                }
            }

            // Search input
            search_container := View {
                width: Fill,
                height: Fit,
                padding: Inset{left: 10, right: 10, bottom: 10},

                search_input := TextInput {
                    width: Fill,
                    height: Fit,
                    empty_text: "Search members...",
                    draw_text +: {
                        text_style: MESSAGE_TEXT_STYLE { font_size: 11 },
                    }
                    draw_bg +: {
                        color: (COLOR_PRIMARY_DARKER),
                        border_radius: 4.0,
                    }
                }
            }

            LineH { padding: 0 }

            // Members list using PortalList for efficient scrolling
            members_list := PortalList {
                width: Fill,
                height: Fill,
                flow: Down,

                MemberEntry := mod.widgets.MemberEntry {}
                Empty := View {}
            }
        }

        slide: 1.0,

        animator: Animator {
            panel: {
                default: @hide
                show: AnimatorState{
                    redraw: true,
                    from: {all: Forward {duration: 0.5}}
                    ease: Ease.ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {
                        slide: 0.0
                    }
                }
                hide: AnimatorState{
                    redraw: true,
                    from: {all: Forward {duration: 0.5}}
                    ease: Ease.ExpDecay {d1: 0.80, d2: 0.97}
                    apply: {
                        slide: 1.0
                    }
                }
            }
        }
    }
}

/// Information about a displayed member.
/// This is Clone and public to allow sharing with background search threads.
#[derive(Clone, Debug)]
pub struct DisplayedMember {
    /// The user ID of this member.
    pub user_id: OwnedUserId,
    /// The optional display name.
    pub(crate) display_name: Option<String>,
    /// The avatar state for this member.
    pub(crate) avatar_state: AvatarState,
    role: Option<String>,
}

impl DisplayedMember {
    fn from_room_member(member: &RoomMember) -> Self {
        let avatar_state = AvatarState::Known(member.avatar_url().map(|u| u.to_owned()));

        let role = match member.suggested_role_for_power_level() {
            matrix_sdk::room::RoomMemberRole::Administrator => Some("Admin".to_string()),
            matrix_sdk::room::RoomMemberRole::Moderator => Some("Mod".to_string()),
            _ => None,
        };

        Self {
            user_id: member.user_id().to_owned(),
            display_name: member.display_name().map(|s| s.to_string()),
            avatar_state,
            role,
        }
    }

    /// Returns the name to display for this member.
    /// Falls back to the user ID if no display name is set.
    pub fn displayable_name(&self) -> &str {
        self.display_name.as_deref()
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| self.user_id.as_str())
    }

    fn first_letter(&self) -> &str {
        self.display_name.as_deref()
            .and_then(|n| utils::user_name_first_letter(n))
            .or_else(|| utils::user_name_first_letter(self.user_id.as_str()))
            .unwrap_or_default()
    }
}

#[derive(Script, ScriptHook, Widget, Animator)]
pub struct MembersPanel {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,
    #[apply_default] animator: Animator,
    #[live] slide: f32,

    /// The list of members to display (Arc for sharing with background threads).
    #[rust] members: Arc<Vec<DisplayedMember>>,
    /// The filtered list of members (based on search).
    #[rust] filtered_members: Vec<usize>,
    /// The current search query.
    #[rust] search_query: String,
    /// The room ID for the current members.
    #[rust] room_id: Option<OwnedRoomId>,
    /// The room name.
    #[rust] room_name: String,
    /// Whether the panel is animating closed.
    #[rust] is_animating_out: bool,
    /// Current state of member search.
    #[rust] search_state: MemberSearchState,
    /// Timer for debouncing search input.
    #[rust] debounce_timer: Timer,
    /// Monotonically increasing ID for search operations.
    #[rust] next_search_id: u64,
    /// Cache of recent search results (query -> matched indices).
    #[rust] search_cache: HashMap<String, Vec<usize>>,
}

impl Widget for MembersPanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if !self.visible {
            return;
        }

        let animator_action = self.animator_handle_event(cx, event);
        if animator_action.must_redraw() {
            self.redraw(cx);
        }

        // Handle animation completion when closing
        if self.is_animating_out && !self.animator.is_track_animating(id!(panel)) {
            self.visible = false;
            self.is_animating_out = false;
            cx.revert_key_focus();
            self.view(cx, ids!(bg_view)).set_visible(cx, false);
            self.redraw(cx);
            return;
        }

        // Handle debounce timer expiration - trigger async search
        if self.debounce_timer.is_event(event).is_some() {
            if let MemberSearchState::Debouncing = &self.search_state {
                self.start_async_search(cx);
            }
        }

        // Handle Signal events - process search results from background thread
        if let Event::Signal = event {
            self.process_search_results(cx);
        }

        let area = self.view.area();

        // Close panel on: close button click, back gesture, Escape key, or click outside
        let close_panel = {
            matches!(
                event,
                Event::Actions(actions) if self.button(cx, ids!(close_button)).clicked(actions)
            )
            || event.back_pressed()
            || match event.hits_with_capture_overload(cx, area, true) {
                Hit::KeyUp(key) => key.key_code == KeyCode::Escape,
                Hit::FingerUp(fe) => {
                    let main_content_rect = self.view(cx, ids!(main_content)).area().rect(cx);
                    !main_content_rect.contains(fe.abs)
                }
                _ => false,
            }
        };

        if close_panel {
            self.hide(cx);
            return;
        }

        // Handle search input and member selection
        if let Event::Actions(actions) = event {
            // Handle search input changes
            let search_input = self.text_input(cx, ids!(search_input));
            if let Some(new_text) = search_input.changed(actions) {
                self.handle_search_input_changed(cx, new_text);
            }

            // Handle member selection from portal list
            let portal_list = self.portal_list(cx, ids!(members_list));
            for (index, item_ref) in portal_list.items_with_actions(actions) {
                // Check for click on member entry (which is a Button)
                if item_ref.as_button().clicked(actions) {
                    if let Some(&member_idx) = self.filtered_members.get(index) {
                        if let Some(member) = self.members.get(member_idx) {
                            if let Some(room_id) = &self.room_id {
                                // Trigger showing the user profile
                                cx.action(ShowUserProfileAction::ShowUserProfile(
                                    UserProfileAndRoomId {
                                        user_profile: UserProfile {
                                            user_id: member.user_id.clone(),
                                            username: member.display_name.clone(),
                                            avatar_state: member.avatar_state.clone(),
                                        },
                                        room_id: room_id.clone(),
                                    }
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        log!("MembersPanel::draw_walk() - visible={}, slide={}, members={}", self.visible, self.slide, self.filtered_members.len());

        // Apply slide animation by adjusting margin
        let slide_offset = (self.slide * 300.0) as f64;
        let margin = Inset {
            left: 0.0,
            top: 0.0,
            right: -slide_offset,
            bottom: 0.0,
        };
        let mut main_content = self.view(cx, ids!(main_content));
        script_apply_eval!(cx, main_content, {
            margin: #(margin)
        });

        // Set bg_view visibility based on animation state
        let bg_visible = self.slide < 1.0;
        self.view(cx, ids!(bg_view)).set_visible(cx, bg_visible);

        // Update member count label - show "Searching..." when search is in progress
        let count_text = match &self.search_state {
            MemberSearchState::Debouncing | MemberSearchState::Searching { .. } => {
                "Searching...".to_string()
            }
            MemberSearchState::Idle => {
                let count = self.filtered_members.len();
                let total = self.members.len();
                if self.search_query.is_empty() {
                    format!("({})", total)
                } else {
                    format!("({}/{})", count, total)
                }
            }
        };
        self.label(cx, ids!(member_count)).set_text(cx, &count_text);

        // Draw the members list
        let displayed_count = self.filtered_members.len();
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut portal_list) = item.as_portal_list().borrow_mut() {
                portal_list.set_item_range(cx, 0, displayed_count);

                while let Some(item_id) = portal_list.next_visible_item(cx) {
                    if item_id >= displayed_count {
                        let item = portal_list.item(cx, item_id, live_id!(Empty));
                        item.draw_all(cx, scope);
                        continue;
                    }

                    let member_idx = self.filtered_members[item_id];
                    let member = &self.members[member_idx];

                    let item = portal_list.item(cx, item_id, live_id!(MemberEntry));

                    // Set avatar - using show_text for text avatar, show_image for loaded avatar
                    let mut avatar_state = member.avatar_state.clone();
                    if let Some(data) = avatar_state.update_from_cache(cx) {
                        let _ = item.avatar(cx, ids!(avatar)).show_image(cx, None, |cx, img| {
                            utils::load_png_or_jpg(&img, cx, data)
                        });
                    } else {
                        item.avatar(cx, ids!(avatar)).show_text(cx, None, None, member.first_letter());
                    }

                    // Set display name and user ID
                    item.label(cx, ids!(display_name)).set_text(cx, member.displayable_name());
                    item.label(cx, ids!(user_id_label)).set_text(cx, member.user_id.as_str());

                    // Set role badge
                    if let Some(role) = &member.role {
                        item.view(cx, ids!(role_badge)).set_visible(cx, true);
                        item.label(cx, ids!(role_label)).set_text(cx, role);
                    } else {
                        item.view(cx, ids!(role_badge)).set_visible(cx, false);
                    }

                    item.draw_all(cx, scope);
                }
            }
        }

        DrawStep::done()
    }
}

impl MembersPanel {
    /// Shows the members panel
    pub fn show(&mut self, cx: &mut Cx) {
        log!("MembersPanel::show() - setting visible=true");
        self.visible = true;
        self.view(cx, ids!(bg_view)).set_visible(cx, true);
        self.animator_play(cx, ids!(panel.show));
        cx.set_key_focus(self.view.area());
        self.redraw(cx);
        log!("MembersPanel::show() - done");
    }

    /// Hides the members panel with animation
    pub fn hide(&mut self, cx: &mut Cx) {
        self.is_animating_out = true;
        self.animator_play(cx, ids!(panel.hide));
    }

    /// Sets the members to display.
    pub fn set_members(&mut self, cx: &mut Cx, members: Arc<Vec<RoomMember>>, room_id: OwnedRoomId, room_name: String) {
        log!("MembersPanel::set_members() - {} members for room {}", members.len(), room_id);
        self.members = Arc::new(
            members.iter()
                .map(DisplayedMember::from_room_member)
                .collect()
        );
        self.room_id = Some(room_id);
        self.room_name = room_name;
        self.search_query.clear();
        // Clear search state and cache when loading new members
        self.search_state = MemberSearchState::Idle;
        self.search_cache.clear();
        cx.stop_timer(self.debounce_timer);
        // Clear the search input
        self.text_input(cx, ids!(search_input)).set_text(cx, "");
        self.filter_members_sync();
        log!("MembersPanel::set_members() - filtered to {} members", self.filtered_members.len());
        self.redraw(cx);
    }

    /// Handles changes to the search input text.
    fn handle_search_input_changed(&mut self, cx: &mut Cx, new_text: String) {
        self.search_query = new_text.clone();

        // For empty queries, show all members immediately
        if new_text.is_empty() {
            cx.stop_timer(self.debounce_timer);
            self.search_state = MemberSearchState::Idle;
            self.filtered_members = (0..self.members.len()).collect();
            self.redraw(cx);
            return;
        }

        // Check cache first
        if let Some(cached_indices) = self.search_cache.get(&new_text) {
            cx.stop_timer(self.debounce_timer);
            self.search_state = MemberSearchState::Idle;
            self.filtered_members = cached_indices.clone();
            self.redraw(cx);
            return;
        }

        // For small member lists, filter synchronously
        if self.members.len() < ASYNC_SEARCH_THRESHOLD {
            cx.stop_timer(self.debounce_timer);
            self.search_state = MemberSearchState::Idle;
            self.filter_members_sync();
            self.redraw(cx);
            return;
        }

        // For larger lists, use debounced async search
        cx.stop_timer(self.debounce_timer);
        self.debounce_timer = cx.start_timeout(SEARCH_DEBOUNCE_DELAY);
        self.search_state = MemberSearchState::Debouncing;
        self.redraw(cx);
    }

    /// Starts an async search operation on a background thread.
    fn start_async_search(&mut self, cx: &mut Cx) {
        let Some(room_id) = self.room_id.clone() else {
            return;
        };

        let search_id = self.next_search_id;
        self.next_search_id = self.next_search_id.wrapping_add(1);
        let query = self.search_query.clone();

        self.search_state = MemberSearchState::Searching {
            query: query.clone(),
            search_id,
        };

        submit_async_request(MatrixRequest::SearchRoomMembers {
            search_id,
            query,
            room_id,
            members: Arc::clone(&self.members),
        });

        self.redraw(cx);
    }

    /// Processes search results from the background thread queue.
    fn process_search_results(&mut self, cx: &mut Cx) {
        let mut needs_redraw = false;

        while let Some(result) = PENDING_MEMBER_SEARCH_RESULTS.pop() {
            // Only process results for the current room
            if self.room_id.as_ref() != Some(&result.room_id) {
                continue;
            }

            // Check if this result matches the current search state
            if let MemberSearchState::Searching { search_id, query } = &self.search_state {
                if result.search_id == *search_id && result.query == *query {
                    // Update filtered members with the search results
                    self.filtered_members = result.matched_indices.clone();

                    // Cache the result (with LRU-style limit)
                    if self.search_cache.len() >= SEARCH_CACHE_LIMIT {
                        // Remove oldest entry (arbitrary key since HashMap doesn't preserve order)
                        if let Some(key) = self.search_cache.keys().next().cloned() {
                            self.search_cache.remove(&key);
                        }
                    }
                    self.search_cache.insert(result.query, result.matched_indices);

                    self.search_state = MemberSearchState::Idle;
                    needs_redraw = true;
                }
                // Ignore stale results (different search_id or query)
            }
        }

        if needs_redraw {
            self.redraw(cx);
        }
    }

    /// Filters the members synchronously based on the current search query.
    /// Used for small member lists where async search would be overkill.
    fn filter_members_sync(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_members = (0..self.members.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_members = self.members.iter()
                .enumerate()
                .filter(|(_, m)| {
                    m.displayable_name().to_lowercase().contains(&query)
                        || m.user_id.as_str().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
    }

    /// Returns whether the panel is currently shown.
    pub fn is_currently_shown(&self) -> bool {
        self.visible && !self.is_animating_out
    }
}

impl MembersPanelRef {
    /// Shows the members panel with the given members
    pub fn show_with_members(&self, cx: &mut Cx, members: Arc<Vec<RoomMember>>, room_id: OwnedRoomId, room_name: String) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_members(cx, members, room_id, room_name);
            inner.show(cx);
        }
    }

    /// Hides the members panel
    pub fn hide(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.hide(cx);
        }
    }

    /// Returns whether the panel is currently shown
    pub fn is_currently_shown(&self) -> bool {
        self.borrow().map(|inner| inner.is_currently_shown()).unwrap_or(false)
    }
}
