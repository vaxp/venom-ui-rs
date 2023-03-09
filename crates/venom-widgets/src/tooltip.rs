//! Tooltip Widget - Displays hints on hover
//!
//! `Tooltip` wraps a child widget and shows a hint message when hovered.

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, BorderRadius, Point};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// TOOLTIP POSITION
// ============================================================================

/// Where to position the tooltip relative to the child
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    /// Above the child
    Top,
    /// Below the child
    #[default]
    Bottom,
    /// To the left
    Left,
    /// To the right
    Right,
}

// ============================================================================
// TOOLTIP
// ============================================================================

/// Widget that displays a hint on hover
pub struct Tooltip {
    /// Child widget
    child: BoxedWidget,
    /// Tooltip message
    message: String,
    /// Position relative to child
    position: TooltipPosition,
    /// Gap between child and tooltip
    gap: f32,
    /// Delay before showing (ms)
    delay_ms: u32,
    /// Background color
    background: Color,
    /// Text color
    text_color: Color,
    /// Padding
    padding: Insets,
    /// Border radius
    border_radius: f32,
    /// Whether visible
    visible: bool,
    /// Max width
    max_width: f32,
}

impl Tooltip {
    /// Create a new tooltip
    pub fn new<W: Widget + 'static>(child: W, message: impl Into<String>) -> Self {
        Self {
            child: Box::new(child),
            message: message.into(),
            position: TooltipPosition::default(),
            gap: 4.0,
            delay_ms: 400,
            background: Color::rgba(50, 50, 50, 230),
            text_color: Color::WHITE,
            padding: Insets::symmetric(8.0, 4.0),
            border_radius: 4.0,
            visible: false,
            max_width: 200.0,
        }
    }

    /// Create from boxed child
    pub fn from_boxed(child: BoxedWidget, message: impl Into<String>) -> Self {
        Self {
            child,
            message: message.into(),
            position: TooltipPosition::default(),
            gap: 4.0,
            delay_ms: 400,
            background: Color::rgba(50, 50, 50, 230),
            text_color: Color::WHITE,
            padding: Insets::symmetric(8.0, 4.0),
            border_radius: 4.0,
            visible: false,
            max_width: 200.0,
        }
    }

    /// Set tooltip position
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// Set gap
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Set delay (ms)
    pub fn delay_ms(mut self, delay: u32) -> Self {
        self.delay_ms = delay;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set max width
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    /// Get the message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Calculate tooltip size
    fn tooltip_size(&self) -> Size {
        let char_width = 7.0;
        let line_height = 16.0;
        let text_width = (self.message.len() as f32 * char_width).min(self.max_width);
        Size::new(
            text_width + self.padding.left + self.padding.right,
            line_height + self.padding.top + self.padding.bottom,
        )
    }

    /// Calculate tooltip offset relative to child
    fn tooltip_offset(&self, child_size: Size, tooltip_size: Size) -> Offset {
        match self.position {
            TooltipPosition::Top => Offset::new(
                (child_size.width - tooltip_size.width) / 2.0,
                -tooltip_size.height - self.gap,
            ),
            TooltipPosition::Bottom => Offset::new(
                (child_size.width - tooltip_size.width) / 2.0,
                child_size.height + self.gap,
            ),
            TooltipPosition::Left => Offset::new(
                -tooltip_size.width - self.gap,
                (child_size.height - tooltip_size.height) / 2.0,
            ),
            TooltipPosition::Right => Offset::new(
                child_size.width + self.gap,
                (child_size.height - tooltip_size.height) / 2.0,
            ),
        }
    }
}

impl Widget for Tooltip {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        self.child.paint(canvas, offset);

        if self.visible {
            let child_size = self.child.layout(BoxConstraints::new());
            let tooltip_size = self.tooltip_size();
            let tooltip_offset = self.tooltip_offset(child_size, tooltip_size);

            let tooltip_rect = Rect::new(
                offset.dx + tooltip_offset.dx,
                offset.dy + tooltip_offset.dy,
                tooltip_size.width,
                tooltip_size.height,
            );

            canvas.draw_rounded_rect(
                tooltip_rect,
                BorderRadius::all(self.border_radius),
                &Paint::fill(self.background),
            );

            // Draw text
            let text_pos = Point::new(
                tooltip_rect.x + self.padding.left,
                tooltip_rect.y + self.padding.top,
            );
            canvas.draw_text(&self.message, text_pos, &Paint::fill(self.text_color), 14.0);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
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
    use crate::SizedBox;

    #[test]
    fn test_tooltip_creation() {
        let tooltip = Tooltip::new(SizedBox::square(50.0), "Test message");
        assert_eq!(tooltip.message(), "Test message");
        assert_eq!(tooltip.position, TooltipPosition::Bottom);
    }

    #[test]
    fn test_tooltip_position() {
        let tooltip = Tooltip::new(SizedBox::square(50.0), "Test")
            .position(TooltipPosition::Top);
        assert_eq!(tooltip.position, TooltipPosition::Top);
    }

    #[test]
    fn test_tooltip_customization() {
        let tooltip = Tooltip::new(SizedBox::square(50.0), "Test")
            .delay_ms(1000)
            .gap(8.0)
            .border_radius(8.0)
            .max_width(300.0);
        
        assert_eq!(tooltip.delay_ms, 1000);
        assert_eq!(tooltip.gap, 8.0);
        assert_eq!(tooltip.border_radius, 8.0);
        assert_eq!(tooltip.max_width, 300.0);
    }
}
