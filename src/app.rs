//! The top-level application content.
//!
//! See `handle_startup()` for the first code that runs on app startup.

use std::{cell::RefCell, collections::HashMap};
use makepad_widgets::*;
#[cfg(not(target_os = "android"))]
use makepad_widgets::makepad_platform::makepad_error_log::LOG_WITH_LEVEL;
use matrix_sdk::{RoomState, ruma::{OwnedEventId, OwnedRoomId, RoomId}};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::sync::Mutex;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::io::Write;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use std::fs::{File, OpenOptions};
use crate::{
    avatar_cache::clear_avatar_cache,
    call::call_state::CallAction,
    call::call_controls::CallControlsAction,
    call::call_screen::CallScreenWidgetRefExt,
    home::{
        event_source_modal::{EventSourceModalAction, EventSourceModalWidgetRefExt}, invite_modal::{InviteModalAction, InviteModalWidgetRefExt}, main_desktop_ui::MainDesktopUiAction, navigation_tab_bar::{NavigationBarAction, SelectedTab}, new_message_context_menu::NewMessageContextMenuWidgetRefExt, room_context_menu::RoomContextMenuWidgetRefExt, space_context_menu::{SpaceContextMenuDetails, SpaceContextMenuWidgetRefExt}, spaces_bar::SpacesBarAction, room_screen::{InviteAction, MessageAction, clear_timeline_states}, rooms_list::{RoomsListAction, RoomsListRef, RoomsListUpdate, RoomsListWidgetRefExt, clear_all_invited_rooms, enqueue_rooms_list_update}, rooms_list_header::RoomsListHeaderAction, rooms_list_header_dropdown::RoomsListHeaderDropdownWidgetRefExt
    }, join_leave_room_modal::{
        JoinLeaveModalKind, JoinLeaveRoomModalAction, JoinLeaveRoomModalWidgetRefExt
    }, login::login_screen::LoginAction, logout::logout_confirm_modal::{LogoutAction, LogoutConfirmModalAction, LogoutConfirmModalWidgetRefExt}, persistence, profile::user_profile_cache::clear_user_profile_cache, room::BasicRoomDetails, shared::{confirmation_modal::{ConfirmationModalContent, ConfirmationModalWidgetRefExt}, file_upload_modal::FilePreviewerAction, image_viewer::{ImageViewerAction, ImageViewerWidgetRefExt, LoadState}, popup_list::{PopupKind, enqueue_popup_notification}}, sliding_sync::{AccountSwitchAction, DirectMessageRoomAction, MatrixRequest, current_user_id, get_sync_service, submit_async_request}, utils::RoomNameId, verification::VerificationAction, verification_modal::{
        VerificationModalAction,
        VerificationModalWidgetRefExt,
    }
};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    load_all_resources() do #(App::script_component(vm)) {
        ui: Root {
            main_window := Window {
                window.inner_size: vec2(1280, 800)
                window.title: "Robrix"
                pass.clear_color: #FFFFFF00
                caption_bar: {
                    caption_label: {
                        label: {
                            margin: Inset{left: 65},
                            align: Align{x: 0.5},
                            text: "Robrix"
                        }
                    }
                }
            

                body +: {
                    padding: 0,

                    View {
                        width: Fill, height: Fill,
                        flow: Overlay,

                        home_screen_view := View {
                            visible: false
                            home_screen := HomeScreen {}
                        }
                        join_leave_modal := Modal {
                            content +: {
                                join_leave_modal_inner := JoinLeaveRoomModal {}
                            }
                        }
                        login_screen_view := View {
                            visible: true
                            login_screen := LoginScreen {}
                        }

                        image_viewer_overlay := View {
                            visible: false,
                            width: Fill, height: Fill,
                            image_viewer_modal_inner := ImageViewer {}
                        }

                        file_upload_modal := Modal {
                            content +: {
                                height: Fill, width: Fill,
                                align: Align{x: 0.5, y: 0.5},
                                file_upload_modal_inner := FileUploadModal {}
                            }
                        }

                        // Context menus should be shown in front of other UI elements,
                        // but behind verification modals.
                        new_message_context_menu := NewMessageContextMenu { }
                        room_context_menu := RoomContextMenu { }
                        space_context_menu := SpaceContextMenu { }
                        rooms_list_header_dropdown := RoomsListHeaderDropdown { }

                        // A modal to confirm sending out an invite to a room.
                        invite_confirmation_modal := Modal {
                            content +: {
                                invite_confirmation_modal_inner := PositiveConfirmationModal {
                                    wrapper +: { buttons_view +: { accept_button +: {
                                        draw_icon +: {
                                            svg: (ICON_INVITE),
                                            color: (COLOR_PRIMARY),
                                        }
                                        icon_walk: Walk{width: 28, height: Fit, margin: Inset{left: -10, right: 2} }
                                    } } }
                                }
                            }
                        }

                        // A modal to invite a user to a room.
                        invite_modal := Modal {
                            content +: {
                                invite_modal_inner := InviteModal {}
                            }
                        }

                        // Show the logout confirmation modal.
                        logout_confirm_modal := Modal {
                            content +: {
                                logout_confirm_modal_inner := LogoutConfirmModal {}
                            }
                        }

                        // Show the event source modal (View Source for messages).
                        event_source_modal := Modal {
                            content +: {
                                height: Fill,
                                width: Fill,
                                align: Align{x: 0.5, y: 0.5},
                                event_source_modal_inner := EventSourceModal {}
                            }
                        }

                        // Show incoming verification requests in front of the aforementioned UI elements.
                        verification_modal := Modal {
                            content +: {
                                verification_modal_inner := VerificationModal {}
                            }
                        }
                        tsp_verification_modal := Modal {
                            content +: {
                                tsp_verification_modal_inner := TspVerificationModal {}
                            }
                        }

                        // A generic modal to confirm any positive action.
                        positive_confirmation_modal := Modal {
                            content +: {
                                positive_confirmation_modal_inner := PositiveConfirmationModal { }
                            }
                        }

                        // A modal to confirm any deletion/removal action.
                        delete_confirmation_modal := Modal {
                            content +: {
                                delete_confirmation_modal_inner := NegativeConfirmationModal { }
                            }
                        }

                        // Call overlay - shown fullscreen during active calls
                        call_overlay := SolidView {
                            visible: false,
                            width: Fill, height: Fill,
                            show_bg: true,
                            draw_bg +: {
                                color: #e8e8e8
                            }
                            call_screen := CallScreen {}
                        }

                        PopupList {}

                        // Tooltips must be shown in front of all other UI elements,
                        // since they can be shown as a hover atop any other widget.
                        app_tooltip := CalloutTooltip {}
                    }
                } // end of body
            }
        }
    }
}

app_main!(App);

#[derive(Script)]
pub struct App {
    #[live] ui: WidgetRef,
    /// The top-level app state, shared across various parts of the app.
    #[rust] app_state: AppState,
    /// The details of a room we're waiting on to be loaded so that we can navigate to it.
    /// This can be either a room we're waiting to join, or one we're waiting to be invited to.
    /// Also includes an optional room ID to be closed once the awaited room has been loaded.
    #[rust] waiting_to_navigate_to_room: Option<(BasicRoomDetails, Option<OwnedRoomId>)>,
}

impl ScriptHook for App {
    /// After a hot-reload update, refresh the login/home screen visibility.
    fn on_after_reload(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            self.update_login_visibility(cx);
        });
    }

    /// After initial creation, set the global singleton for the PopupList widget.
    fn on_after_new(&mut self, vm: &mut ScriptVm) {
        vm.with_cx_mut(|cx| {
            crate::shared::popup_list::set_global_popup_list(cx, &self.ui);
        });
    }
}

// =============================================================================
// File Logging for Packaged Builds (non-mobile platforms)
// =============================================================================

/// Global log file handle for packaged builds.
/// Only used on desktop platforms when running as a packaged application.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
static LOG_FILE: std::sync::OnceLock<Option<Mutex<File>>> = std::sync::OnceLock::new();

/// Detects if the application is running as a packaged build (not via `cargo run`).
///
/// Detection methods per platform:
/// - macOS: Check if executable is inside a `.app/Contents/MacOS/` bundle
/// - Windows: Check if executable is in `Program Files` or similar installation directory
/// - Linux: Check if executable is in `/usr`, `/opt`, or is an AppImage
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn is_packaged_build() -> bool {
    let Ok(exe_path) = std::env::current_exe() else {
        return false;
    };
    let exe_path_str = exe_path.to_string_lossy();

    #[cfg(target_os = "macos")]
    {
        // Check if running from a .app bundle
        exe_path_str.contains(".app/Contents/MacOS/")
    }

    #[cfg(target_os = "windows")]
    {
        // Check if running from Program Files or a typical installation directory
        let exe_lower = exe_path_str.to_lowercase();
        exe_lower.contains("program files")
            || exe_lower.contains("programfiles")
            || exe_lower.contains("appdata\\local\\programs")
    }

    #[cfg(target_os = "linux")]
    {
        // Check if running from system directories or AppImage
        exe_path_str.starts_with("/usr/")
            || exe_path_str.starts_with("/opt/")
            || exe_path_str.contains(".AppImage")
            || std::env::var("APPIMAGE").is_ok()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

/// Initializes file logging for packaged builds.
/// Creates a log file in the app data directory with timestamp.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn init_file_logging() -> Option<()> {
    if !is_packaged_build() {
        LOG_FILE.get_or_init(|| None);
        return None;
    }

    // Get platform-specific logs directory
    let logs_dir = logs_dir();
    std::fs::create_dir_all(&logs_dir).ok()?;

    // Create log file with timestamp
    let now = chrono::Local::now();
    let log_filename = format!("robrix_{}.log", now.format("%Y-%m-%d_%H-%M-%S"));
    let log_path = logs_dir.join(&log_filename);

    // Also create/update a symlink to the latest log file for convenience
    let latest_log_path = logs_dir.join("robrix_latest.log");

    // Remove old symlink if it exists (ignore errors)
    #[cfg(unix)]
    {
        let _ = std::fs::remove_file(&latest_log_path);
        let _ = std::os::unix::fs::symlink(&log_filename, &latest_log_path);
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok()?;

    LOG_FILE.get_or_init(|| Some(Mutex::new(file)));

    // Print to stderr so user knows where logs are going
    eprintln!("[Robrix] Logging to file: {}", log_path.display());

    Some(())
}

/// Writes a log message to the log file (if file logging is enabled).
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn write_to_log_file(message: &str) {
    if let Some(Some(file_mutex)) = LOG_FILE.get() {
        if let Ok(mut file) = file_mutex.lock() {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let _ = writeln!(file, "[{}] {}", timestamp, message);
            let _ = file.flush();
        }
    }
}

/// Returns the path to the logs directory using platform-standard locations.
///
/// Platform-specific paths:
/// - macOS: `~/Library/Logs/Robrix/`
/// - Windows: `%APPDATA%/Robrix/logs/`
/// - Linux: `~/.local/share/robrix/logs/` (or `$XDG_DATA_HOME/robrix/logs/`)
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn logs_dir() -> std::path::PathBuf {
    use std::path::PathBuf;

    #[cfg(target_os = "macos")]
    {
        // macOS standard log location: ~/Library/Logs/Robrix/
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Logs")
                .join("Robrix");
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: %APPDATA%/Robrix/logs/
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("Robrix").join("logs");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Use XDG_DATA_HOME if set, otherwise ~/.local/share/
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg_data).join("robrix").join("logs");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("robrix")
                .join("logs");
        }
    }

    // Fallback to app data directory
    crate::app_data_dir().join("logs")
}

/// Cleans up old log files, keeping only the most recent N log files.
/// This should be called periodically to prevent disk space issues.
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn cleanup_old_logs(max_logs_to_keep: usize) {
    let logs_dir = logs_dir();
    if !logs_dir.exists() {
        return;
    }

    // Collect all log files (excluding the symlink)
    let mut log_files: Vec<_> = match std::fs::read_dir(&logs_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                name_str.starts_with("robrix_")
                    && name_str.ends_with(".log")
                    && name_str != "robrix_latest.log"
            })
            .collect(),
        Err(_) => return,
    };

    // Sort by modification time (oldest first)
    log_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        a_time.cmp(&b_time)
    });

    // Remove old log files
    if log_files.len() > max_logs_to_keep {
        let files_to_remove = log_files.len() - max_logs_to_keep;
        for entry in log_files.into_iter().take(files_to_remove) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

/// Maximum number of log files to keep
#[cfg(not(any(target_os = "android", target_os = "ios")))]
const MAX_LOG_FILES_TO_KEEP: usize = 10;

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        // Initialize the project directory here from the main UI thread
        // such that background threads/tasks will be able to access it.
        // This must be done before initializing file logging.
        let _app_data_dir = crate::app_data_dir();

        // Initialize file logging for packaged builds (non-mobile platforms).
        // This must be done before setting up the log handler.
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            init_file_logging();
            // Clean up old log files to prevent disk space issues
            cleanup_old_logs(MAX_LOG_FILES_TO_KEEP);
        }

        // only init logging/tracing once
        let _ = tracing_subscriber::fmt::try_init();

        // Override Makepad's default JSON logger with regular formatting.
        // Only do this on non-Android platforms to preserve Android's native logcat integration.
        #[cfg(not(target_os = "android"))]
        {
            fn regular_log(file_name: &str, line_start: u32, column_start: u32, _line_end: u32, _column_end: u32, message: String, level: LogLevel) {
                let l = match level {
                    LogLevel::Panic   => "[!]",
                    LogLevel::Error   => "[E]",
                    LogLevel::Warning => "[W]",
                    LogLevel::Log     => "[I]",
                    LogLevel::Wait    => "[.]",
                };
                let formatted_msg = format!("{l} {file_name}:{}:{}: {message}", line_start + 1, column_start + 1);

                // Print to stdout
                println!("{}", formatted_msg);

                // Also write to log file if file logging is enabled (packaged builds)
                #[cfg(not(target_os = "ios"))]
                write_to_log_file(&formatted_msg);
            }
            *LOG_WITH_LEVEL.write().unwrap() = regular_log;
        }

        log!("App::handle_startup(): app_data_dir: {:?}", _app_data_dir);

        // Set the global singleton for PopupList so other modules can enqueue toasts.
        crate::shared::popup_list::set_global_popup_list(cx, &self.ui);

        if let Err(e) = persistence::load_window_state(self.ui.window(cx, ids!(main_window)), cx) {
            error!("Failed to load window state: {}", e);
        }

        self.update_login_visibility(cx);

        log!("App::Startup: starting matrix sdk loop");
        let _tokio_rt_handle = crate::sliding_sync::start_matrix_tokio().unwrap();

        #[cfg(feature = "tsp")] {
            log!("App::Startup: initializing TSP (Trust Spanning Protocol) module.");
            crate::tsp::tsp_init(_tokio_rt_handle).unwrap();
        }
    }

    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        let invite_confirmation_modal_inner = self.ui.confirmation_modal(cx, ids!(invite_confirmation_modal_inner));
        if let Some(_accepted) = invite_confirmation_modal_inner.closed(actions) {
            self.ui.modal(cx, ids!(invite_confirmation_modal)).close(cx);
        }

        let delete_confirmation_modal_inner = self.ui.confirmation_modal(cx, ids!(delete_confirmation_modal_inner));
        if let Some(_accepted) = delete_confirmation_modal_inner.closed(actions) {
            self.ui.modal(cx, ids!(delete_confirmation_modal)).close(cx);
        }

        let positive_confirmation_modal_inner = self.ui.confirmation_modal(cx, ids!(positive_confirmation_modal_inner));
        if let Some(_accepted) = positive_confirmation_modal_inner.closed(actions) {
            self.ui.modal(cx, ids!(positive_confirmation_modal)).close(cx);
        }

        for action in actions {
            match action.downcast_ref() {
                Some(LogoutConfirmModalAction::Open) => {
                    self.ui.logout_confirm_modal(cx, ids!(logout_confirm_modal_inner)).reset_state(cx);
                    self.ui.modal(cx, ids!(logout_confirm_modal)).open(cx);
                    continue;
                },
                Some(LogoutConfirmModalAction::Close { was_internal, .. }) => {
                    if *was_internal {
                        self.ui.modal(cx, ids!(logout_confirm_modal)).close(cx);
                    }
                    continue;
                },
                _ => {}
            }

            match action.downcast_ref() {
                Some(LogoutAction::LogoutSuccess) => {
                    self.app_state.logged_in = false;
                    self.ui.modal(cx, ids!(logout_confirm_modal)).close(cx);
                    self.update_login_visibility(cx);
                    self.ui.redraw(cx);
                    continue;
                }
                Some(LogoutAction::ClearAppState { on_clear_appstate }) =>  {
                    // Clear user profile cache, invited_rooms timeline states 
                    clear_all_app_state(cx);
                    // Reset all app state to its default.
                    self.app_state = Default::default();
                    on_clear_appstate.notify_one();
                    continue;
                }
                _ => {}
            }

            if let Some(LoginAction::LoginSuccess) = action.downcast_ref() {
                log!("Received LoginAction::LoginSuccess, hiding login view.");
                self.app_state.logged_in = true;
                self.app_state.adding_account = false;
                self.update_login_visibility(cx);
                self.ui.redraw(cx);
                continue;
            }

            // Handle request to show login screen for adding another account
            if let Some(LoginAction::ShowAddAccountScreen) = action.downcast_ref() {
                log!("Received LoginAction::ShowAddAccountScreen, showing login view for adding account.");
                self.app_state.adding_account = true;
                self.ui.view(cx, ids!(login_screen_view)).set_visible(cx, true);
                self.ui.redraw(cx);
                continue;
            }

            // Handle successful addition of a new account
            if let Some(LoginAction::AddAccountSuccess) = action.downcast_ref() {
                log!("Received LoginAction::AddAccountSuccess, hiding login view.");
                self.app_state.adding_account = false;
                self.ui.view(cx, ids!(login_screen_view)).set_visible(cx, false);
                self.ui.redraw(cx);
                continue;
            }

            // Handle cancellation of adding an account
            if let Some(LoginAction::CancelAddAccount) = action.downcast_ref() {
                log!("Received LoginAction::CancelAddAccount, hiding login view.");
                self.app_state.adding_account = false;
                self.ui.view(cx, ids!(login_screen_view)).set_visible(cx, false);
                self.ui.redraw(cx);
                continue;
            }

            // Handle account switch actions
            match action.downcast_ref() {
                Some(AccountSwitchAction::Starting(user_id)) => {
                    log!("Account switch starting to: {}", user_id);
                    // Clear UI state during account switch
                    clear_all_app_state(cx);
                    self.app_state.selected_room = None;
                    // Clear saved dock state so tabs will be closed
                    self.app_state.saved_dock_state_home = Default::default();
                    self.app_state.saved_dock_state_per_space.clear();
                    // Reload the dock from the now-empty app state to close all tabs
                    cx.action(MainDesktopUiAction::LoadDockFromAppState);
                    enqueue_popup_notification(
                        format!("Switching to account {}...", user_id),
                        PopupKind::Info,
                        Some(5.0),
                    );
                    self.ui.redraw(cx);
                    continue;
                }
                Some(AccountSwitchAction::Switched(user_id)) => {
                    log!("Account switch completed to: {}", user_id);
                    enqueue_popup_notification(
                        format!("Switched to account {}", user_id),
                        PopupKind::Info,
                        Some(5.0),
                    );
                    self.ui.redraw(cx);
                    continue;
                }
                Some(AccountSwitchAction::Failed(error)) => {
                    log!("Account switch failed: {}", error);
                    enqueue_popup_notification(
                        format!("Failed to switch account: {}", error),
                        PopupKind::Error,
                        None,
                    );
                    continue;
                }
                _ => {}
            }

            // Handle an action requesting to open the new message context menu.
            if let MessageAction::OpenMessageContextMenu { details, abs_pos } = action.as_widget_action().cast() {
                self.ui.callout_tooltip(cx, ids!(app_tooltip)).hide(cx);
                let new_message_context_menu = self.ui.new_message_context_menu(cx, ids!(new_message_context_menu));
                let expected_dimensions = new_message_context_menu.show(cx, details);
                // Ensure the context menu does not spill over the window's bounds.
                let rect = self.ui.window(cx, ids!(main_window)).area().rect(cx);
                let pos_x = min(abs_pos.x, rect.size.x - expected_dimensions.x);
                let pos_y = min(abs_pos.y, rect.size.y - expected_dimensions.y);
                let margin = Inset {
                    left: pos_x as f64,
                    top: pos_y as f64,
                    right: 0.0,
                    bottom: 0.0,
                };
                let mut main_content_view = new_message_context_menu.view(cx, ids!(main_content));
                script_apply_eval!(cx, main_content_view, {
                    margin: #(margin)
                });
                self.ui.redraw(cx);
                continue;
            }

            // Handle an action requesting to open the room context menu.
            if let RoomsListAction::OpenRoomContextMenu { details, pos } = action.as_widget_action().cast() {
                self.ui.callout_tooltip(cx, ids!(app_tooltip)).hide(cx);
                let room_context_menu = self.ui.room_context_menu(cx, ids!(room_context_menu));
                let expected_dimensions = room_context_menu.show(cx, details);
                // Ensure the context menu does not spill over the window's bounds.
                let rect = self.ui.window(cx, ids!(main_window)).area().rect(cx);
                let pos_x = min(pos.x, rect.size.x - expected_dimensions.x);
                let pos_y = min(pos.y, rect.size.y - expected_dimensions.y);
                let margin = Inset {
                    left: pos_x as f64,
                    top: pos_y as f64,
                    right: 0.0,
                    bottom: 0.0,
                };
                let mut main_content_view = room_context_menu.view(cx, ids!(main_content));
                script_apply_eval!(cx, main_content_view, {
                    margin: #(margin)
                });
                self.ui.redraw(cx);
                continue;
            }

            // Handle an action requesting to open the space context menu.
            if let SpacesBarAction::ButtonSecondaryClicked { space_name_id, pos } = action.as_widget_action().cast() {
                // Get the space_request_sender from the RoomsList widget.
                let rooms_list: RoomsListRef = self.ui.rooms_list(cx, ids!(rooms_list));
                let Some(space_request_sender) = rooms_list.get_space_request_sender() else {
                    log!("Warning: could not get space_request_sender to open space context menu");
                    continue;
                };
                self.ui.callout_tooltip(cx, ids!(app_tooltip)).hide(cx);
                let space_context_menu = self.ui.space_context_menu(cx, ids!(space_context_menu));
                let details = SpaceContextMenuDetails {
                    space_name_id,
                    space_request_sender,
                };
                let expected_dimensions = space_context_menu.show(cx, details);
                // Ensure the context menu does not spill over the window's bounds.
                let rect = self.ui.window(cx, ids!(main_window)).area().rect(cx);
                let pos_x = min(pos.x, rect.size.x - expected_dimensions.x);
                let pos_y = min(pos.y, rect.size.y - expected_dimensions.y);
                let margin = Inset {
                    left: pos_x as f64,
                    top: pos_y as f64,
                    right: 0.0,
                    bottom: 0.0,
                };
                let mut main_content_view = space_context_menu.view(cx, ids!(main_content));
                script_apply_eval!(cx, main_content_view, {
                    margin: #(margin)
                });
                self.ui.redraw(cx);
                continue;
            }

            // Handle an action requesting to toggle the rooms list header dropdown.
            if let Some(RoomsListHeaderAction::ShowDropdown { pos, filter, sort }) = action.downcast_ref() {
                let dropdown = self.ui.rooms_list_header_dropdown(cx, ids!(rooms_list_header_dropdown));
                // Toggle: if already shown, hide it
                if dropdown.is_currently_shown(cx) {
                    if let Some(mut inner) = dropdown.borrow_mut() {
                        inner.visible = false;
                        inner.redraw(cx);
                    }
                    self.ui.redraw(cx);
                    continue;
                }
                dropdown.show(cx, *pos, *filter, *sort);
                // Ensure the dropdown does not spill over the window's bounds.
                let rect = self.ui.window(cx, ids!(main_window)).area().rect(cx);
                let dropdown_width = 200.0;
                let dropdown_height = 350.0;
                let pos_x = f64::min(pos.x, rect.size.x - dropdown_width);
                let pos_y = f64::min(pos.y, rect.size.y - dropdown_height);
                let margin = Inset {
                    left: pos_x,
                    top: pos_y,
                    right: 0.0,
                    bottom: 0.0,
                };
                let mut main_content_view = dropdown.view(cx, ids!(main_content));
                script_apply_eval!(cx, main_content_view, {
                    margin: #(margin)
                });
                self.ui.redraw(cx);
                continue;
            }

            // A new room has been selected, update the app state and navigate to the main content view.
            if let RoomsListAction::Selected(selected_room) = action.as_widget_action().cast() {
                // Set the Stack Navigation header to show the name of the newly-selected room.
                self.ui
                    .label(cx, ids!(main_content_view.header.content.title_container.title))
                    .set_text(cx, &selected_room.display_name());

                self.app_state.selected_room = Some(selected_room);

                // Navigate to the main content view
                cx.widget_action(
                    self.ui.widget_uid(), 
                    StackNavigationAction::Push(id!(main_content_view))
                );
                self.ui.redraw(cx);
                continue;
            }

            // Handle actions that instruct us to update the top-level app state.
            match action.downcast_ref() {
                Some(AppStateAction::RoomFocused(selected_room)) => {
                    self.app_state.selected_room = Some(selected_room.clone());
                    continue;
                }
                Some(AppStateAction::FocusNone) => {
                    self.app_state.selected_room = None;
                    continue;
                }
                Some(AppStateAction::UpgradedInviteToJoinedRoom(room_id)) => {
                    if let Some(selected_room) = self.app_state.selected_room.as_mut() {
                        let did_upgrade = selected_room.upgrade_invite_to_joined(room_id);
                        // Updating the AppState's selected room and issuing a redraw
                        // will cause the MainMobileUI to redraw the newly-joined room.
                        if did_upgrade {
                            self.ui.redraw(cx);
                        }
                    }
                    continue;
                }
                Some(AppStateAction::RestoreAppStateFromPersistentState(app_state)) => {
                    // Ignore the `logged_in` state that was stored persistently.
                    let logged_in_actual = self.app_state.logged_in;
                    self.app_state = app_state.clone();
                    self.app_state.logged_in = logged_in_actual;
                    cx.action(MainDesktopUiAction::LoadDockFromAppState);
                    continue;
                }
                Some(AppStateAction::NavigateToRoom { room_to_close, destination_room }) => {
                    self.navigate_to_room(cx, room_to_close.as_ref(), destination_room);
                    continue;
                }
                // If we successfully loaded a room that we were waiting on,
                // we can now navigate to it and optionally close a previous room.
                Some(AppStateAction::RoomLoadedSuccessfully { room_name_id, .. }) if
                    self.waiting_to_navigate_to_room.as_ref()
                        .is_some_and(|(dr, _)| dr.room_id() == room_name_id.room_id()) =>
                {
                    log!("Loaded awaited room {room_name_id:?}, navigating to it now...");
                    if let Some((dest_room, room_to_close)) = self.waiting_to_navigate_to_room.take() {
                        self.navigate_to_room(cx, room_to_close.as_ref(), &dest_room);
                    }
                    continue;
                }
                _ => {}
            }

            // Handle actions for showing or hiding the tooltip.
            match action.as_widget_action().cast() {
                TooltipAction::HoverIn { text, widget_rect, options } => {
                    // Don't show any tooltips if the message context menu is currently shown.
                    if self.ui.new_message_context_menu(cx, ids!(new_message_context_menu)).is_currently_shown(cx) {
                        self.ui.callout_tooltip(cx, ids!(app_tooltip)).hide(cx);
                    }
                    else {
                        self.ui.callout_tooltip(cx, ids!(app_tooltip)).show_with_options(
                            cx,
                            &text,
                            widget_rect,
                            options,
                        );
                    }
                    continue;
                }
                TooltipAction::HoverOut => {
                    self.ui.callout_tooltip(cx, ids!(app_tooltip)).hide(cx);
                    continue;
                }
                _ => {}
            }

            // Handle actions needed to open/close the join/leave room modal.
            match action.downcast_ref() {
                Some(JoinLeaveRoomModalAction::Open { kind, show_tip }) => {
                    self.ui
                        .join_leave_room_modal(cx, ids!(join_leave_modal_inner))
                        .set_kind(cx, kind.clone(), *show_tip);
                    self.ui.modal(cx, ids!(join_leave_modal)).open(cx);
                    continue;
                }
                Some(JoinLeaveRoomModalAction::Close { was_internal, .. }) => {
                    if *was_internal {
                        self.ui.modal(cx, ids!(join_leave_modal)).close(cx);
                    }
                    continue;
                }
                _ => {}
            }

            // `VerificationAction`s come from a background thread, so they are NOT widget actions.
            // Therefore, we cannot use `as_widget_action().cast()` to match them.
            //
            // Note: other verification actions are handled by the verification modal itself.
            if let Some(VerificationAction::RequestReceived(state)) = action.downcast_ref() {
                self.ui.verification_modal(cx, ids!(verification_modal_inner))
                    .initialize_with_data(cx, state.clone());
                self.ui.modal(cx, ids!(verification_modal)).open(cx);
                continue;
            }
            if let Some(VerificationModalAction::Close) = action.downcast_ref() {
                self.ui.modal(cx, ids!(verification_modal)).close(cx);
                continue;
            }
            match action.downcast_ref() {
                Some(ImageViewerAction::Show(state)) => {
                    let mut image_viewer = self.ui.image_viewer(cx, ids!(image_viewer_modal_inner));
                    match state {
                        LoadState::Loading(texture, metadata) => {
                            image_viewer.show_loading(cx, texture.clone(), metadata);
                            let overlay = self.ui.view(cx, ids!(image_viewer_overlay));
                            overlay.set_visible(cx, true);
                            cx.set_key_focus(overlay.area());
                            // Block scrolling immediately so scroll events don't
                            // pierce through to the timeline before the next draw.
                            cx.block_scrolling_except_within(Area::Empty);
                        }
                        LoadState::Loaded(data) => {
                            image_viewer.show_loaded(cx, data);
                        }
                        LoadState::Error(error) => {
                            image_viewer.show_error(cx, error);
                        }
                        LoadState::FinishedBackgroundDecoding => {
                            // Handled internally by the ImageViewer's Signal handler.
                        }
                    }
                    continue;
                }
                Some(ImageViewerAction::Hide) => {
                    self.ui.view(cx, ids!(image_viewer_overlay)).set_visible(cx, false);
                    cx.revert_key_focus();
                    cx.unblock_scrolling();
                    continue;
                }
                _ => {}
            }

            // Handle actions to show/hide the file upload modal.
            match action.downcast_ref() {
                Some(FilePreviewerAction::Show(_)) => {
                    self.ui.modal(cx, ids!(file_upload_modal)).open(cx);
                    continue;
                }
                Some(FilePreviewerAction::Hide) => {
                    self.ui.modal(cx, ids!(file_upload_modal)).close(cx);
                    continue;
                }
                _ => {}
            }

            // Handle actions to open/close the TSP verification modal.
            #[cfg(feature = "tsp")] {
                use std::ops::Deref;
                use crate::tsp::{tsp_verification_modal::{TspVerificationModalAction, TspVerificationModalWidgetRefExt}, TspIdentityAction};

                if let Some(TspIdentityAction::ReceivedDidAssociationRequest { details, wallet_db }) = action.downcast_ref() {
                    self.ui.tsp_verification_modal(cx, ids!(tsp_verification_modal_inner))
                        .initialize_with_details(cx, details.clone(), wallet_db.deref().clone());
                    self.ui.modal(cx, ids!(tsp_verification_modal)).open(cx);
                    continue;
                }
                if let Some(TspVerificationModalAction::Close) = action.downcast_ref() {
                    self.ui.modal(cx, ids!(tsp_verification_modal)).close(cx);
                    continue;
                }
            }

            // Handle a request to show the invite confirmation modal.
            if let Some(InviteAction::ShowInviteConfirmationModal(content_opt)) = action.downcast_ref() {
                if let Some(content) = content_opt.borrow_mut().take() {
                    invite_confirmation_modal_inner.show(cx, content);
                    self.ui.modal(cx, ids!(invite_confirmation_modal)).open(cx);
                }
                continue;
            }

            // Handle a request to show the generic positive confirmation modal.
            if let Some(PositiveConfirmationModalAction::Show(content_opt)) = action.downcast_ref() {
                if let Some(content) = content_opt.borrow_mut().take() {
                    positive_confirmation_modal_inner.show(cx, content);
                    self.ui.modal(cx, ids!(positive_confirmation_modal)).open(cx);
                }
                continue;
            }

            // Handle a request to show the delete confirmation modal.
            if let Some(ConfirmDeleteAction::Show(content_opt)) = action.downcast_ref() {
                if let Some(content) = content_opt.borrow_mut().take() {
                    self.ui.confirmation_modal(cx, ids!(delete_confirmation_modal_inner)).show(cx, content);
                    self.ui.modal(cx, ids!(delete_confirmation_modal)).open(cx);
                }
                continue;
            }

            // Handle InviteModalAction to open/close the invite modal.
            match action.downcast_ref() {
                Some(InviteModalAction::Open(room_name_id)) => {
                    self.ui.invite_modal(cx, ids!(invite_modal_inner)).show(cx, room_name_id.clone());
                    self.ui.modal(cx, ids!(invite_modal)).open(cx); 
                    continue;
                }
                Some(InviteModalAction::Close) => {
                    self.ui.modal(cx, ids!(invite_modal)).close(cx);
                    continue;
                }
                _ => {}
            }

            // Handle EventSourceModalAction to open/close the event source modal.
            match action.downcast_ref() {
                Some(EventSourceModalAction::Open { room_id, event_id, original_json }) => {
                    self.ui.event_source_modal(cx, ids!(event_source_modal_inner))
                        .show(cx, room_id.clone(), event_id.clone(), original_json.clone());
                    self.ui.modal(cx, ids!(event_source_modal)).open(cx);
                    continue;
                }
                Some(EventSourceModalAction::Close) => {
                    self.ui.modal(cx, ids!(event_source_modal)).close(cx);
                    continue;
                }
                _ => {}
            }

            // Handle call-related actions
            match action.downcast_ref() {
                Some(CallAction::StateChanged { room_id: _, new_state }) => {
                    // Update UI based on call state changes
                    match new_state {
                        crate::call::call_state::CallState::Connected { .. } => {
                            // Show call overlay
                            self.ui.view(cx, ids!(call_overlay)).set_visible(cx, true);
                            self.ui.call_screen(cx, ids!(call_screen))
                                .start_call_timer(cx);
                        }
                        crate::call::call_state::CallState::Idle
                        | crate::call::call_state::CallState::Ended { .. } => {
                            // Hide call overlay
                            self.ui.view(cx, ids!(call_overlay)).set_visible(cx, false);
                            self.ui.call_screen(cx, ids!(call_screen))
                                .stop_call_timer();
                        }
                        _ => {}
                    }
                    self.ui.redraw(cx);
                    continue;
                }
                Some(CallAction::ShowCallScreen { room_id, user_display_name }) => {
                    // Show the call overlay immediately when joining
                    self.ui.view(cx, ids!(call_overlay)).set_visible(cx, true);
                    self.ui.call_screen(cx, ids!(call_screen))
                        .set_room(cx, room_id.clone(), user_display_name.as_deref().unwrap_or(""));
                    self.ui.call_screen(cx, ids!(call_screen))
                        .start_call_timer(cx);
                    self.ui.redraw(cx);
                    continue;
                }
                Some(CallAction::HideCallScreen) => {
                    // Hide the call overlay
                    self.ui.view(cx, ids!(call_overlay)).set_visible(cx, false);
                    self.ui.call_screen(cx, ids!(call_screen))
                        .stop_call_timer();
                    self.ui.redraw(cx);
                    continue;
                }
                Some(CallAction::IncomingCall { room_id: _, caller, is_video_call }) => {
                    // Show incoming call notification
                    let call_type = if *is_video_call { "video" } else { "audio" };
                    enqueue_popup_notification(
                        format!("Incoming {} call from {}", call_type, caller.displayable_name()),
                        PopupKind::Info,
                        Some(30.0),
                    );
                    continue;
                }
                Some(CallAction::MediaError { error }) => {
                    enqueue_popup_notification(
                        format!("Call error: {}", error),
                        PopupKind::Error,
                        None,
                    );
                    continue;
                }
                _ => {}
            }

            // Handle call controls actions
            match action.as_widget_action().cast() {
                CallControlsAction::ToggleMute { room_id } => {
                    submit_async_request(MatrixRequest::ToggleCallAudio { room_id });
                    continue;
                }
                CallControlsAction::ToggleCamera { room_id } => {
                    submit_async_request(MatrixRequest::ToggleCallVideo { room_id });
                    continue;
                }
                CallControlsAction::EndCall { room_id } => {
                    submit_async_request(MatrixRequest::LeaveCall { room_id });
                    continue;
                }
                CallControlsAction::None => {}
            }

            // Handle DirectMessageRoomActions
            match action.downcast_ref() {
                Some(DirectMessageRoomAction::FoundExisting { room_name_id, .. }) => {
                    self.navigate_to_room(cx, None, &BasicRoomDetails::RoomId(room_name_id.clone()));
                }
                Some(DirectMessageRoomAction::DidNotExist { user_profile }) => {
                    let user_profile = user_profile.clone();
                    let body_text = match &user_profile.username {
                        Some(un) if !un.is_empty() => format!(
                            "You don't have an existing direct message room with {} ({}).\n\n\
                            Would you like to create one now?",
                            un,
                            user_profile.user_id,
                        ),
                        _ => format!(
                            "You don't have an existing direct message room with {}.\n\n\
                            Would you like to create one now?",
                            user_profile.user_id,
                        ),
                    };
                    positive_confirmation_modal_inner.show(
                        cx,
                        ConfirmationModalContent {
                            title_text: "Create New Direct Message".into(),
                            body_text: body_text.into(),
                            accept_button_text: Some("Create DM".into()),
                            on_accept_clicked: Some(Box::new(move |_cx| {
                                submit_async_request(MatrixRequest::OpenOrCreateDirectMessage {
                                    user_profile,
                                    allow_create: true,
                                });
                                enqueue_popup_notification(
                                    "Sending request to create DM room...\n\nThe room will be shown once it has been created by the homeserver.".to_string(),
                                    PopupKind::Info,
                                    Some(10.0),
                                );
                            })),
                            ..Default::default()
                        },
                    );
                    self.ui.modal(cx, ids!(positive_confirmation_modal)).open(cx);
                }
                Some(DirectMessageRoomAction::FailedToCreate { user_profile, error }) => {
                    enqueue_popup_notification(
                        format!("Failed to create a new DM room with {}.\n\nError: {error}", user_profile.displayable_name()),
                        PopupKind::Error,
                        None,
                    );
                }
                Some(DirectMessageRoomAction::NewlyCreated { room_name_id, .. }) => {
                    self.navigate_to_room(cx, None, &BasicRoomDetails::RoomId(room_name_id.clone()));
                }
                _ => {}
            }
        }
    }
}

/// Clears all thread-local UI caches (user profiles, invited rooms, and timeline states).
/// The `cx` parameter ensures that these thread-local caches are cleared on the main UI thread, 
fn clear_all_app_state(cx: &mut Cx) {
    clear_user_profile_cache(cx);
    clear_all_invited_rooms(cx);
    clear_timeline_states(cx);
    clear_avatar_cache(cx);
}

impl AppMain for App {
    fn script_mod(vm: &mut ScriptVm) -> makepad_widgets::ScriptValue {
        // Order matters: base widgets first, then app widgets, then app UI.
        makepad_widgets::script_mod(vm);
        makepad_code_editor::script_mod(vm);
        crate::shared::script_mod(vm);

        #[cfg(feature = "tsp")]
        crate::tsp::script_mod(vm);
        #[cfg(not(feature = "tsp"))]
        crate::tsp_dummy::script_mod(vm);

        crate::settings::script_mod(vm);
        // RoomInputBar depends on these Home widgets; preload them before room::script_mod.
        crate::home::location_preview::script_mod(vm);
        crate::home::tombstone_footer::script_mod(vm);
        crate::home::editing_pane::script_mod(vm);
        crate::room::script_mod(vm);
        crate::join_leave_room_modal::script_mod(vm);
        crate::verification_modal::script_mod(vm);
        crate::profile::script_mod(vm);
        crate::home::script_mod(vm);
        crate::login::script_mod(vm);
        crate::logout::script_mod(vm);
        // WebRTC call support
        crate::call::script_mod(vm);

        self::script_mod(vm)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        match event {
            Event::Shutdown => {
                let window_ref = self.ui.window(cx, ids!(main_window));
                if let Err(e) = persistence::save_window_state(window_ref, cx) {
                    error!("Failed to save window state. Error: {e}");
                }
                if let Some(user_id) = current_user_id() {
                    let app_state = self.app_state.clone();
                    if let Err(e) = persistence::save_app_state(app_state, user_id) {
                        error!("Failed to save app state. Error: {e}");
                    }
                }
                #[cfg(feature = "tsp")] {
                    // Save the TSP wallet state, if it exists, with a 3-second timeout.
                    let tsp_state = std::mem::take(&mut *crate::tsp::tsp_state_ref().lock().unwrap());
                    let res = crate::sliding_sync::block_on_async_with_timeout(
                        Some(std::time::Duration::from_secs(3)),
                        async move {
                            match tsp_state.close_and_serialize().await {
                                Ok(saved_state) => match persistence::save_tsp_state_async(saved_state).await {
                                    Ok(_) => { }
                                    Err(e) => error!("Failed to save TSP wallet state. Error: {e}"),
                                }
                                Err(e) => error!("Failed to close and serialize TSP wallet state. Error: {e}"),
                            }
                        },
                    );
                    if let Err(_e) = res {
                        error!("Failed to save TSP wallet state before app shutdown. Error: Timed Out.");
                    }
                }
            }
            Event::Pause => {
                // App is being paused (e.g., on mobile when another app comes to foreground)
                // Stop the sync service to save battery and network resources
                log!("App paused - stopping sync service");
                if let Some(sync_service) = get_sync_service() {
                    if let Ok(handle) = Handle::try_current() {
                        handle.spawn(async move {
                            sync_service.stop().await;
                            log!("Sync service stopped due to app pause");
                        });
                    }
                }
            }
            Event::Resume => {
                // App is resuming from a paused state
                // Restart the sync service to resume real-time updates
                log!("App resumed - restarting sync service");
                if let Some(sync_service) = get_sync_service() {
                    if let Ok(handle) = Handle::try_current() {
                        handle.spawn(async move {
                            sync_service.start().await;
                            log!("Sync service restarted due to app resume");
                        });
                    }
                }
            }
            // Note: Event::AppGotFocus and Event::AppLostFocus are not available in current makepad version
            // but can be added when makepad adds support for these events
            _ => { }
        }

        // Forward events to the MatchEvent trait implementation.
        self.match_event(cx, event);
        let scope = &mut Scope::with_data(&mut self.app_state);
        self.ui.handle_event(cx, event, scope);

        /*
         * TODO: I'd like for this to work, but it doesn't behave as expected.
         *       The context menu fails to draw properly when a draw event is passed to it.
         *       Also, once we do get this to work, we should remove the
         *       Hit::FingerScroll event handler in the new_message_context_menu widget.
         *
        // We only forward "interactive hit" events to the underlying UI view
        // if none of the various overlay views are visible.
        // Currently, the only overlay view that captures interactive events is
        // the new message context menu.
        // We always forward "non-interactive hit" events to the inner UI view.
        // We check which overlay views are visible in the order of those views' z-ordering,
        // such that the top-most views get a chance to handle the event first.

        let new_message_context_menu = self.ui.new_message_context_menu(cx, ids!(new_message_context_menu));
        let is_interactive_hit = utils::is_interactive_hit_event(event);
        let is_pane_shown: bool;
        if new_message_context_menu.is_currently_shown(cx) {
            is_pane_shown = true;
            new_message_context_menu.handle_event(cx, event, scope);
        }
        else {
            is_pane_shown = false;
        }

        if !is_pane_shown || !is_interactive_hit {
            // Forward the event to the inner UI view.
            self.ui.handle_event(cx, event, scope);
        }
         *
         */
    }
}

impl App {
    fn update_login_visibility(&self, cx: &mut Cx) {
        let show_login = !self.app_state.logged_in;
        if !show_login {
            self.ui
                .modal(cx, ids!(login_screen_view.login_screen.login_status_modal))
                .close(cx);
        }
        self.ui.view(cx, ids!(login_screen_view)).set_visible(cx, show_login);
        self.ui.view(cx, ids!(home_screen_view)).set_visible(cx, !show_login);
    }

    /// Navigates to the given `destination_room`, optionally closing the `room_to_close`.
    fn navigate_to_room(
        &mut self,
        cx: &mut Cx,
        room_to_close: Option<&OwnedRoomId>,
        destination_room: &BasicRoomDetails,
    ) {
        // A closure that closes the given `room_to_close`, if it exists in an open tab.
        let close_room_closure_opt = room_to_close.map(|to_close| {
            let tab_id = LiveId::from_str(to_close.as_str());
            let widget_uid = self.ui.widget_uid();
            move |cx: &mut Cx| {
                cx.widget_action(
                    widget_uid, 
                    DockAction::TabCloseWasPressed(tab_id),
                );
                enqueue_rooms_list_update(RoomsListUpdate::HideRoom { room_id: to_close.clone() });
            }
        });

        let destination_room_id = destination_room.room_id();
        let room_state = cx.get_global::<RoomsListRef>().get_room_state(destination_room_id);
        let new_selected_room = match room_state {
            Some(RoomState::Joined) => SelectedRoom::JoinedRoom {
                room_name_id: destination_room.room_name_id().clone(),
            },
            Some(RoomState::Invited) => SelectedRoom::InvitedRoom {
                room_name_id: destination_room.room_name_id().clone(),
            },
            // If the destination room is not yet loaded, show a join modal.
            _ => {
                log!("Destination room {:?} not loaded, showing join modal...", destination_room.room_name_id());
                self.waiting_to_navigate_to_room = Some((
                    destination_room.clone(),
                    room_to_close.cloned(),
                ));
                cx.action(JoinLeaveRoomModalAction::Open {
                    kind: JoinLeaveModalKind::JoinRoom {
                        details: destination_room.clone(),
                        is_space: false,
                    },
                    show_tip: false,
                });
                return;
            }
        };


        log!("Navigating to destination room {:?}, closing room {:?}",
            destination_room.room_name_id(),
            room_to_close,
        );

        // Before we navigate to the room, if the AddRoom tab is currently shown,
        // then we programmatically navigate to the Home tab to show the actual room.
        if matches!(self.app_state.selected_tab, SelectedTab::AddRoom) {
            cx.action(NavigationBarAction::GoToHome);
        }
        cx.widget_action(
            self.ui.widget_uid(), 
            RoomsListAction::Selected(new_selected_room),
        );
        // Select and scroll to the destination room in the rooms list.
        enqueue_rooms_list_update(RoomsListUpdate::ScrollToRoom(destination_room_id.clone()));

        // Close a previously/currently-open room if specified.
        if let Some(closure) = close_room_closure_opt {
            closure(cx);
        }
    }
}

/// App-wide state that is stored persistently across multiple app runs
/// and shared/updated across various parts of the app.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AppState {
    /// The currently-selected room, which is highlighted (selected) in the RoomsList
    /// and considered "active" in the main rooms screen.
    pub selected_room: Option<SelectedRoom>,
    /// The currently-selected navigation tab: defines which top-level view is shown.
    ///
    /// This field is only updated by the `HomeScreen` widget, which has the
    /// necessary context to be able to determine how it should be modified.
    ///
    /// This is not saved to or restored from persistent storage,
    /// so the `Home` screen and tab are always selected upon app startup.
    #[serde(skip)]
    pub selected_tab: SelectedTab,
    /// The saved "snapshot" of the dock's UI layout/state for the main "all rooms" home view.
    pub saved_dock_state_home: SavedDockState,
    /// The saved "snapshot" of the dock's UI layout/state for each space,
    /// keyed by the space ID.
    pub saved_dock_state_per_space: HashMap<OwnedRoomId, SavedDockState>,
    /// Whether a user is currently logged in to Robrix or not.
    pub logged_in: bool,
    /// Whether the app is currently showing the login screen for adding another account.
    /// This is transient state and not persisted.
    #[serde(skip)]
    pub adding_account: bool,
}

/// A snapshot of the main dock: all state needed to restore the dock tabs/layout.
///
/// This struct uses custom serialization to store room IDs as strings instead of
/// Makepad `LiveId` hashes, making the persisted state human-readable and robust
/// against hash algorithm changes.
#[derive(Clone, Default, Debug)]
pub struct SavedDockState {
    /// All items contained in the dock, keyed by their room or space ID.
    pub dock_items: HashMap<LiveId, DockItem>,
    /// The rooms that are currently open, keyed by their room or space ID.
    pub open_rooms: HashMap<LiveId, SelectedRoom>,
    /// The order in which the rooms were opened, in chronological order
    /// from first opened (at the beginning) to last opened (at the end).
    pub room_order: Vec<SelectedRoom>,
    /// The selected room tab in this dock when the dock state was saved.
    pub selected_room: Option<SelectedRoom>,
}

/// Serializable version of `DockItem` that uses strings instead of `LiveId`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SerializableDockItem {
    Splitter {
        axis: SplitterAxis,
        align: SplitterAlign,
        a: String,
        b: String,
    },
    Tabs {
        tabs: Vec<String>,
        selected: usize,
        closable: bool,
        #[serde(default)]
        hide_tab_bar: bool,
    },
    Tab {
        name: String,
        template: String,
        kind: String,
    },
}

/// Serializable version of `SavedDockState` that uses room ID strings as keys.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct SerializableSavedDockState {
    /// All items contained in the dock, keyed by room ID string or internal ID.
    pub dock_items: HashMap<String, SerializableDockItem>,
    /// The rooms that are currently open, keyed by their room ID string.
    pub open_rooms: HashMap<String, SelectedRoom>,
    /// The order in which the rooms were opened.
    pub room_order: Vec<SelectedRoom>,
    /// The selected room tab in this dock when the dock state was saved.
    pub selected_room: Option<SelectedRoom>,
}

impl SavedDockState {
    /// Converts a `LiveId` to a string key for serialization.
    ///
    /// For room tabs, uses the room ID string from `open_rooms`.
    /// For internal dock items (splitters, tab containers), uses a hex representation.
    fn live_id_to_string(&self, id: LiveId) -> String {
        // Check if this LiveId corresponds to a known room
        if let Some(selected_room) = self.open_rooms.get(&id) {
            return selected_room.tab_id_string();
        }
        // For internal dock IDs (splitters, tab containers), use hex format
        format!("__dock_{:016x}", id.0)
    }

    /// Converts a string key back to a `LiveId`.
    fn string_to_live_id(s: &str) -> LiveId {
        if let Some(hex) = s.strip_prefix("__dock_") {
            // Internal dock ID - parse hex
            LiveId(u64::from_str_radix(hex, 16).unwrap_or(0))
        } else {
            // Room ID string - hash it back to LiveId
            LiveId::from_str(s)
        }
    }

    /// Converts this `SavedDockState` to a serializable format with string keys.
    pub fn to_serializable(&self) -> SerializableSavedDockState {
        // Build the LiveId → String mapping
        let id_to_string: HashMap<LiveId, String> = self.dock_items.keys()
            .map(|id| (*id, self.live_id_to_string(*id)))
            .collect();

        // Convert dock_items
        let dock_items = self.dock_items.iter()
            .map(|(id, item)| {
                let key = id_to_string.get(id).cloned().unwrap_or_else(|| self.live_id_to_string(*id));
                let serializable_item = match item {
                    DockItem::Splitter { axis, align, a, b } => SerializableDockItem::Splitter {
                        axis: *axis,
                        align: *align,
                        a: id_to_string.get(a).cloned().unwrap_or_else(|| self.live_id_to_string(*a)),
                        b: id_to_string.get(b).cloned().unwrap_or_else(|| self.live_id_to_string(*b)),
                    },
                    DockItem::Tabs { tabs, selected, closable, hide_tab_bar } => SerializableDockItem::Tabs {
                        tabs: tabs.iter()
                            .map(|t| id_to_string.get(t).cloned().unwrap_or_else(|| self.live_id_to_string(*t)))
                            .collect(),
                        selected: *selected,
                        closable: *closable,
                        hide_tab_bar: *hide_tab_bar,
                    },
                    DockItem::Tab { name, template, kind } => SerializableDockItem::Tab {
                        name: name.clone(),
                        template: live_id_to_known_string(*template),
                        kind: live_id_to_known_string(*kind),
                    },
                };
                (key, serializable_item)
            })
            .collect();

        // Convert open_rooms to use room ID strings as keys
        let open_rooms = self.open_rooms.iter()
            .map(|(_, room)| (room.tab_id_string(), room.clone()))
            .collect();

        SerializableSavedDockState {
            dock_items,
            open_rooms,
            room_order: self.room_order.clone(),
            selected_room: self.selected_room.clone(),
        }
    }

    /// Creates a `SavedDockState` from a serializable format.
    pub fn from_serializable(serializable: SerializableSavedDockState) -> Self {
        // Convert dock_items
        let dock_items = serializable.dock_items.iter()
            .map(|(key, item)| {
                let id = Self::string_to_live_id(key);
                let dock_item = match item {
                    SerializableDockItem::Splitter { axis, align, a, b } => DockItem::Splitter {
                        axis: *axis,
                        align: *align,
                        a: Self::string_to_live_id(a),
                        b: Self::string_to_live_id(b),
                    },
                    SerializableDockItem::Tabs { tabs, selected, closable, hide_tab_bar } => DockItem::Tabs {
                        tabs: tabs.iter().map(|t| Self::string_to_live_id(t)).collect(),
                        selected: *selected,
                        closable: *closable,
                        hide_tab_bar: *hide_tab_bar,
                    },
                    SerializableDockItem::Tab { name, template, kind } => DockItem::Tab {
                        name: name.clone(),
                        template: known_string_to_live_id(template),
                        kind: known_string_to_live_id(kind),
                    },
                };
                (id, dock_item)
            })
            .collect();

        // Convert open_rooms
        let open_rooms = serializable.open_rooms.iter()
            .map(|(_, room)| (room.tab_id(), room.clone()))
            .collect();

        SavedDockState {
            dock_items,
            open_rooms,
            room_order: serializable.room_order,
            selected_room: serializable.selected_room,
        }
    }
}

/// Converts well-known LiveIds (like template and kind) to readable strings.
fn live_id_to_known_string(id: LiveId) -> String {
    // Check for well-known widget kinds
    if id == id!(room_screen) { return "room_screen".to_string(); }
    if id == id!(invite_screen) { return "invite_screen".to_string(); }
    if id == id!(space_lobby_screen) { return "space_lobby_screen".to_string(); }
    if id == id!(welcome_screen) { return "welcome_screen".to_string(); }
    if id == id!(settings_screen) { return "settings_screen".to_string(); }
    // Check for well-known templates
    if id == id!(CloseableTab) { return "CloseableTab".to_string(); }
    if id == id!(PermanentTab) { return "PermanentTab".to_string(); }
    // Fallback to hex
    format!("__id_{:016x}", id.0)
}

/// Converts string back to well-known LiveIds.
fn known_string_to_live_id(s: &str) -> LiveId {
    match s {
        "room_screen" => id!(room_screen),
        "invite_screen" => id!(invite_screen),
        "space_lobby_screen" => id!(space_lobby_screen),
        "welcome_screen" => id!(welcome_screen),
        "settings_screen" => id!(settings_screen),
        "CloseableTab" => id!(CloseableTab),
        "PermanentTab" => id!(PermanentTab),
        _ if s.starts_with("__id_") => {
            let hex = &s[5..];
            LiveId(u64::from_str_radix(hex, 16).unwrap_or(0))
        }
        _ => LiveId::from_str(s),
    }
}

impl Serialize for SavedDockState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_serializable().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SavedDockState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // First, try to deserialize as the new format
        let value = serde_json::Value::deserialize(deserializer)?;

        // Check if this is the old format (keys are numeric LiveId values)
        if let Some(obj) = value.as_object() {
            if let Some(dock_items) = obj.get("dock_items").and_then(|v| v.as_object()) {
                // Check if the first key looks like a numeric LiveId (old format)
                if let Some(first_key) = dock_items.keys().next() {
                    if first_key.parse::<u64>().is_ok() {
                        // Old format detected - deserialize using legacy format
                        return Self::deserialize_legacy(&value)
                            .map_err(serde::de::Error::custom);
                    }
                }
            }
        }

        // New format - deserialize as SerializableSavedDockState
        let serializable: SerializableSavedDockState = serde_json::from_value(value)
            .map_err(serde::de::Error::custom)?;
        Ok(Self::from_serializable(serializable))
    }
}

impl SavedDockState {
    /// Deserializes from the legacy format where keys are numeric LiveId values.
    fn deserialize_legacy(value: &serde_json::Value) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct LegacySavedDockState {
            dock_items: HashMap<u64, DockItem>,
            open_rooms: HashMap<u64, SelectedRoom>,
            room_order: Vec<SelectedRoom>,
            selected_room: Option<SelectedRoom>,
        }

        let legacy: LegacySavedDockState = serde_json::from_value(value.clone())
            .map_err(|e| format!("Failed to deserialize legacy format: {e}"))?;

        Ok(SavedDockState {
            dock_items: legacy.dock_items.into_iter()
                .map(|(id, item)| (LiveId(id), item))
                .collect(),
            open_rooms: legacy.open_rooms.into_iter()
                .map(|(id, room)| (LiveId(id), room))
                .collect(),
            room_order: legacy.room_order,
            selected_room: legacy.selected_room,
        })
    }
}


/// Represents a room currently or previously selected by the user.
///
/// ## PartialEq/Eq equality comparison behavior
/// Room/Space names are ignored for the purpose of equality comparison.
/// Two `SelectedRoom`s are considered equal if their `room_id`s are equal,
/// unless they are `Thread`s,` in which case their `thread_root_event_id`s
/// are also compared for equality.
/// A `Thread` is never considered equal to a non-`Thread`, even if their `room_id`s are equal.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SelectedRoom {
    JoinedRoom {
        room_name_id: RoomNameId,
    },
    Thread {
        room_name_id: RoomNameId,
        /// The event ID of the root message of this thread,
        /// which is used to distinguish this thread from the main room timeline.
        thread_root_event_id: OwnedEventId,
    },
    InvitedRoom {
        room_name_id: RoomNameId,
    },
    Space {
        space_name_id: RoomNameId,
    },
}

impl SelectedRoom {
    pub fn room_id(&self) -> &OwnedRoomId {
        match self {
            SelectedRoom::JoinedRoom { room_name_id } => room_name_id.room_id(),
            SelectedRoom::InvitedRoom { room_name_id } => room_name_id.room_id(),
            SelectedRoom::Space { space_name_id } => space_name_id.room_id(),
            SelectedRoom::Thread { room_name_id, .. } => room_name_id.room_id(),
        }
    }

    pub fn room_name(&self) -> &RoomNameId {
        match self {
            SelectedRoom::JoinedRoom { room_name_id } => room_name_id,
            SelectedRoom::InvitedRoom { room_name_id } => room_name_id,
            SelectedRoom::Space { space_name_id } => space_name_id,
            SelectedRoom::Thread { room_name_id, .. } => room_name_id,
        }
    }

    /// Upgrades this room from an invite to a joined room
    /// if its `room_id` matches the given `room_id`.
    ///
    /// Returns `true` if the room was an `InvitedRoom` with the same `room_id`
    /// that was successfully upgraded to a `JoinedRoom`;
    /// otherwise, returns `false`.
    pub fn upgrade_invite_to_joined(&mut self, room_id: &RoomId) -> bool {
        match self {
            SelectedRoom::InvitedRoom { room_name_id } if room_name_id.room_id() == room_id => {
                let name = room_name_id.clone();
                *self = SelectedRoom::JoinedRoom {
                    room_name_id: name,
                };
                true
            }
            _ => false,
        }
    }

    /// Returns the `LiveId` of the room tab corresponding to this `SelectedRoom`.
    pub fn tab_id(&self) -> LiveId {
        LiveId::from_str(&self.tab_id_string())
    }

    /// Returns the string key used for serialization and to generate the tab's `LiveId`.
    ///
    /// For threads, this includes both the room ID and thread root event ID.
    /// For other rooms, this is just the room ID string.
    pub fn tab_id_string(&self) -> String {
        match self {
            SelectedRoom::Thread { room_name_id, thread_root_event_id } => {
                format!("{}##{}", room_name_id.room_id(), thread_root_event_id)
            }
            other => other.room_id().to_string(),
        }
    }

    /// Returns the display name to be shown for this room in the UI.
    pub fn display_name(&self) -> String {
        match self {
            SelectedRoom::JoinedRoom { room_name_id } => room_name_id.to_string(),
            SelectedRoom::InvitedRoom { room_name_id } => room_name_id.to_string(),
            SelectedRoom::Space { space_name_id } => format!("[Space] {space_name_id}"),
            SelectedRoom::Thread { room_name_id, .. } => format!("[Thread] {room_name_id}"),
        }
    }
}

impl PartialEq for SelectedRoom {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                SelectedRoom::Thread {
                    room_name_id: lhs_room_name_id,
                    thread_root_event_id: lhs_thread_root_event_id,
                },
                SelectedRoom::Thread {
                    room_name_id: rhs_room_name_id,
                    thread_root_event_id: rhs_thread_root_event_id,
                },
            ) => {
                lhs_room_name_id.room_id() == rhs_room_name_id.room_id()
                    && lhs_thread_root_event_id == rhs_thread_root_event_id
            }
            (SelectedRoom::Thread { .. }, _) | (_, SelectedRoom::Thread { .. }) => false,
            _ => self.room_id() == other.room_id(),
        }
    }
}
impl Eq for SelectedRoom {}

/// Actions sent to the top-level App in order to update / restore its [`AppState`].
///
/// These are *NOT* widget actions.
#[derive(Debug)]
pub enum AppStateAction {
    /// The given room was focused (selected).
    RoomFocused(SelectedRoom),
    /// Resets the focus to none, meaning that no room is selected.
    FocusNone,
    /// The given room has successfully been upgraded from being displayed
    /// as an InviteScreen to a RoomScreen.
    UpgradedInviteToJoinedRoom(OwnedRoomId),
    /// The given app state was loaded from persistent storage
    /// and is ready to be restored.
    RestoreAppStateFromPersistentState(AppState),
    /// The given room was successfully loaded from the homeserver
    /// and is now known to our client.
    ///
    /// The RoomScreen for this room can now fully display the room's timeline.
    RoomLoadedSuccessfully {
        room_name_id: RoomNameId,
        /// `true` if this room is an invitation, `false` otherwise.
        is_invite: bool,
    },
    /// A request to navigate to a different room, optionally closing a prior/current room.
    NavigateToRoom {
        room_to_close: Option<OwnedRoomId>,
        destination_room: BasicRoomDetails,
    },
    None,
}

/// An action to show the generic top-level positive confirmation modal.
///
/// This is NOT a widget action.
#[derive(Debug)]
pub enum PositiveConfirmationModalAction {
    /// Show the confirmation modal with the given content.
    ///
    /// The content is wrapped in a `RefCell` to ensure that only one entity handles it
    /// and that that one entity can take ownership of the content object,
    /// which avoids having to clone it.
    Show(RefCell<Option<ConfirmationModalContent>>),
}

/// An action to show a deletion/removal confirmation modal.
///
/// This is NOT a widget action.
#[derive(Debug)]
pub enum ConfirmDeleteAction {
    /// Show the deletion confirmation modal with the given content.
    ///
    /// The content is wrapped in a `RefCell` to ensure that only one entity handles it
    /// and that that one entity can take ownership of the content object,
    /// which avoids having to clone it.
    Show(RefCell<Option<ConfirmationModalContent>>),
}
