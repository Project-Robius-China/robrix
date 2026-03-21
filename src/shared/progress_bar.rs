//! A horizontal progress bar widget with a capsule-shaped design.
//!
//! Displays progress as a percentage (0-100) with a visual fill.

use makepad_widgets::*;

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    PROGRESS_BAR_HEIGHT: 8.0
    PROGRESS_BAR_RADIUS: 4.0

    // A horizontal progress bar with rounded ends (capsule shape).
    // Displays progress from 0% to 100% with a colored fill.
    mod.widgets.ProgressBar = set_type_default() do #(ProgressBar::register_widget(vm)) {
        ..mod.widgets.View

        width: Fill,
        height: (PROGRESS_BAR_HEIGHT),
        flow: Overlay,

        // Track (background)
        track := RoundedView {
            width: Fill,
            height: Fill,
            show_bg: true,
            draw_bg +: {
                color: #e2e8f0,
                border_radius: (PROGRESS_BAR_RADIUS),
            }
        }

        // Fill (foreground progress)
        fill := RoundedView {
            width: 0,
            height: Fill,
            show_bg: true,
            draw_bg +: {
                color: #3b82f6,
                border_radius: (PROGRESS_BAR_RADIUS),
            }
        }
    }
}

/// A horizontal progress bar widget that displays a percentage value (0-100).
#[derive(Script, ScriptHook, Widget)]
pub struct ProgressBar {
    #[source]
    source: ScriptObjectRef,

    #[deref]
    #[redraw]
    view: View,

    #[walk]
    walk: Walk,

    #[rust(0.0)]
    value: f64,
}

impl Widget for ProgressBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update the fill width before drawing
        let normalized_progress = (self.value / 100.0).clamp(0.0, 1.0);
        let fill_width = (normalized_progress * 100.0) as f64;

        // Set the fill width using script_apply_eval
        let mut fill_view = self.view(cx, ids!(fill));
        script_apply_eval!(cx, fill_view, {
            width: #(fill_width)
        });

        self.view.draw_walk(cx, scope, walk)
    }
}

impl ProgressBar {
    /// Returns the current progress value as a percentage (0.0-100.0).
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sets the progress value as a percentage (0.0-100.0).
    /// Values outside this range will be clamped.
    pub fn set_value(&mut self, cx: &mut Cx, value: f64) {
        self.value = value.clamp(0.0, 100.0);
        self.redraw(cx);
    }
}

impl ProgressBarRef {
    /// Returns the current progress value as a percentage (0.0-100.0).
    /// Returns 0.0 if the widget reference is invalid.
    pub fn value(&self) -> f64 {
        if let Some(inner) = self.borrow() {
            inner.value
        } else {
            0.0
        }
    }

    /// Sets the progress value as a percentage (0.0-100.0).
    /// Values outside this range will be clamped.
    pub fn set_value(&self, cx: &mut Cx, value: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_value(cx, value);
        }
    }
}
