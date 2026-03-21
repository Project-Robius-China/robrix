//! Widgets that represent a preview of a message that was (or is being) replied to.
//!
//! The core view is private, `ReplyPreviewContent`, which is used by both of the public views
//! exported by this module: `RepliedToMessage` and `ReplyingPreview`.

use makepad_widgets::*;

/// Maximum initial height for reply previews in pixels before truncation occurs.
const REPLY_PREVIEW_MAX_HEIGHT: f64 = 100.0;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // Height limit constant for use in DSL
    mod.widgets.REPLY_PREVIEW_MAX_HEIGHT = 100.0

    mod.widgets.ReplyPreviewContent = View {
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{left: 16.0, bottom: 5.0, top: 2.0, right: 11.0}
        cursor: MouseCursor.Hand,

        View {
            width: Fill
            height: Fit
            flow: Right
            margin: Inset{ bottom: 10.0, top: 0.0, right: 5.0 }
            align: Align{y: 0.5}

            reply_preview_avatar := Avatar {
                width: 19.,
                height: 19.,
                text_view +: {
                    text +: {
                        draw_text +: {
                            text_style: theme.font_regular { font_size: 6.0 }
                        }
                    }
                }
            }

            reply_preview_username := Label {
                width: Fill,
                flow: Right, // do not wrap
                margin: Inset{ left: 5.0, top: 2 }
                draw_text +: {
                    text_style: USERNAME_TEXT_STYLE { font_size: 10 },
                    color: (USERNAME_TEXT_COLOR)
                    flow: Flow.Right{wrap: true},
                }
                text: "<Username not available>"
            }
        }

        reply_preview_body := HtmlOrPlaintext {
            margin: Inset{left: 1.5}
            html_view +: {
                html +: {
                    font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE)
                    text_style_normal +: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) }
                    text_style_italic +: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) }
                    text_style_bold +: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) }
                    text_style_bold_italic +: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) }
                    text_style_fixed +: { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) }
                }
            }
            plaintext_view +: {
                pt_label +: {
                    draw_text +: {
                        text_style: MESSAGE_TEXT_STYLE { font_size: (MESSAGE_REPLY_PREVIEW_FONT_SIZE) },
                    }
                }
            }
        }
    }

    // A view that shows a preview of the message that the user is currently drafting a reply to,
    // along with a "Replying to" label and a cancel button.
    mod.widgets.ReplyingPreview = View {
        visible: false
        width: Fill
        height: Fit
        flow: Down
        padding: Inset{ left: 9, right: 9 }

        // Displays a "Replying to" label and a cancel button
        // above the preview of the message being replied to.
        View {
            width: Fill
            height: Fit
            flow: Right
            align: Align{y: 0.5}
            padding: Inset{left: 14, right: 6, top: 10, bottom: 0}

            Label {
                width: Fill,
                flow: Right, // do not wrap
                // Vertically align the text with the X icon in the cancel_reply_button
                padding: Inset{top: 5}

                draw_text +: {
                    text_style: USERNAME_TEXT_STYLE {},
                    color: #222,
                    flow: Flow.Right{wrap: true},
                }
                text: "Replying to:"
            }

            cancel_reply_button := RobrixNegativeIconButton {
                width: Fit,
                height: Fit,
                padding: 13,
                spacing: 0,
                margin: Inset{left: 5, right: 0},
                draw_bg.border_radius: 5.0
                draw_icon.svg: (ICON_CLOSE)
                icon_walk: Walk{width: 16, height: 16, margin: 0}
            }
        }

        reply_preview_content := mod.widgets.ReplyPreviewContent { }

        LineH {
            margin: Inset{top: 4.0, left: 5, right: 5} //, bottom: 10}
        }
    }

    // A small inline preview of a message that was replied to by another message
    // within a room timeline.
    // That is, this view contains a preview of the earlier message
    // that is shown above the "in-reply-to" message.
    // This widget supports expand/collapse for long reply previews.
    mod.widgets.RepliedToMessage = set_type_default() do #(RepliedToMessage::register_widget(vm)) {
        visible: false
        width: Fill
        height: Fit
        flow: Down

        padding: Inset{top: 0.0, right: 12.0, bottom: 0.0, left: 12.0}

        // Clipping container that enforces max height when collapsed
        reply_content_clip := View {
            width: Fill
            height: Fit
            clip_overflow: true

            // A reply preview with a vertical bar drawn in the background.
            replied_to_message_content := mod.widgets.ReplyPreviewContent {
                show_bg: true
                draw_bg +: {
                    color: instance(COLOR_TRANSPARENT)
                    vertical_bar_color: instance(USERNAME_TEXT_COLOR)
                    vertical_bar_width: instance(2.0)
                    border_radius: instance(0.0)

                    pixel: fn() {
                        let sdf = Sdf2d.viewport(self.pos * self.rect_size);

                        sdf.box(
                            0.0,
                            0.0,
                            self.rect_size.x,
                            self.rect_size.y,
                            max(1.0, self.border_radius)
                        );
                        sdf.fill(self.color);

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

        // Expand/collapse toggle button (only visible when content exceeds max height)
        expand_collapse_button := View {
            visible: false
            width: Fill
            height: Fit
            flow: Right
            align: Align{x: 0.0, y: 0.5}
            margin: Inset{top: 2, left: 16}
            cursor: MouseCursor.Hand

            expand_arrow := mod.widgets.ExpandArrow {
                width: 14, height: 14,
                draw_bg +: {
                    color: #888
                }
            }

            expand_label := Label {
                width: Fit
                height: Fit
                margin: Inset{left: 4}
                draw_text +: {
                    text_style: theme.font_regular { font_size: 10 }
                    color: #888
                }
                text: "Show more"
            }
        }
    }
}

use crate::shared::expand_arrow::ExpandArrow;
use makepad_widgets::animator::Animate;

/// Action emitted when expand/collapse state changes on RepliedToMessage.
#[derive(Clone, Debug, Default)]
pub enum RepliedToMessageAction {
    /// The expand/collapse state was toggled.
    Toggled { is_expanded: bool },
    #[default]
    None,
}

/// A collapsible reply preview widget that shows a preview of an earlier message
/// that was replied to. Supports expand/collapse for long content.
#[derive(Script, ScriptHook, Widget)]
pub struct RepliedToMessage {
    #[deref] view: View,
    /// Whether the content is currently expanded (showing full content).
    #[rust(false)] is_expanded: bool,
    /// Whether the content exceeds the max height and needs expand/collapse controls.
    #[rust(false)] needs_truncation: bool,
    /// The actual content height measured during draw.
    #[rust(0.0)] content_height: f64,
}

impl Widget for RepliedToMessage {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        // Handle click on the expand/collapse button area
        match event.hits(cx, self.view.view(cx, ids!(expand_collapse_button)).area()) {
            Hit::FingerUp(fe) if fe.is_over && fe.was_tap() => {
                self.toggle_expand_collapse(cx);
                cx.widget_action(
                    self.widget_uid(),
                    RepliedToMessageAction::Toggled { is_expanded: self.is_expanded },
                );
            }
            _ => {}
        }

        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // First, measure the content height from the previous draw
        let content_view = self.view.view(cx, ids!(reply_content_clip.replied_to_message_content));
        let rect = content_view.area().rect(cx);
        self.content_height = rect.size.y;

        // Determine if we need truncation based on content height
        self.needs_truncation = self.content_height > REPLY_PREVIEW_MAX_HEIGHT;

        // Update expand/collapse button visibility
        let button = self.view.view(cx, ids!(expand_collapse_button));
        button.set_visible(cx, self.needs_truncation);

        // Update button label and arrow state
        if self.needs_truncation {
            let label = self.view.label(cx, ids!(expand_collapse_button.expand_label));
            label.set_text(cx, if self.is_expanded { "Show less" } else { "Show more" });

            if let Some(mut arrow) = self.view.child_by_path(ids!(expand_collapse_button.expand_arrow)).borrow_mut::<ExpandArrow>() {
                arrow.set_is_open_no_animate(self.is_expanded);
            }
        }

        // Apply height constraint based on expand state using script_apply_eval
        if !self.is_expanded && self.needs_truncation {
            // When collapsed and needs truncation, clip to max height
            let mut clip_view = self.view.view(cx, ids!(reply_content_clip));
            script_apply_eval!(cx, clip_view, {
                height: (REPLY_PREVIEW_MAX_HEIGHT)
            });
        } else {
            // When expanded or content fits, use natural height
            let mut clip_view = self.view.view(cx, ids!(reply_content_clip));
            script_apply_eval!(cx, clip_view, {
                height: Fit
            });
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl RepliedToMessage {
    /// Toggle between expanded and collapsed states with animation.
    fn toggle_expand_collapse(&mut self, cx: &mut Cx) {
        self.is_expanded = !self.is_expanded;

        // Animate the arrow
        if let Some(mut arrow) = self.view.child_by_path(ids!(expand_collapse_button.expand_arrow)).borrow_mut::<ExpandArrow>() {
            arrow.set_is_open(cx, self.is_expanded, Animate::Yes);
        }

        self.view.redraw(cx);
    }
}

impl RepliedToMessageRef {
    /// Sets the expanded state of the reply preview.
    pub fn set_expanded(&self, cx: &mut Cx, is_expanded: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            if inner.is_expanded != is_expanded {
                inner.is_expanded = is_expanded;
                inner.view.redraw(cx);
            }
        }
    }

    /// Returns whether the reply preview is currently expanded.
    pub fn is_expanded(&self) -> bool {
        self.borrow().map(|inner| inner.is_expanded).unwrap_or(false)
    }

    /// Returns whether the content needs truncation (exceeds max height).
    pub fn needs_truncation(&self) -> bool {
        self.borrow().map(|inner| inner.needs_truncation).unwrap_or(false)
    }
}