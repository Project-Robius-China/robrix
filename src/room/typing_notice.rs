//! A notice that slides into view when someone is typing.
//!
//! This shows avatars of typing users (up to 3) alongside a text message
//! indicating who is currently typing in the room.

use makepad_widgets::*;

use crate::{
    avatar_cache::{self, AvatarCacheEntry},
    home::room_screen::TypingUser,
    shared::{
        avatar::{AvatarRef, AvatarWidgetExt},
        bouncing_dots::BouncingDotsWidgetExt,
    },
    utils,
};

/// Maximum number of typing user avatars to display.
const MAX_TYPING_AVATARS: usize = 3;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*


    mod.widgets.TYPING_NOTICE_ANIMATION_DURATION_SECS = 0.3
    mod.widgets.TYPING_AVATAR_SIZE = 20.0

    mod.widgets.TypingNotice = set_type_default() do #(TypingNotice::register_widget(vm)) {
        ..mod.widgets.SolidView

        visible: false
        width: Fill
        height: 30
        flow: Right
        align: Align{y: 0.5}
        padding: Inset{left: 12.0, top: 5.0, bottom: 5.0, right: 10.0}

        show_bg: true
        draw_bg +: {
            color: #e8f4ff
        }

        // Container for typing user avatars (up to 3)
        avatars_view := View {
            width: Fit
            height: Fit
            flow: Right
            spacing: -6.0  // Overlap avatars slightly like Element Web
            align: Align{y: 0.5}

            avatar1 := Avatar {
                visible: false
                width: (mod.widgets.TYPING_AVATAR_SIZE)
                height: (mod.widgets.TYPING_AVATAR_SIZE)
            }
            avatar2 := Avatar {
                visible: false
                width: (mod.widgets.TYPING_AVATAR_SIZE)
                height: (mod.widgets.TYPING_AVATAR_SIZE)
            }
            avatar3 := Avatar {
                visible: false
                width: (mod.widgets.TYPING_AVATAR_SIZE)
                height: (mod.widgets.TYPING_AVATAR_SIZE)
            }
        }

        typing_label := Label {
            align: Align{x: 0.0, y: 0.5},
            padding: Inset{left: 8.0, right: 0.0, top: 0.0, bottom: 0.0}
            draw_text +: {
                color: (TYPING_NOTICE_TEXT_COLOR),
                text_style: REGULAR_TEXT {font_size: 9}
            }
            text: "Someone is typing"
        }

        bouncing_dots := BouncingDots {
            margin: Inset{top: 1.1, left: -4 }
            padding: 0.0,
            draw_bg.color: (TYPING_NOTICE_TEXT_COLOR)
        }

        animator: Animator{
            typing_notice_animator: {
                default: @show
                show: AnimatorState{
                    redraw: true,
                    from: { all: Forward { duration: (mod.widgets.TYPING_NOTICE_ANIMATION_DURATION_SECS) } }
                    apply: { height: 30 }
                }
                hide: AnimatorState{
                    redraw: true,
                    from: { all: Forward { duration: (mod.widgets.TYPING_NOTICE_ANIMATION_DURATION_SECS) } }
                    apply: { height: 0 }
                }
            }
        }
    }
}

/// A notice that slides into view when someone is typing.
#[derive(Script, ScriptHook, Widget, Animator)]
pub struct TypingNotice {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,
    #[apply_default] animator: Animator,
}

impl Widget for TypingNotice {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl TypingNotice {
    /// Shows or hides the typing notice based on whether there are any typing users.
    fn show_or_hide(&mut self, cx: &mut Cx, typing_users: &[TypingUser]) {
        if typing_users.is_empty() {
            // Animate out the typing notice view (sliding it out towards the bottom).
            self.animator_play(cx, ids!(typing_notice_animator.hide));
            self.view.bouncing_dots(cx, ids!(bouncing_dots)).stop_animation(cx);
            return;
        }

        // Set up avatars for typing users (up to MAX_TYPING_AVATARS)
        let avatar_ids = [ids!(avatars_view.avatar1), ids!(avatars_view.avatar2), ids!(avatars_view.avatar3)];
        for (i, avatar_id) in avatar_ids.iter().enumerate() {
            let avatar_ref = self.view.avatar(cx, *avatar_id);
            if i < typing_users.len() && i < MAX_TYPING_AVATARS {
                let user = &typing_users[i];
                self.set_typing_avatar(cx, &avatar_ref, user);
                avatar_ref.set_visible(cx, true);
            } else {
                avatar_ref.set_visible(cx, false);
            }
        }

        // Build the typing notice text
        let display_names: Vec<&str> = typing_users.iter()
            .map(|u| u.display_name.as_str())
            .collect();
        let typing_notice_text = match display_names.as_slice() {
            [user] => format!("{user} is typing "),
            [user1, user2] => format!("{user1} and {user2} are typing "),
            [user1, user2, others @ ..] => {
                if others.len() == 1 {
                    format!("{user1}, {user2}, and {} are typing ", others[0])
                } else {
                    format!(
                        "{user1}, {user2}, and {} others are typing ",
                        others.len()
                    )
                }
            }
            _ => return, // Empty case handled above
        };

        // Set the typing notice text and make its view visible.
        self.view.label(cx, ids!(typing_label)).set_text(cx, &typing_notice_text);
        self.view.set_visible(cx, true);
        // Animate in the typing notice view (sliding it up from the bottom).
        self.animator_play(cx, ids!(typing_notice_animator.show));
        // Start the typing notice text animation of bouncing dots.
        self.view.bouncing_dots(cx, ids!(bouncing_dots)).start_animation(cx);
    }

    /// Sets the avatar for a typing user.
    fn set_typing_avatar(&self, cx: &mut Cx, avatar_ref: &AvatarRef, user: &TypingUser) {
        // Try to load avatar image if we have a URL
        if let Some(avatar_url) = &user.avatar_url {
            match avatar_cache::get_or_fetch_avatar(cx, avatar_url) {
                AvatarCacheEntry::Loaded(data) => {
                    let _ = avatar_ref.show_image(
                        cx,
                        None, // Not clickable in typing notice
                        |cx, img| utils::load_png_or_jpg(&img, cx, &data),
                    );
                    return;
                }
                AvatarCacheEntry::Failed | AvatarCacheEntry::Requested => {
                    // Fall through to show text avatar
                }
            }
        }
        // Show text avatar (first letter of display name)
        avatar_ref.show_text(cx, None, None, &user.display_name);
    }
}

impl TypingNoticeRef {
    /// Shows or hides the typing notice based on whether there are any typing users.
    pub fn show_or_hide(&self, cx: &mut Cx, typing_users: &[TypingUser]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.show_or_hide(cx, typing_users);
        }
    }
}
