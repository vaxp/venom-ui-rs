//! ScrollView widget - scrollable content area
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::ScrollView;
//!
//! let scrollable = ScrollView::vertical()
//!     .child(long_content);
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Point, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// SCROLL AXIS
// ============================================================================

/// Scrolling direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollAxis {
    /// Vertical scrolling only
    #[default]
    Vertical,
    /// Horizontal scrolling only
    Horizontal,
    /// Both directions
    Both,
}

// ============================================================================
// SCROLL VIEW
// ============================================================================

/// A scrollable container widget
/// 
/// # Example
/// 
/// ```ignore
/// // Vertical scroll (most common)
/// let list = ScrollView::vertical()
///     .child(Column::new()
///         .child(item1)
///         .child(item2)
///         // ... many items
///     );
/// 
/// // Horizontal scroll
/// let gallery = ScrollView::horizontal()
///     .child(Row::new().children(images));
/// 
/// // Both directions
/// let canvas = ScrollView::both()
///     .child(large_content);
/// ```
pub struct ScrollView {
    /// Scroll axis
    axis: ScrollAxis,
    /// Child widget
    child: Option<Box<dyn Widget>>,
    /// Current scroll offset X
    scroll_x: f32,
    /// Current scroll offset Y
    scroll_y: f32,
    /// Visible width
    viewport_width: f32,
    /// Visible height
    viewport_height: f32,
    /// Content width
    content_width: f32,
    /// Content height
    content_height: f32,
    /// Scrollbar width
    scrollbar_width: f32,
    /// Scrollbar color
    scrollbar_color: Color,
    /// Scrollbar thumb color
    thumb_color: Color,
    /// Show scrollbar always
    always_show_scrollbar: bool,
    /// Scroll callback
    on_scroll: Option<Box<dyn Fn(f32, f32) + Send + Sync>>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::vertical()
    }
}

impl ScrollView {
    /// Create a new vertical scroll view
    pub fn vertical() -> Self {
        Self::new(ScrollAxis::Vertical)
    }

    /// Create a new horizontal scroll view
    pub fn horizontal() -> Self {
        Self::new(ScrollAxis::Horizontal)
    }

    /// Create a scroll view with both directions
    pub fn both() -> Self {
        Self::new(ScrollAxis::Both)
    }

    /// Create with specific axis
    pub fn new(axis: ScrollAxis) -> Self {
        Self {
            axis,
            child: None,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport_width: 0.0,
            viewport_height: 0.0,
            content_width: 0.0,
            content_height: 0.0,
            scrollbar_width: 8.0,
            scrollbar_color: Color::rgba(100, 100, 100, 80),
            thumb_color: Color::rgba(150, 150, 150, 150),
            always_show_scrollbar: false,
            on_scroll: None,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set the child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set scrollbar width
    pub fn scrollbar_width(mut self, width: f32) -> Self {
        self.scrollbar_width = width.max(2.0);
        self
    }

    /// Set scrollbar colors
    pub fn scrollbar_colors(mut self, track: Color, thumb: Color) -> Self {
        self.scrollbar_color = track;
        self.thumb_color = thumb;
        self
    }

    /// Always show scrollbar
    pub fn always_show_scrollbar(mut self, show: bool) -> Self {
        self.always_show_scrollbar = show;
        self
    }

    /// Set scroll callback
    pub fn on_scroll<F: Fn(f32, f32) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_scroll = Some(Box::new(callback));
        self
    }

    /// Set initial scroll position
    pub fn initial_scroll(mut self, x: f32, y: f32) -> Self {
        self.scroll_x = x.max(0.0);
        self.scroll_y = y.max(0.0);
        self
    }

    // ========================================================================
    // SCROLL CONTROL
    // ========================================================================

    /// Get current scroll offset
    pub fn scroll_offset(&self) -> (f32, f32) {
        (self.scroll_x, self.scroll_y)
    }

    /// Set scroll offset
    pub fn set_scroll(&mut self, x: f32, y: f32) {
        let max_x = (self.content_width - self.viewport_width).max(0.0);
        let max_y = (self.content_height - self.viewport_height).max(0.0);
        
        self.scroll_x = x.clamp(0.0, max_x);
        self.scroll_y = y.clamp(0.0, max_y);
        
        if let Some(callback) = &self.on_scroll {
            callback(self.scroll_x, self.scroll_y);
        }
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, dx: f32, dy: f32) {
        self.set_scroll(self.scroll_x + dx, self.scroll_y + dy);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.set_scroll(self.scroll_x, 0.0);
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        let max_y = (self.content_height - self.viewport_height).max(0.0);
        self.set_scroll(self.scroll_x, max_y);
    }

    /// Check if can scroll vertically
    pub fn can_scroll_vertical(&self) -> bool {
        matches!(self.axis, ScrollAxis::Vertical | ScrollAxis::Both) 
            && self.content_height > self.viewport_height
    }

    /// Check if can scroll horizontally
    pub fn can_scroll_horizontal(&self) -> bool {
        matches!(self.axis, ScrollAxis::Horizontal | ScrollAxis::Both) 
            && self.content_width > self.viewport_width
    }

    /// Get scroll progress (0.0 to 1.0)
    pub fn scroll_progress(&self) -> (f32, f32) {
        let max_x = (self.content_width - self.viewport_width).max(1.0);
        let max_y = (self.content_height - self.viewport_height).max(1.0);
        
        (self.scroll_x / max_x, self.scroll_y / max_y)
    }
}

impl Widget for ScrollView {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            constraints.max_height,
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Draw viewport background (optional)
        let viewport_rect = Rect::new(
            offset.dx,
            offset.dy,
            self.viewport_width,
            self.viewport_height,
        );

        // Save canvas state for clipping (conceptual)
        canvas.save();

        // Draw child with scroll offset
        if let Some(child) = &self.child {
            let child_offset = Offset::new(
                offset.dx - self.scroll_x,
                offset.dy - self.scroll_y,
            );
            child.paint(canvas, child_offset);
        }

        canvas.restore();

        // Draw vertical scrollbar
        if self.can_scroll_vertical() || self.always_show_scrollbar {
            self.draw_vertical_scrollbar(canvas, offset);
        }

        // Draw horizontal scrollbar
        if self.can_scroll_horizontal() || self.always_show_scrollbar {
            self.draw_horizontal_scrollbar(canvas, offset);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl ScrollView {
    fn draw_vertical_scrollbar(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if self.viewport_height <= 0.0 || self.content_height <= 0.0 {
            return;
        }

        let track_x = offset.dx + self.viewport_width - self.scrollbar_width;
        let track_height = self.viewport_height;
        
        // Draw track
        let track_rect = Rect::new(
            track_x,
            offset.dy,
            self.scrollbar_width,
            track_height,
        );
        canvas.draw_rounded_rect(
            track_rect,
            BorderRadius::all(self.scrollbar_width / 2.0),
            &Paint::fill(self.scrollbar_color),
        );

        // Calculate thumb size and position
        let visible_ratio = (self.viewport_height / self.content_height).min(1.0);
        let thumb_height = (track_height * visible_ratio).max(20.0);
        
        let scroll_ratio = self.scroll_y / (self.content_height - self.viewport_height).max(1.0);
        let thumb_y = offset.dy + scroll_ratio * (track_height - thumb_height);

        // Draw thumb
        let thumb_rect = Rect::new(
            track_x,
            thumb_y,
            self.scrollbar_width,
            thumb_height,
        );
        canvas.draw_rounded_rect(
            thumb_rect,
            BorderRadius::all(self.scrollbar_width / 2.0),
            &Paint::fill(self.thumb_color),
        );
    }

    fn draw_horizontal_scrollbar(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if self.viewport_width <= 0.0 || self.content_width <= 0.0 {
            return;
        }

        let track_y = offset.dy + self.viewport_height - self.scrollbar_width;
        let track_width = self.viewport_width - if self.can_scroll_vertical() { self.scrollbar_width } else { 0.0 };
        
        // Draw track
        let track_rect = Rect::new(
            offset.dx,
            track_y,
            track_width,
            self.scrollbar_width,
        );
        canvas.draw_rounded_rect(
            track_rect,
            BorderRadius::all(self.scrollbar_width / 2.0),
            &Paint::fill(self.scrollbar_color),
        );

        // Calculate thumb
        let visible_ratio = (self.viewport_width / self.content_width).min(1.0);
        let thumb_width = (track_width * visible_ratio).max(20.0);
        
        let scroll_ratio = self.scroll_x / (self.content_width - self.viewport_width).max(1.0);
        let thumb_x = offset.dx + scroll_ratio * (track_width - thumb_width);

        // Draw thumb
        let thumb_rect = Rect::new(
            thumb_x,
            track_y,
            thumb_width,
            self.scrollbar_width,
        );
        canvas.draw_rounded_rect(
            thumb_rect,
            BorderRadius::all(self.scrollbar_width / 2.0),
            &Paint::fill(self.thumb_color),
        );
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SizedBox;

    #[test]
    fn test_scroll_view_creation() {
        let sv = ScrollView::vertical();
        assert_eq!(sv.axis, ScrollAxis::Vertical);
        
        let sv = ScrollView::horizontal();
        assert_eq!(sv.axis, ScrollAxis::Horizontal);
    }

    #[test]
    fn test_scroll_offset() {
        let mut sv = ScrollView::vertical();
        sv.viewport_height = 100.0;
        sv.content_height = 500.0;
        
        sv.set_scroll(0.0, 50.0);
        assert_eq!(sv.scroll_offset(), (0.0, 50.0));
    }

    #[test]
    fn test_scroll_clamping() {
        let mut sv = ScrollView::vertical();
        sv.viewport_height = 100.0;
        sv.content_height = 500.0;
        
        sv.set_scroll(0.0, 1000.0);
        assert_eq!(sv.scroll_offset().1, 400.0); // max is content - viewport
    }

    #[test]
    fn test_scroll_progress() {
        let mut sv = ScrollView::vertical();
        sv.viewport_height = 100.0;
        sv.content_height = 500.0;
        
        sv.set_scroll(0.0, 200.0);
        let (_, progress_y) = sv.scroll_progress();
        assert!((progress_y - 0.5).abs() < 0.01);
    }
}
