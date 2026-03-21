//! A simple loading view displayed while generating thumbnails for file uploads.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    // A view that displays a loading spinner and text while generating thumbnails.
    mod.widgets.ThumbnailLoadingView = View {
        visible: false,
        width: Fill,
        height: Fit,
        padding: Inset{top: 8, bottom: 8, left: 10, right: 10}
        flow: Right,
        spacing: 10,
        align: Align{x: 0, y: 0.5}

        loading_spinner := LoadingSpinner {
            width: 25,
            height: 25,
            draw_bg +: {
                color: (COLOR_ACTIVE_PRIMARY)
                border_size: 3.0,
            }
        }

        loading_text := Label {
            width: Fit,
            height: Fit,
            draw_text +: {
                text_style: REGULAR_TEXT{font_size: 11},
                color: #666
            }
            text: "Generating thumbnail..."
        }
    }
}
