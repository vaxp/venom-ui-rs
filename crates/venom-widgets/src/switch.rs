//! Switch/Toggle widget - on/off control
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::Switch;
//!
//! let switch = Switch::new(is_on)
//!     .on_change(|value| println!("Now: {}", value));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Point, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// SWITCH
// ============================================================================

/// A toggle switch for boolean values
/// 
/// # Example
/// 
/// ```ignore
/// let notifications = Switch::new(enabled)
///     .active_color(Color::hex("#22c55e"))
///     .on_change(|enabled| {
///         settings.notifications = enabled;
///     });
/// ```
pub struct Switch {
    /// Current on/off state
    value: bool,
    /// Track width
    width: f32,
    /// Track height
    height: f32,
    /// Active (on) color
    active_color: Color,
    /// Inactive (off) color
    inactive_color: Color,
    /// Thumb color
    thumb_color: Color,
    /// Callback when toggled
    on_change: Option<Box<dyn Fn(bool) + Send + Sync>>,
    /// Disabled state
    disabled: bool,
}

impl Default for Switch {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Switch {
    /// Create a new switch with the given value
    pub fn new(value: bool) -> Self {
        Self {
            value,
            width: 44.0,
            height: 24.0,
            active_color: Color::hex("#22c55e"),
            inactive_color: Color::hex("#374151"),
            thumb_color: Color::WHITE,
            on_change: None,
            disabled: false,
        }
    }

    /// Create an "on" switch
    pub fn on() -> Self {
        Self::new(true)
    }

    /// Create an "off" switch
    pub fn off() -> Self {
        Self::new(false)
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set switch dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width.max(30.0);
        self.height = height.max(16.0);
        self
    }

    /// Set active (on) color
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set inactive (off) color
    pub fn inactive_color(mut self, color: Color) -> Self {
        self.inactive_color = color;
        self
    }

    /// Set thumb color
    pub fn thumb_color(mut self, color: Color) -> Self {
        self.thumb_color = color;
        self
    }

    /// Set change callback
    pub fn on_change<F: Fn(bool) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Disable the switch
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get current value
    pub fn is_on(&self) -> bool {
        self.value
    }

    /// Check if disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Toggle the switch
    pub fn toggle(&mut self) {
        if !self.disabled {
            self.value = !self.value;
            if let Some(callback) = &self.on_change {
                callback(self.value);
            }
        }
    }
}

impl Widget for Switch {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.constrain_width(self.width),
            constraints.constrain_height(self.height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let track_rect = Rect::new(offset.dx, offset.dy, self.width, self.height);
        let radius = self.height / 2.0;

        // Track color
        let track_color = if self.disabled {
            Color::hex("#1f2937")
        } else if self.value {
            self.active_color
        } else {
            self.inactive_color
        };

        // Draw track
        canvas.draw_rounded_rect(
            track_rect,
            BorderRadius::all(radius),
            &Paint::fill(track_color),
        );

        // Thumb position
        let thumb_padding = 2.0;
        let thumb_radius = radius - thumb_padding;
        let thumb_x = if self.value {
            offset.dx + self.width - radius
        } else {
            offset.dx + radius
        };
        let thumb_y = offset.dy + radius;

        // Thumb color
        let thumb_color = if self.disabled {
            Color::hex("#6b7280")
        } else {
            self.thumb_color
        };

        // Draw thumb
        canvas.draw_circle(
            Point::new(thumb_x, thumb_y),
            thumb_radius,
            &Paint::fill(thumb_color),
        );
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
    fn test_switch_creation() {
        let sw = Switch::new(false);
        assert!(!sw.is_on());
        
        let sw = Switch::on();
        assert!(sw.is_on());
    }

    #[test]
    fn test_switch_toggle() {
        let mut sw = Switch::new(false);
        sw.toggle();
        assert!(sw.is_on());
        
        sw.toggle();
        assert!(!sw.is_on());
    }

    #[test]
    fn test_switch_disabled() {
        let mut sw = Switch::new(false).disabled();
        sw.toggle();
        assert!(!sw.is_on()); // Should not change
    }

    #[test]
    fn test_switch_layout() {
        let sw = Switch::new(false).size(48.0, 28.0);
        let size = sw.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 48.0);
        assert_eq!(size.height, 28.0);
    }
}
