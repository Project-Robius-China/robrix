//! Widgets that represent a preview of a message that was (or is being) replied to.
//!
//! The core view is private, `ReplyPreviewContent`, which is used by both of the public views
//! exported by this module: `RepliedToMessage` and `ReplyingPreview`.
//!
//! `RepliedToMessage` also supports expandable/collapsible behavior for long reply previews.

use makepad_widgets::*;

/// Maximum height for a collapsed reply preview in pixels.
const REPLY_PREVIEW_MAX_HEIGHT: f64 = 150.0;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*


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
    //
    // Supports expandable/collapsible behavior for long reply previews.
    mod.widgets.RepliedToMessage = #(RepliedToMessage::register_widget(vm)) {
        visible: false
        width: Fill
        height: Fit
        flow: Down

        padding: Inset{top: 0.0, right: 12.0, bottom: 0.0, left: 12.0}

        // Container with clipped content that respects max height when collapsed
        content_container := View {
            width: Fill
            height: Fit
            flow: Down

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

        // Expand/collapse toggle button - only visible when content is truncated
        expand_toggle := View {
            visible: false
            width: Fill
            height: Fit
            flow: Right
            align: Align{x: 0.0, y: 0.5}
            padding: Inset{left: 16, top: 2, bottom: 2}
            cursor: MouseCursor.Hand

            expand_label := Label {
                width: Fit
                height: Fit
                draw_text +: {
                    text_style: TEXT_SUB { font_size: 9 }
                    color: #1a73e8
                }
                text: "Show more"
            }
        }
    }
}

/// A collapsible/expandable reply preview widget.
#[derive(Script, ScriptHook, Widget)]
pub struct RepliedToMessage {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,
    /// Whether the preview is currently expanded
    #[rust] is_expanded: bool,
    /// Whether the content exceeds the max height and needs truncation
    #[rust] needs_truncation: bool,
    /// The measured content height (updated during draw)
    #[rust] content_height: f64,
}

impl Widget for RepliedToMessage {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if !self.visible { return; }

        // Handle click on expand toggle
        let expand_toggle_area = self.view(cx, ids!(expand_toggle)).area();
        match event.hits(cx, expand_toggle_area) {
            Hit::FingerUp(fe) if fe.is_over && fe.was_tap() => {
                self.is_expanded = !self.is_expanded;
                // Update label text
                let label_text = if self.is_expanded { "Show less" } else { "Show more" };
                self.label(cx, ids!(expand_label)).set_text(cx, label_text);
                self.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // First, measure the actual content height by doing a layout pass
        let content_container = self.view(cx, ids!(content_container));
        let content_rect = content_container.area().rect(cx);

        // Check if we've measured content yet and it exceeds max height
        if content_rect.size.y > 0.0 {
            self.content_height = content_rect.size.y;
            self.needs_truncation = self.content_height > REPLY_PREVIEW_MAX_HEIGHT;
        }

        // Apply height constraint if collapsed and needs truncation
        if self.needs_truncation && !self.is_expanded {
            let mut content_container = self.view(cx, ids!(content_container));
            script_apply_eval!(cx, content_container, {
                height: 150.0  // Use fixed max height when collapsed
            });
        } else {
            let mut content_container = self.view(cx, ids!(content_container));
            script_apply_eval!(cx, content_container, {
                height: Fit
            });
        }

        // Show/hide the expand toggle based on whether truncation is needed
        self.view(cx, ids!(expand_toggle)).set_visible(cx, self.needs_truncation);

        self.view.draw_walk(cx, scope, walk)
    }
}

impl RepliedToMessage {
    /// Resets the expand state (should be called when new content is set)
    pub fn reset_expand_state(&mut self) {
        self.is_expanded = false;
        self.needs_truncation = false;
        self.content_height = 0.0;
    }
}

impl RepliedToMessageRef {
    /// Resets the expand state
    pub fn reset_expand_state(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.reset_expand_state();
            inner.label(cx, ids!(expand_label)).set_text(cx, "Show more");
        }
    }
}
