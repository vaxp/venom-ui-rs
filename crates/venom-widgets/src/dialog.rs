//! Dialog and Modal widgets
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Dialog, AlertDialog};
//!
//! let dialog = Dialog::new()
//!     .title("Confirm")
//!     .content("Are you sure?");
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// DIALOG
// ============================================================================

/// A dialog/modal overlay
/// 
/// # Example
/// 
/// ```ignore
/// let confirm = Dialog::new()
///     .title("Delete Item?")
///     .content("This action cannot be undone.")
///     .action("Cancel", || close())
///     .action("Delete", || delete_item());
/// ```
pub struct Dialog {
    /// Dialog title
    title: Option<String>,
    /// Dialog content/message
    content: Option<String>,
    /// Dialog width
    width: f32,
    /// Background color
    background: Color,
    /// Border radius
    border_radius: f32,
    /// Overlay color
    overlay_color: Color,
    /// Actions (label, callback)
    actions: Vec<(String, Box<dyn Fn() + Send + Sync>)>,
    /// Is visible
    visible: bool,
    /// Close callback
    on_close: Option<Box<dyn Fn() + Send + Sync>>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new()
    }
}

impl Dialog {
    /// Create a new dialog
    pub fn new() -> Self {
        Self {
            title: None,
            content: None,
            width: 320.0,
            background: Color::hex("#1f2937"),
            border_radius: 16.0,
            overlay_color: Color::rgba(0, 0, 0, 180),
            actions: Vec::new(),
            visible: true,
            on_close: None,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set dialog title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set dialog content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set dialog width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(200.0);
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set overlay color
    pub fn overlay_color(mut self, color: Color) -> Self {
        self.overlay_color = color;
        self
    }

    /// Add an action button
    pub fn action<F: Fn() + Send + Sync + 'static>(mut self, label: impl Into<String>, callback: F) -> Self {
        self.actions.push((label.into(), Box::new(callback)));
        self
    }

    /// Set close callback
    pub fn on_close<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_close = Some(Box::new(callback));
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    // ========================================================================
    // CONTROL
    // ========================================================================

    /// Show the dialog
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the dialog
    pub fn hide(&mut self) {
        self.visible = false;
        if let Some(callback) = &self.on_close {
            callback();
        }
    }

    /// Check if visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Widget for Dialog {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        // Dialog takes full screen for overlay
        Size::new(constraints.max_width, constraints.max_height)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if !self.visible {
            return;
        }

        // Draw overlay
        let screen_rect = Rect::new(offset.dx, offset.dy, 800.0, 600.0); // TODO: get actual size
        canvas.draw_rect(screen_rect, &Paint::fill(self.overlay_color));

        // Calculate dialog position (centered)
        let content_height = self.calculate_height();
        let dialog_x = (800.0 - self.width) / 2.0 + offset.dx;
        let dialog_y = (600.0 - content_height) / 2.0 + offset.dy;

        // Draw dialog background
        let dialog_rect = Rect::new(dialog_x, dialog_y, self.width, content_height);
        canvas.draw_rounded_rect(
            dialog_rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(self.background),
        );

        // Draw border
        canvas.draw_rounded_rect(
            dialog_rect,
            BorderRadius::all(self.border_radius),
            &Paint::stroke(Color::hex("#374151"), 1.0),
        );

        let padding = 24.0;
        let mut y = dialog_y + padding;

        // Draw title
        if let Some(title) = &self.title {
            canvas.draw_text(
                title,
                venom_core::Point::new(dialog_x + padding, y),
                &Paint::fill(Color::WHITE),
                18.0,
            );
            y += 28.0;
        }

        // Draw content
        if let Some(content) = &self.content {
            canvas.draw_text(
                content,
                venom_core::Point::new(dialog_x + padding, y),
                &Paint::fill(Color::hex("#9ca3af")),
                14.0,
            );
            y += 24.0;
        }

        // Draw action buttons
        if !self.actions.is_empty() {
            y += 16.0;
            let button_spacing = 12.0;
            let button_height = 36.0;
            let button_width = (self.width - padding * 2.0 - button_spacing * (self.actions.len() - 1) as f32) / self.actions.len() as f32;
            
            let mut button_x = dialog_x + padding;
            for (i, (label, _)) in self.actions.iter().enumerate() {
                // Button background
                let is_primary = i == self.actions.len() - 1;
                let bg_color = if is_primary {
                    Color::hex("#6366f1")
                } else {
                    Color::hex("#374151")
                };
                
                let button_rect = Rect::new(button_x, y, button_width, button_height);
                canvas.draw_rounded_rect(
                    button_rect,
                    BorderRadius::all(8.0),
                    &Paint::fill(bg_color),
                );

                // Button text
                canvas.draw_text(
                    label,
                    venom_core::Point::new(button_x + button_width / 2.0 - label.len() as f32 * 3.5, y + 10.0),
                    &Paint::fill(Color::WHITE),
                    14.0,
                );

                button_x += button_width + button_spacing;
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Dialog {
    fn calculate_height(&self) -> f32 {
        let padding = 24.0;
        let mut height: f32 = padding * 2.0;
        
        if self.title.is_some() {
            height += 28.0;
        }
        if self.content.is_some() {
            height += 24.0;
        }
        if !self.actions.is_empty() {
            height += 16.0 + 36.0; // spacing + button height
        }
        
        height.max(120.0)
    }
}

// ============================================================================
// SNACKBAR
// ============================================================================

/// A brief message shown at the bottom of the screen
/// 
/// # Example
/// 
/// ```ignore
/// let snack = Snackbar::new("Item deleted")
///     .action("Undo", || undo())
///     .duration_seconds(3);
/// ```
pub struct Snackbar {
    /// Message text
    message: String,
    /// Action label and callback
    action: Option<(String, Box<dyn Fn() + Send + Sync>)>,
    /// Background color
    background: Color,
    /// Is visible
    visible: bool,
}

impl Default for Snackbar {
    fn default() -> Self {
        Self::new("")
    }
}

impl Snackbar {
    /// Create a new snackbar
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            background: Color::hex("#323232"),
            visible: true,
        }
    }

    /// Add an action button
    pub fn action<F: Fn() + Send + Sync + 'static>(mut self, label: impl Into<String>, callback: F) -> Self {
        self.action = Some((label.into(), Box::new(callback)));
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }


    /// Show the snackbar
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the snackbar
    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Widget for Snackbar {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(constraints.max_width, 56.0)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if !self.visible {
            return;
        }

        let width = 800.0; // TODO: get actual width
        let height = 48.0;
        let margin = 16.0;
        
        // Position at bottom
        let x = margin;
        let y = offset.dy;
        let bar_width = width - margin * 2.0;

        // Background
        let rect = Rect::new(x, y, bar_width, height);
        canvas.draw_rounded_rect(
            rect,
            BorderRadius::all(8.0),
            &Paint::fill(self.background),
        );

        // Message
        canvas.draw_text(
            &self.message,
            venom_core::Point::new(x + 16.0, y + 16.0),
            &Paint::fill(Color::WHITE),
            14.0,
        );

        // Action button
        if let Some((label, _)) = &self.action {
            let action_x = x + bar_width - 16.0 - label.len() as f32 * 8.0;
            canvas.draw_text(
                label,
                venom_core::Point::new(action_x, y + 16.0),
                &Paint::fill(Color::hex("#bb86fc")),
                14.0,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_creation() {
        let dialog = Dialog::new()
            .title("Test")
            .content("Content");
        
        assert!(dialog.title.is_some());
        assert!(dialog.is_visible());
    }

    #[test]
    fn test_dialog_visibility() {
        let mut dialog = Dialog::new();
        assert!(dialog.is_visible());
        
        dialog.hide();
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_snackbar() {
        let snack = Snackbar::new("Message");
        assert!(snack.visible);
    }
}
