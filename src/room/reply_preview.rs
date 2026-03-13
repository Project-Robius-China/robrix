//! Widgets that represent a preview of a message that was (or is being) replied to.
//!
//! The core view is private, `ReplyPreviewContent`, which is used by both of the public views
//! exported by this module: `RepliedToMessage` and `ReplyingPreview`.
//!
//! `RepliedToMessage` supports expand/collapse for long reply previews.

use makepad_widgets::*;

/// Maximum height for collapsed reply previews (in pixels).
/// Replies taller than this will show a "Show more" toggle.
const REPLY_PREVIEW_MAX_COLLAPSED_HEIGHT: f64 = 80.0;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::shared::styles::*;
    use crate::shared::helpers::*;
    use crate::shared::icon_button::*;
    use crate::shared::avatar::Avatar;
    use crate::shared::html_or_plaintext::*;

    ReplyPreviewContent = <View> {
        width: Fill
        height: Fit
        flow: Down
        padding: {left: 16.0, bottom: 5.0, top: 2.0, right: 11.0}
        cursor: Hand,

        <View> {
            width: Fill
            height: Fit
            flow: Right
            margin: { bottom: 10.0, top: 0.0, right: 5.0 }
            align: {y: 0.5}

            reply_preview_avatar = <Avatar> {
                width: 19.,
                height: 19.,
                text_view = { text = { draw_text: {
                    text_style: { font_size: 6.0 }
                }}}
            }

            reply_preview_username = <Label> {
                width: Fill,
                flow: Right, // do not wrap
                margin: { left: 5.0, top: 2 }
                draw_text: {
                    text_style: <USERNAME_TEXT_STYLE> { font_size: 10 },
                    color: (USERNAME_TEXT_COLOR)
                    wrap: Ellipsis,
                }
                text: "<Username not available>"
            }
        }

        reply_preview_body = <HtmlOrPlaintext> {
            margin: {left: 1.5}
            html_view = { html = {
                font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE)
                    draw_normal:      { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } },
                    draw_italic:      { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } },
                    draw_bold:        { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } },
                    draw_bold_italic: { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } },
                    draw_fixed:       { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } },
                    // a = { draw_text:  { text_style: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) } } },
            } }
            plaintext_view = { pt_label = {
                draw_text: {
                    text_style: <MESSAGE_TEXT_STYLE> { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) },
                }
            } }
        }
    }

    // A view that shows a preview of the message that the user is currently drafting a reply to,
    // along with a "Replying to" label and a cancel button.
    pub ReplyingPreview = <View> {
        visible: false
        width: Fill
        height: Fit
        flow: Down
        padding: { left: 9, right: 9 }

        // Displays a "Replying to" label and a cancel button
        // above the preview of the message being replied to.
        <View> {
            width: Fill
            height: Fit
            flow: Right
            align: {y: 0.5}
            padding: {left: 14, right: 6, top: 10, bottom: 0}

            <Label> {
                width: Fill,
                flow: Right, // do not wrap
                // Vertically align the text with the X icon in the cancel_reply_button
                padding: {top: 5}

                draw_text: {
                    text_style: <USERNAME_TEXT_STYLE> {},
                    color: #222,
                    wrap: Ellipsis,
                }
                text: "Replying to:"
            }

            cancel_reply_button = <RobrixIconButton> {
                width: Fit,
                height: Fit,
                padding: 13,
                spacing: 0,
                margin: {left: 5, right: 0},

                draw_bg: {
                    border_color: (COLOR_FG_DANGER_RED),
                    color: (COLOR_BG_DANGER_RED)
                    border_radius: 5
                }
                draw_icon: {
                    svg_file: (ICON_CLOSE),
                    color: (COLOR_FG_DANGER_RED)
                }
                icon_walk: {width: 16, height: 16, margin: 0}
            }
        }

        reply_preview_content = <ReplyPreviewContent> { }

        <LineH> {
            margin: {top: 4.0, left: 5, right: 5} //, bottom: 10}
        }
    }

    // A small inline preview of a message that was replied to by another message
    // within a room timeline.
    // That is, this view contains a preview of the earlier message
    // that is shown above the "in-reply-to" message.
    // Supports expand/collapse for long reply previews.
    pub RepliedToMessage = {{RepliedToMessage}} {
        visible: false
        width: Fill
        height: Fit
        flow: Down

        padding: {top: 0.0, right: 12.0, bottom: 0.0, left: 12.0}

        // Container that clips the content when collapsed
        content_container = <View> {
            width: Fill
            height: Fit
            flow: Down

            // A reply preview with a vertical bar drawn in the background.
            replied_to_message_content = <ReplyPreviewContent> {
                show_bg: true
                draw_bg: {
                    instance vertical_bar_color: (USERNAME_TEXT_COLOR)
                    instance vertical_bar_width: 2.0
                    instance border_radius: 0.0

                    fn get_color(self) -> vec4 {
                        return self.color;
                    }

                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);

                        sdf.box(
                            0.0,
                            0.0,
                            self.rect_size.x,
                            self.rect_size.y,
                            max(1.0, self.border_radius)
                        );
                        sdf.fill(self.get_color());

                        sdf.rect(
                            0.0,
                            0.0,
                            self.vertical_bar_width,
                            self.rect_size.y
                        );
                        sdf.fill(self.vertical_bar_color);

                        return sdf.result;
                    }
                }
            }
        }

        // Toggle button for expand/collapse (only shown when content exceeds max height)
        expand_toggle = <View> {
            visible: false
            width: Fill
            height: Fit
            align: {x: 0.0, y: 0.5}
            padding: {left: 16.0, top: 2.0, bottom: 2.0}

            toggle_label = <Label> {
                width: Fit
                height: Fit
                cursor: Hand
                draw_text: {
                    text_style: <REGULAR_TEXT> { font_size: 9 }
                    color: #0066cc
                }
                text: "Show more"
            }
        }
    }
}

/// Actions emitted by the RepliedToMessage widget.
#[derive(Clone, Debug, DefaultNone)]
pub enum RepliedToMessageAction {
    /// The user clicked on the expand/collapse toggle.
    ToggleExpand,
    None,
}

/// A preview of a message that was replied to, with expand/collapse support for long content.
#[derive(Live, LiveHook, Widget)]
pub struct RepliedToMessage {
    #[deref] view: View,
    /// Whether the reply preview is currently expanded.
    #[rust] is_expanded: bool,
    /// Whether this reply content needs expand/collapse (exceeds max height).
    #[rust] needs_expand_toggle: bool,
    /// The natural height of the content when fully expanded.
    #[rust] natural_height: f64,
}

impl Widget for RepliedToMessage {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Handle click on the expand toggle view
        if let Hit::FingerUp(fe) = event.hits(cx, self.view.view(ids!(expand_toggle)).area()) {
            if fe.is_over && fe.was_tap() {
                self.is_expanded = !self.is_expanded;
                self.update_expand_state(cx);
            }
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        let step = self.view.draw_walk(cx, scope, walk);

        // After drawing, check if we need to apply height constraints
        // Only do this once when natural_height is not yet measured
        if self.natural_height == 0.0 {
            let content_container = self.view.view(ids!(content_container));
            let content_rect = content_container.area().rect(cx);
            if content_rect.size.y > 0.0 {
                self.natural_height = content_rect.size.y;

                if self.natural_height > REPLY_PREVIEW_MAX_COLLAPSED_HEIGHT {
                    self.needs_expand_toggle = true;
                    self.update_expand_state(cx);
                }
            }
        }

        step
    }
}

impl RepliedToMessage {
    /// Updates the UI based on the current expand/collapse state.
    fn update_expand_state(&mut self, cx: &mut Cx) {
        let expand_toggle = self.view.view(ids!(expand_toggle));
        let toggle_label = self.view.label(ids!(expand_toggle.toggle_label));
        let content_container = self.view.view(ids!(content_container));

        expand_toggle.set_visible(cx, self.needs_expand_toggle);

        if self.needs_expand_toggle {
            if self.is_expanded {
                toggle_label.set_text(cx, "Show less");
                // Remove height constraint - use Fit without max
                content_container.apply_over(cx, live! {
                    height: Fit
                });
            } else {
                toggle_label.set_text(cx, "Show more");
                // Apply max height constraint using direct value
                content_container.apply_over(cx, live! {
                    height: 80.0
                });
            }
        }

        self.redraw(cx);
    }

    /// Resets the expand state (call when reusing the widget for a different reply).
    pub fn reset_expand_state(&mut self) {
        self.is_expanded = false;
        self.needs_expand_toggle = false;
        self.natural_height = 0.0;
    }
}

impl RepliedToMessageRef {
    /// See [`RepliedToMessage::reset_expand_state()`].
    pub fn reset_expand_state(&self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.reset_expand_state();
        }
    }
}
