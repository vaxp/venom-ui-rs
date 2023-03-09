//! Radio button widget - single selection from group
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::Radio;
//!
//! let radio = Radio::new("option1", &selected)
//!     .label("Option 1")
//!     .on_select(|| println!("Selected!"));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Point};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// RADIO
// ============================================================================

/// A radio button for single-selection groups
/// 
/// # Example
/// 
/// ```ignore
/// let option1 = Radio::new("light", &theme)
///     .label("Light theme")
///     .on_select(|| set_theme("light"));
/// 
/// let option2 = Radio::new("dark", &theme)
///     .label("Dark theme")
///     .on_select(|| set_theme("dark"));
/// ```
pub struct Radio<T: PartialEq + Clone + Send + Sync> {
    /// This radio's value
    value: T,
    /// Currently selected value (reference for comparison)
    selected: T,
    /// Label text
    label: Option<String>,
    /// Outer circle size
    size: f32,
    /// Spacing between radio and label
    spacing: f32,
    /// Active color
    active_color: Color,
    /// Inactive color
    inactive_color: Color,
    /// Dot color
    dot_color: Color,
    /// Callback when selected
    on_select: Option<Box<dyn Fn() + Send + Sync>>,
    /// Disabled state
    disabled: bool,
}

impl<T: PartialEq + Clone + Default + Send + Sync> Default for Radio<T> {
    fn default() -> Self {
        Self::new(T::default(), &T::default())
    }
}

impl<T: PartialEq + Clone + Send + Sync> Radio<T> {
    /// Create a new radio button
    /// 
    /// - `value`: This radio's value
    /// - `selected`: Currently selected value in the group
    pub fn new(value: T, selected: &T) -> Self {
        Self {
            value,
            selected: selected.clone(),
            label: None,
            size: 18.0,
            spacing: 8.0,
            active_color: Color::hex("#6366f1"),
            inactive_color: Color::hex("#374151"),
            dot_color: Color::WHITE,
            on_select: None,
            disabled: false,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set label text
    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Set radio size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(12.0);
        self
    }

    /// Set spacing between radio and label
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set active color
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set callback when selected
    pub fn on_select<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }

    /// Disable the radio button
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Check if this radio is currently selected
    pub fn is_selected(&self) -> bool {
        self.value == self.selected
    }

    /// Check if disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Get the value
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T: PartialEq + Clone + Send + Sync + 'static> Widget for Radio<T> {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let label_width = self.label.as_ref().map(|l| l.len() as f32 * 8.0).unwrap_or(0.0);
        let total_width = self.size + if label_width > 0.0 { self.spacing + label_width } else { 0.0 };
        
        Size::new(
            constraints.constrain_width(total_width),
            constraints.constrain_height(self.size),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let center = Point::new(
            offset.dx + self.size / 2.0,
            offset.dy + self.size / 2.0,
        );
        let radius = self.size / 2.0;

        // Colors based on state
        let (fill_color, border_color) = if self.disabled {
            (Color::hex("#1f2937"), Color::hex("#374151"))
        } else if self.is_selected() {
            (self.active_color, self.active_color)
        } else {
            (self.inactive_color, Color::hex("#6b7280"))
        };

        // Draw outer circle
        canvas.draw_circle(center, radius, &Paint::fill(fill_color));
        canvas.draw_circle(center, radius, &Paint::stroke(border_color, 1.5));

        // Draw inner dot if selected
        if self.is_selected() {
            let dot_color = if self.disabled {
                Color::hex("#6b7280")
            } else {
                self.dot_color
            };
            canvas.draw_circle(center, radius * 0.4, &Paint::fill(dot_color));
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
                Point::new(label_x, offset.dy + 2.0),
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
    fn test_radio_creation() {
        let selected = "option1";
        let radio = Radio::new("option1", &selected);
        
        assert!(radio.is_selected());
    }

    #[test]
    fn test_radio_not_selected() {
        let selected = "option1";
        let radio = Radio::new("option2", &selected);
        
        assert!(!radio.is_selected());
    }

    #[test]
    fn test_radio_builder() {
        let selected = 1;
        let radio = Radio::new(1, &selected)
            .label("Option 1")
            .size(24.0);
        
        assert!(radio.is_selected());
        assert!(radio.label.is_some());
    }
}
