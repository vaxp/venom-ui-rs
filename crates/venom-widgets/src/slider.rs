//! Slider widget - value range input
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::Slider;
//!
//! let volume = Slider::new(0.0..=100.0, current_volume)
//!     .on_change(|value| set_volume(value));
//! ```

use std::any::Any;
use std::ops::RangeInclusive;
use venom_core::{BoxConstraints, Size, Offset, Color, Point, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// SLIDER
// ============================================================================

/// A slider for selecting a value in a range
/// 
/// # Example
/// 
/// ```ignore
/// // Volume slider
/// let volume = Slider::new(0.0..=100.0, 50.0)
///     .on_change(|v| println!("Volume: {}", v));
/// 
/// // Brightness with custom style
/// let brightness = Slider::new(0.0..=1.0, 0.8)
///     .track_color(Color::hex("#374151"))
///     .active_color(Color::hex("#fbbf24"))
///     .thumb_size(16.0);
/// ```
pub struct Slider {
    /// Value range
    range: RangeInclusive<f32>,
    /// Current value
    value: f32,
    /// Slider width
    width: f32,
    /// Track height
    track_height: f32,
    /// Thumb radius
    thumb_radius: f32,
    /// Track color (inactive part)
    track_color: Color,
    /// Active color (filled part)
    active_color: Color,
    /// Thumb color
    thumb_color: Color,
    /// Callback when value changes
    on_change: Option<Box<dyn Fn(f32) + Send + Sync>>,
    /// Step size (0 = continuous)
    step: f32,
    /// Disabled state
    disabled: bool,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0..=100.0, 50.0)
    }
}

impl Slider {
    /// Create a new slider with range and initial value
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        Self {
            range: range.clone(),
            value: value.clamp(*range.start(), *range.end()),
            width: 200.0,
            track_height: 4.0,
            thumb_radius: 8.0,
            track_color: Color::hex("#374151"),
            active_color: Color::hex("#6366f1"),
            thumb_color: Color::WHITE,
            on_change: None,
            step: 0.0,
            disabled: false,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set slider width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(50.0);
        self
    }

    /// Set track height
    pub fn track_height(mut self, height: f32) -> Self {
        self.track_height = height.max(2.0);
        self
    }

    /// Set thumb size (radius)
    pub fn thumb_size(mut self, radius: f32) -> Self {
        self.thumb_radius = radius.max(4.0);
        self
    }

    /// Set track color
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    /// Set active (filled) color
    pub fn active_color(mut self, color: Color) -> Self {
        self.active_color = color;
        self
    }

    /// Set thumb color
    pub fn thumb_color(mut self, color: Color) -> Self {
        self.thumb_color = color;
        self
    }

    /// Set step size (0 = continuous)
    pub fn step(mut self, step: f32) -> Self {
        self.step = step.max(0.0);
        self
    }

    /// Set change callback
    pub fn on_change<F: Fn(f32) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Disable the slider
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get current value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Get the range
    pub fn range(&self) -> &RangeInclusive<f32> {
        &self.range
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        let start = *self.range.start();
        let end = *self.range.end();
        if (end - start).abs() < f32::EPSILON {
            0.0
        } else {
            (self.value - start) / (end - start)
        }
    }

    /// Set value (for external updates)
    pub fn set_value(&mut self, value: f32) {
        let mut v = value.clamp(*self.range.start(), *self.range.end());
        
        // Apply step
        if self.step > 0.0 {
            let start = *self.range.start();
            v = start + ((v - start) / self.step).round() * self.step;
        }
        
        self.value = v;
    }
}

impl Widget for Slider {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let height = self.thumb_radius * 2.0;
        Size::new(
            constraints.constrain_width(self.width),
            constraints.constrain_height(height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let track_y = offset.dy + self.thumb_radius - self.track_height / 2.0;
        let track_radius = self.track_height / 2.0;
        
        let progress = self.progress();
        let active_width = self.width * progress;

        // Draw inactive track
        let track_rect = Rect::new(offset.dx, track_y, self.width, self.track_height);
        let track_color = if self.disabled {
            Color::hex("#1f2937")
        } else {
            self.track_color
        };
        canvas.draw_rounded_rect(
            track_rect,
            BorderRadius::all(track_radius),
            &Paint::fill(track_color),
        );

        // Draw active track
        if active_width > 0.0 {
            let active_rect = Rect::new(offset.dx, track_y, active_width, self.track_height);
            let active_color = if self.disabled {
                Color::hex("#374151")
            } else {
                self.active_color
            };
            canvas.draw_rounded_rect(
                active_rect,
                BorderRadius::all(track_radius),
                &Paint::fill(active_color),
            );
        }

        // Draw thumb
        let thumb_x = offset.dx + active_width;
        let thumb_y = offset.dy + self.thumb_radius;
        let thumb_color = if self.disabled {
            Color::hex("#6b7280")
        } else {
            self.thumb_color
        };

        canvas.draw_circle(
            Point::new(thumb_x, thumb_y),
            self.thumb_radius,
            &Paint::fill(thumb_color),
        );

        // Draw thumb border
        if !self.disabled {
            canvas.draw_circle(
                Point::new(thumb_x, thumb_y),
                self.thumb_radius,
                &Paint::stroke(self.active_color, 2.0),
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
    fn test_slider_creation() {
        let slider = Slider::new(0.0..=100.0, 50.0);
        assert_eq!(slider.value(), 50.0);
        assert_eq!(slider.progress(), 0.5);
    }

    #[test]
    fn test_slider_clamping() {
        let slider = Slider::new(0.0..=100.0, 150.0);
        assert_eq!(slider.value(), 100.0);
        
        let slider = Slider::new(0.0..=100.0, -10.0);
        assert_eq!(slider.value(), 0.0);
    }

    #[test]
    fn test_slider_step() {
        let mut slider = Slider::new(0.0..=100.0, 0.0).step(10.0);
        slider.set_value(23.0);
        assert_eq!(slider.value(), 20.0);
        
        slider.set_value(27.0);
        assert_eq!(slider.value(), 30.0);
    }

    #[test]
    fn test_slider_progress() {
        let slider = Slider::new(0.0..=100.0, 25.0);
        assert_eq!(slider.progress(), 0.25);
        
        let slider = Slider::new(50.0..=150.0, 100.0);
        assert_eq!(slider.progress(), 0.5);
    }
}
