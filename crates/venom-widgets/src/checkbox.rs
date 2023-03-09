//! Checkbox widget - simple toggle control
//!
//! A clean, high-level checkbox with minimal boilerplate.
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::Checkbox;
//!
//! let checkbox = Checkbox::new(is_checked)
//!     .label("Accept terms")
//!     .on_change(|checked| println!("Now: {}", checked));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// CHECKBOX
// ============================================================================

/// A simple checkbox widget
/// 
/// # Example
/// 
/// ```ignore
/// // Basic checkbox
/// let cb = Checkbox::new(false);
/// 
/// // With label and callback
/// let cb = Checkbox::new(is_checked)
///     .label("Remember me")
///     .on_change(|new_value| {
///         // Handle change
///     });
/// 
/// // Styled checkbox
/// let cb = Checkbox::new(true)
///     .label("Dark mode")
///     .size(20.0)
///     .active_color(Color::hex("#6366f1"))
///     .disabled();
/// ```
pub struct Checkbox {
    /// Current checked state
    checked: bool,
    /// Optional label text
    label: Option<String>,
    /// Size of the checkbox box
    size: f32,
    /// Spacing between box and label
    spacing: f32,
    /// Color when checked
    active_color: Color,
    /// Color when unchecked
    inactive_color: Color,
    /// Check mark color
    check_color: Color,
    /// Border color
    border_color: Color,
    /// Border radius
    border_radius: f32,
    /// Callback when toggled
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
    /// Whether the checkbox is disabled
    disabled: bool,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Checkbox {
    /// Create a new checkbox with the given checked state
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            size: 18.0,
            spacing: 8.0,
            active_color: Color::hex("#6366f1"),
            inactive_color: Color::hex("#374151"),
            check_color: Color::WHITE,
            border_color: Color::hex("#6b7280"),
            border_radius: 4.0,
            on_change: None,
            disabled: false,
        }
    }

    /// Create a checked checkbox
    pub fn checked() -> Self {
        Self::new(true)
    }

    /// Create an unchecked checkbox
    pub fn unchecked() -> Self {
        Self::new(false)
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Set the label text
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set the checkbox size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(12.0);
        self
    }

    /// Set spacing between checkbox and label
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set active (checked) color
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set inactive (unchecked) color
    pub fn inactive_color(mut self, color: Color) -> Self {
        self.inactive_color = color;
        self
    }

    /// Set check mark color
    pub fn check_color(mut self, color: Color) -> Self {
        self.check_color = color;
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set the change callback
    pub fn on_change<F: Fn(bool) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Disable the checkbox
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Enable the checkbox
    pub fn enabled(mut self) -> Self {
        self.disabled = false;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get current checked state
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Get disabled state
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Toggle the checkbox (for internal use or testing)
    pub fn toggle(&mut self) {
        if !self.disabled {
            self.checked = !self.checked;
            if let Some(callback) = &self.on_change {
                callback(self.checked);
            }
        }
    }
}

impl Widget for Checkbox {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let label_width = self.label.as_ref().map(|l| l.len() as f32 * 8.0).unwrap_or(0.0);
        let total_width = self.size + if label_width > 0.0 { self.spacing + label_width } else { 0.0 };
        
        Size::new(
            constraints.constrain_width(total_width),
            constraints.constrain_height(self.size),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let box_rect = Rect::new(offset.dx, offset.dy, self.size, self.size);
        
        // Determine colors based on state
        let (bg_color, border_color) = if self.disabled {
            (Color::hex("#1f2937"), Color::hex("#374151"))
        } else if self.checked {
            (self.active_color, self.active_color)
        } else {
            (self.inactive_color, self.border_color)
        };

        // Draw box background
        canvas.draw_rounded_rect(
            box_rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(bg_color),
        );

        // Draw border
        canvas.draw_rounded_rect(
            box_rect,
            BorderRadius::all(self.border_radius),
            &Paint::stroke(border_color, 1.5),
        );

        // Draw checkmark if checked
        if self.checked {
            let check_color = if self.disabled {
                Color::hex("#6b7280")
            } else {
                self.check_color
            };

            // Draw checkmark as two lines
            let cx = offset.dx + self.size / 2.0;
            let cy = offset.dy + self.size / 2.0;
            let s = self.size * 0.25;

            // First line of check (going down-right)
            canvas.draw_line(
                venom_core::Point::new(cx - s, cy),
                venom_core::Point::new(cx - s * 0.3, cy + s * 0.7),
                &Paint::stroke(check_color, 2.0),
            );
            
            // Second line of check (going up-right)
            canvas.draw_line(
                venom_core::Point::new(cx - s * 0.3, cy + s * 0.7),
                venom_core::Point::new(cx + s, cy - s * 0.5),
                &Paint::stroke(check_color, 2.0),
            );
        }

        // Draw label
        if let Some(label) = &self.label {
            let label_x = offset.dx + self.size + self.spacing;
            let text_color = if self.disabled {
                Color::hex("#6b7280")
            } else {
                Color::WHITE
            };
            
            canvas.draw_text(
                label,
                venom_core::Point::new(label_x, offset.dy + 2.0),
                &Paint::fill(text_color),
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
    fn test_checkbox_creation() {
        let cb = Checkbox::new(false);
        assert!(!cb.is_checked());
        
        let cb = Checkbox::checked();
        assert!(cb.is_checked());
    }

    #[test]
    fn test_checkbox_builder() {
        let cb = Checkbox::new(true)
            .label("Test")
            .size(24.0)
            .active_color(Color::RED);
        
        assert!(cb.is_checked());
        assert!(cb.label.is_some());
        assert_eq!(cb.size, 24.0);
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut cb = Checkbox::new(false);
        assert!(!cb.is_checked());
        
        cb.toggle();
        assert!(cb.is_checked());
        
        cb.toggle();
        assert!(!cb.is_checked());
    }

    #[test]
    fn test_checkbox_disabled() {
        let mut cb = Checkbox::new(false).disabled();
        assert!(cb.is_disabled());
        
        cb.toggle(); // Should not toggle when disabled
        assert!(!cb.is_checked());
    }

    #[test]
    fn test_checkbox_layout() {
        let cb = Checkbox::new(false).size(20.0);
        let size = cb.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 20.0);
        assert_eq!(size.height, 20.0);
    }
}
