//! Progress indicator widgets
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{ProgressBar, ProgressCircle};
//!
//! let bar = ProgressBar::new(0.75);
//! let circle = ProgressCircle::new(0.5);
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Point, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// PROGRESS BAR
// ============================================================================

/// A horizontal progress bar
/// 
/// # Example
/// 
/// ```ignore
/// // Simple progress
/// let progress = ProgressBar::new(0.5); // 50%
/// 
/// // Styled progress
/// let download = ProgressBar::new(0.75)
///     .height(8.0)
///     .color(Color::hex("#22c55e"))
///     .background(Color::hex("#1f2937"));
/// ```
pub struct ProgressBar {
    /// Progress value (0.0 to 1.0)
    value: f32,
    /// Bar width
    width: f32,
    /// Bar height
    height: f32,
    /// Progress color
    color: Color,
    /// Background color
    background: Color,
    /// Border radius
    border_radius: f32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl ProgressBar {
    /// Create a new progress bar with value (0.0 to 1.0)
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            width: 200.0,
            height: 6.0,
            color: Color::hex("#6366f1"),
            background: Color::hex("#374151"),
            border_radius: 3.0,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(20.0);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(2.0);
        self.border_radius = self.height / 2.0;
        self
    }

    /// Set progress color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
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

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get progress value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set progress value
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for ProgressBar {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.constrain_width(self.width),
            constraints.constrain_height(self.height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let radius = BorderRadius::all(self.border_radius);

        // Background track
        let bg_rect = Rect::new(offset.dx, offset.dy, self.width, self.height);
        canvas.draw_rounded_rect(bg_rect, radius, &Paint::fill(self.background));

        // Progress fill
        let fill_width = self.width * self.value;
        if fill_width > 0.0 {
            let fill_rect = Rect::new(offset.dx, offset.dy, fill_width, self.height);
            canvas.draw_rounded_rect(fill_rect, radius, &Paint::fill(self.color));
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// PROGRESS CIRCLE
// ============================================================================

/// A circular progress indicator
/// 
/// # Example
/// 
/// ```ignore
/// let loading = ProgressCircle::new(0.6)
///     .size(48.0)
///     .stroke_width(4.0);
/// ```
pub struct ProgressCircle {
    /// Progress value (0.0 to 1.0)
    value: f32,
    /// Circle size
    size: f32,
    /// Stroke width
    stroke_width: f32,
    /// Progress color
    color: Color,
    /// Track color
    track_color: Color,
}

impl Default for ProgressCircle {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl ProgressCircle {
    /// Create a new circular progress indicator
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            size: 40.0,
            stroke_width: 4.0,
            color: Color::hex("#6366f1"),
            track_color: Color::hex("#374151"),
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set circle size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(16.0);
        self
    }

    /// Set stroke width
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width.max(1.0);
        self
    }

    /// Set progress color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set track color
    pub fn track_color(mut self, color: Color) -> Self {
        self.track_color = color;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get progress value
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Set progress value
    pub fn set_value(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}

impl Widget for ProgressCircle {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.constrain_width(self.size),
            constraints.constrain_height(self.size),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let center = Point::new(
            offset.dx + self.size / 2.0,
            offset.dy + self.size / 2.0,
        );
        let radius = (self.size - self.stroke_width) / 2.0;

        // Draw track (full circle)
        canvas.draw_circle(center, radius, &Paint::stroke(self.track_color, self.stroke_width));

        // Draw progress arc
        // For now, draw a simple arc representation
        // TODO: Implement proper arc drawing via path
        if self.value > 0.0 {
            // Simplified: draw as a partial fill for now
            // A full implementation would use arc paths
            let progress_radius = radius * self.value;
            canvas.draw_circle(center, progress_radius, &Paint::fill(self.color));
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
    fn test_progress_bar() {
        let bar = ProgressBar::new(0.5);
        assert_eq!(bar.value(), 0.5);
        
        let bar = ProgressBar::new(1.5); // Clamped
        assert_eq!(bar.value(), 1.0);
    }

    #[test]
    fn test_progress_circle() {
        let circle = ProgressCircle::new(0.75).size(60.0);
        assert_eq!(circle.value(), 0.75);
    }

    #[test]
    fn test_progress_set_value() {
        let mut bar = ProgressBar::new(0.0);
        bar.set_value(0.5);
        assert_eq!(bar.value(), 0.5);
        
        bar.set_value(2.0); // Clamped
        assert_eq!(bar.value(), 1.0);
    }
}
