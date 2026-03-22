//! A file previewer modal widget that displays file metadata and previews.
//!
//! This widget handles FilePreviewerAction to show and hide the previewer modal.
//! When the user confirms the upload, it sends a `TimelineUpdate::FileUploadConfirmed`
//! through the timeline-specific channel to ensure the upload is associated with
//! the correct room/timeline.

use makepad_widgets::*;
use makepad_widgets::image_cache::{ImageBuffer, ImageError};
use matrix_sdk::attachment::Thumbnail;

use crate::home::room_screen::TimelineUpdate;
use crate::image_utils::ImageDimensions;

/// Decodes image data into an `ImageBuffer` for rendering.
///
/// Supports PNG and JPEG formats only. Other formats will return an error.
///
/// # Errors
/// Returns `ImageError::UnsupportedFormat` if the image format is not PNG or JPEG,
/// or if the format cannot be detected from the data.
fn load_image_from_bytes(data: &[u8]) -> Result<ImageBuffer, ImageError> {
    match imghdr::from_bytes(data) {
        Some(imghdr::Type::Png) => ImageBuffer::from_png(data),
        Some(imghdr::Type::Jpeg) => ImageBuffer::from_jpg(data),
        Some(_) | None => Err(ImageError::UnsupportedFormat),
    }
}

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    FILE_UPLOAD_MODAL_BORDER_RADIUS: 6.0

    mod.widgets.FileUploadModal = set_type_default() do #(FileUploadModal::register_widget(vm)) {
        ..mod.widgets.RoundedView

        width: Fill { max: 1000 }
        height: Fit
        align: Align{x: 0.5, y: 0.5}
        margin: 40,

        flow: Down
        padding: Inset{top: 20, right: 25, bottom: 20, left: 25}

        show_bg: true
        draw_bg +: {
            color: (COLOR_PRIMARY)
            border_radius: (FILE_UPLOAD_MODAL_BORDER_RADIUS)
            border_size: 0.0
        }
        // Make this a ScrollYView
        scroll_bars: ScrollBars {
            show_scroll_x: false, show_scroll_y: true,
            scroll_bar_y: {drag_scrolling: true}
        }
        // Title and close button
        title_view := View {
            width: Fill, height: Fit,
            flow: Right,
            align: Align{x: 0, y: 0.5}

            title := Label {
                width: Fill, height: Fit,
                draw_text +: {
                    text_style: TITLE_TEXT{font_size: 16},
                    color: #000
                }
                text: "Upload File"
            }
        }

        // File metadata section
        metadata_view := View {
            width: Fill, height: Fit,
            flow: Right,
            align: Align{x: 0, y: 0.5}
            margin: Inset{top: 10, bottom: 10}

            // Document icon (visible only for non-image files)
            document_view := View {
                visible: false,
                width: Fit, height: Fit,
                align: Align{x: 0.5, y: 0.5}
                margin: Inset{right: 10}

                file_icon := Icon {
                    draw_icon +: {
                        svg_file: (ICON_FILE),
                        color: #999,
                    }
                    icon_walk: Walk{width: 24, height: 24}
                }
            }

            filename_text := Label {
                width: Fill, height: Fit,
                draw_text +: {
                    text_style: REGULAR_TEXT{font_size: 13},
                    color: (COLOR_TEXT),
                    wrap: Word
                }
            }
        }

        // Image preview (visible only for image files)
        image_view := View {
            width: Fill, height: Fit { max: 400 },
            flow: Down,
            align: Align{x: 0.5, y: 0.5}
            margin: Inset{top: 5, bottom: 5}

            preview_image := Image {
                width: Fill, height: 300,
                fit: ImageFit.Smallest,
            }
        }

        // Action buttons
        buttons_view := View {
            width: Fill, height: Fit,
            flow: Right,
            margin: Inset{top: 15}
            align: Align{x: 0.5, y: 0.5}
            spacing: 20

            cancel_button := RobrixIconButton {
                width: 100,
                align: Align{x: 0.5, y: 0.5}
                padding: 15,
                icon_walk: Walk{width: 0, height: 0, margin: 0}

                draw_bg +: {
                    border_size: 1.0
                    border_color: (COLOR_BG_DISABLED),
                    color: (COLOR_SECONDARY)
                }
                draw_text +: {
                    color: (COLOR_TEXT),
                }
                text: "Cancel"
            }

            upload_button := RobrixIconButton {
                width: 100
                align: Align{x: 0.5, y: 0.5}
                padding: 15,
                icon_walk: Walk{width: 0, height: 0, margin: 0}

                draw_bg +: {
                    border_size: 1.0
                    border_color: (COLOR_ACTIVE_PRIMARY_DARKER),
                    color: (COLOR_ACTIVE_PRIMARY)
                }
                draw_text +: {
                    color: (COLOR_PRIMARY),
                }
                text: "Upload"
            }
        }
    }
}

/// Actions emitted by the `RoomInputBar` widget.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Default)]
pub enum FilePreviewerAction {
    /// Display the FileUploadModal widget with the given file data.
    /// The file data includes the timeline update sender for sending confirmation.
    Show(FileData),
    /// Hide the FileUploadModal widget.
    Hide,
    /// No action.
    #[default]
    None,
}

/// Data for a file to be uploaded, including metadata, optional thumbnail,
/// and the timeline update sender to associate the upload with a specific timeline.
pub struct FileData {
    /// Metadata about the file (path, size, MIME type).
    pub metadata: FilePreviewerMetaData,
    /// Optional thumbnail for image files.
    pub thumbnail: Option<Thumbnail>,
    /// Optional dimensions for image/video files, width and height in pixels.
    pub dimensions: Option<ImageDimensions>,
    /// The sender to notify the timeline when upload is confirmed.
    pub timeline_update_sender: crossbeam_channel::Sender<TimelineUpdate>,
}

impl Clone for FileData {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            thumbnail: self.thumbnail.as_ref().map(|t| Thumbnail {
                data: t.data.clone(),
                content_type: t.content_type.clone(),
                height: t.height,
                width: t.width,
                size: t.size,
            }),
            dimensions: self.dimensions,
            timeline_update_sender: self.timeline_update_sender.clone(),
        }
    }
}

impl std::fmt::Debug for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileData")
            .field("metadata", &self.metadata)
            .field("thumbnail", &self.thumbnail.as_ref().map(|_| "..."))
            .field("dimensions", &self.dimensions)
            .field("timeline_update_sender", &"<channel>")
            .finish()
    }
}

impl FileData {
    /// Creates a new FileData by combining loaded file info with a timeline sender.
    pub fn new(
        loaded: FileLoadedData,
        timeline_update_sender: crossbeam_channel::Sender<TimelineUpdate>,
    ) -> Self {
        Self {
            metadata: loaded.metadata,
            thumbnail: loaded.thumbnail,
            dimensions: loaded.dimensions,
            timeline_update_sender,
        }
    }
}

/// Data loaded from a file by a background thread.
/// This is sent through a channel and combined with a timeline sender to create `FileData`.
#[derive(Debug)]
pub struct FileLoadedData {
    /// Metadata about the file (path, size, MIME type).
    pub metadata: FilePreviewerMetaData,
    /// Optional thumbnail for image files.
    pub thumbnail: Option<Thumbnail>,
    /// Optional dimensions for image/video files, width and height in pixels.
    pub dimensions: Option<ImageDimensions>,
}

impl Clone for FileLoadedData {
    fn clone(&self) -> Self {
        Self {
            metadata: self.metadata.clone(),
            thumbnail: self.thumbnail.as_ref().map(|t| Thumbnail {
                data: t.data.clone(),
                content_type: t.content_type.clone(),
                height: t.height,
                width: t.width,
                size: t.size,
            }),
            dimensions: self.dimensions,
        }
    }
}

/// Type alias for the receiver that gets loaded file data from a background thread.
pub type FileLoadReceiver = std::sync::mpsc::Receiver<Option<FileLoadedData>>;

/// Metadata about a file being previewed or uploaded.
#[derive(Clone, Debug)]
pub struct FilePreviewerMetaData {
    /// MIME type of the file.
    pub mime: mime_guess::mime::Mime,
    /// Size of the file in bytes.
    pub file_size: u64,
    /// Path to the file on the local filesystem.
    pub file_path: std::path::PathBuf,
}

/// Supported file types for preview.
#[derive(Clone, Debug, Default)]
enum FileType {
    /// An image file that can be previewed.
    Image,
    /// A document or other file type.
    #[default]
    Document,
}

/// A widget that previews files by displaying metadata and content based on file type.
#[derive(Script, ScriptHook, Widget)]
pub struct FileUploadModal {
    #[source]
    source: ScriptObjectRef,
    #[redraw]
    #[deref]
    view: View,
    #[rust]
    file_type: FileType,
    #[rust]
    file_data: Option<FileData>,
}

impl Widget for FileUploadModal {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.widget_match_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl WidgetMatchEvent for FileUploadModal {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions, _scope: &mut Scope) {
        // Handle FilePreviewerAction::Show to display the modal
        for action in actions.iter() {
            if let Some(FilePreviewerAction::Show(file_data)) = action.downcast_ref() {
                self.show(cx, file_data.clone());
            }
        }

        // Handle cancel button click
        if self.button(cx, ids!(cancel_button)).clicked(actions) {
            self.hide(cx);
            cx.action(FilePreviewerAction::Hide);
        }

        // Handle upload button click
        if self.button(cx, ids!(upload_button)).clicked(actions) {
            if let Some(file_data) = self.file_data.take() {
                // Send the file upload confirmation through the timeline-specific channel
                let _ = file_data.timeline_update_sender.send(TimelineUpdate::FileUploadConfirmed(file_data.clone()));
                SignalToUI::set_ui_signal();
            }
            self.hide(cx);
            cx.action(FilePreviewerAction::Hide);
        }
    }
}

impl FileUploadModal {
    /// Shows the file upload modal with the given file data.
    fn show(&mut self, cx: &mut Cx, file_data: FileData) {
        // Determine file type and update UI accordingly
        let is_image = file_data.metadata.mime.type_() == mime_guess::mime::IMAGE;
        self.file_type = if is_image { FileType::Image } else { FileType::Document };

        // Update filename display
        let filename = file_data.metadata.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown file");
        let file_size_str = format_file_size(file_data.metadata.file_size);
        self.label(cx, ids!(filename_text))
            .set_text(cx, &format!("{} ({})", filename, file_size_str));

        // Show/hide appropriate views based on file type
        match self.file_type {
            FileType::Image => {
                self.view(cx, ids!(document_view)).set_visible(cx, false);
                self.view(cx, ids!(image_view)).set_visible(cx, true);

                // Load image preview from thumbnail if available
                if let Some(ref thumbnail) = file_data.thumbnail {
                    if let Ok(image_buffer) = load_image_from_bytes(&thumbnail.data) {
                        let texture = image_buffer.into_new_texture(cx);
                        self.image(cx, ids!(preview_image)).set_texture(cx, Some(texture));
                    }
                }
            }
            FileType::Document => {
                self.view(cx, ids!(document_view)).set_visible(cx, true);
                self.view(cx, ids!(image_view)).set_visible(cx, false);
            }
        }

        self.file_data = Some(file_data);
        self.redraw(cx);
    }

    /// Hides the file upload modal and clears its state.
    fn hide(&mut self, cx: &mut Cx) {
        self.file_data = None;
        self.file_type = FileType::default();
        self.redraw(cx);
    }
}

/// Formats a file size in bytes to a human-readable string.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
