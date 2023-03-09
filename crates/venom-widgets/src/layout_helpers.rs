//! Layout Helper Widgets - Spacer, Divider, Expanded, Flexible
//!
//! This module provides layout helper widgets for creating flexible UI layouts:
//!
//! - [`Spacer`] - Flexible empty space that expands to fill available room
//! - [`Divider`] - Visual separator line (horizontal or vertical)
//! - [`Expanded`] - Widget that expands to fill available space in Flex layouts
//! - [`Flexible`] - Widget with flexible sizing in Flex layouts
//! - [`AspectRatio`] - Maintains a fixed aspect ratio for its child
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Row, Spacer, Divider, Text};
//!
//! // Push items apart with Spacer
//! let toolbar = Row::new()
//!     .child(Text::new("Left"))
//!     .child(Spacer::horizontal())
//!     .child(Text::new("Right"));
//!
//! // Add visual separation
//! let layout = Column::new()
//!     .child(section1)
//!     .child(Divider::horizontal())
//!     .child(section2);
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Color, Offset, Size};
use venom_render::PaintCanvas;
use crate::Widget;

// ============================================================================
// SPACER
// ============================================================================

/// Flexible empty space widget
///
/// `Spacer` takes up available space along its main axis. It's commonly used
/// to push widgets apart in `Row` or `Column` layouts.
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{Row, Spacer, Button};
///
/// // Push buttons to opposite ends
/// let row = Row::new()
///     .child(Button::new("Cancel"))
///     .child(Spacer::horizontal())
///     .child(Button::new("OK"));
///
/// // Create fixed spacing
/// let row = Row::new()
///     .child(icon)
///     .child(Spacer::new().width(16.0))
///     .child(label);
/// ```
#[derive(Debug, Clone, Default)]
pub struct Spacer {
    /// Minimum width (0 = flexible)
    min_width: f32,
    /// Minimum height (0 = flexible)
    min_height: f32,
    /// Flex factor for horizontal expansion
    flex_horizontal: bool,
    /// Flex factor for vertical expansion  
    flex_vertical: bool,
}

impl Spacer {
    /// Create a new spacer with no minimum size
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a horizontal spacer that fills available width
    ///
    /// This is the most common use case - pushing items apart in a Row.
    pub fn horizontal() -> Self {
        Self {
            flex_horizontal: true,
            ..Default::default()
        }
    }

    /// Create a vertical spacer that fills available height
    ///
    /// Useful for pushing items apart in a Column.
    pub fn vertical() -> Self {
        Self {
            flex_vertical: true,
            ..Default::default()
        }
    }

    /// Create a spacer that expands in both directions
    pub fn expand() -> Self {
        Self {
            flex_horizontal: true,
            flex_vertical: true,
            ..Default::default()
        }
    }

    /// Set minimum width
    pub fn width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Set minimum height
    pub fn height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    /// Set both minimum width and height
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    /// Set flex behavior for horizontal axis
    pub fn flex_horizontal(mut self, flex: bool) -> Self {
        self.flex_horizontal = flex;
        self
    }

    /// Set flex behavior for vertical axis
    pub fn flex_vertical(mut self, flex: bool) -> Self {
        self.flex_vertical = flex;
        self
    }
}

impl Widget for Spacer {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let width = if self.flex_horizontal {
            constraints.max_width.min(f32::MAX)
        } else {
            constraints.constrain_width(self.min_width)
        };

        let height = if self.flex_vertical {
            constraints.max_height.min(f32::MAX)
        } else {
            constraints.constrain_height(self.min_height)
        };

        Size::new(width, height)
    }

    fn paint(&self, _canvas: &mut dyn PaintCanvas, _offset: Offset) {
        // Spacer is invisible - nothing to paint
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// DIVIDER
// ============================================================================

/// Visual separator line
///
/// `Divider` draws a thin line to visually separate content.
/// Can be horizontal (for separating rows) or vertical (for separating columns).
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{Column, Divider, Text};
///
/// let list = Column::new()
///     .child(Text::new("Item 1"))
///     .child(Divider::horizontal())
///     .child(Text::new("Item 2"))
///     .child(Divider::horizontal().color(Color::RED))
///     .child(Text::new("Item 3"));
/// ```
#[derive(Debug, Clone)]
pub struct Divider {
    /// Orientation
    is_horizontal: bool,
    /// Line color
    color: Color,
    /// Line thickness
    thickness: f32,
    /// Indent from start
    indent: f32,
    /// Indent from end
    end_indent: f32,
}

impl Default for Divider {
    fn default() -> Self {
        Self {
            is_horizontal: true,
            color: Color::rgba(128, 128, 128, 77), // Semi-transparent gray
            thickness: 1.0,
            indent: 0.0,
            end_indent: 0.0,
        }
    }
}

impl Divider {
    /// Create a horizontal divider
    pub fn horizontal() -> Self {
        Self::default()
    }

    /// Create a vertical divider
    pub fn vertical() -> Self {
        Self {
            is_horizontal: false,
            ..Default::default()
        }
    }

    /// Set the divider color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the line thickness
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// Set indent from start (left for horizontal, top for vertical)
    pub fn indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    /// Set indent from end (right for horizontal, bottom for vertical)
    pub fn end_indent(mut self, end_indent: f32) -> Self {
        self.end_indent = end_indent;
        self
    }

    /// Set both start and end indent
    pub fn indents(mut self, start: f32, end: f32) -> Self {
        self.indent = start;
        self.end_indent = end;
        self
    }
}

impl Widget for Divider {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        if self.is_horizontal {
            Size::new(
                constraints.max_width.min(f32::MAX),
                self.thickness,
            )
        } else {
            Size::new(
                self.thickness,
                constraints.max_height.min(f32::MAX),
            )
        }
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        use venom_core::Rect;
        use venom_render::Paint;

        // For dividers, we use a fixed reasonable length if we don't have constraints
        // In practice, layout() should be called first which will give us proper sizing
        let paint = Paint::fill(self.color);

        if self.is_horizontal {
            // Draw a horizontal line
            let rect = Rect::new(
                offset.dx + self.indent,
                offset.dy,
                200.0 - self.indent - self.end_indent, // Default length
                self.thickness,
            );
            canvas.draw_rect(rect, &paint);
        } else {
            // Draw a vertical line
            let rect = Rect::new(
                offset.dx,
                offset.dy + self.indent,
                self.thickness,
                200.0 - self.indent - self.end_indent, // Default length
            );
            canvas.draw_rect(rect, &paint);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// EXPANDED
// ============================================================================

/// Widget that expands to fill available space in Flex layouts
///
/// `Expanded` forces its child to fill all available space along the main
/// axis of a `Row` or `Column`. Multiple `Expanded` widgets share the space
/// according to their `flex` factor.
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{Row, Expanded, Container};
///
/// // Two containers sharing space equally
/// let row = Row::new()
///     .child(Expanded::new(Container::new().color(Color::RED)))
///     .child(Expanded::new(Container::new().color(Color::BLUE)));
///
/// // Different flex factors (2:1 ratio)
/// let row = Row::new()
///     .child(Expanded::new(sidebar).flex(1))
///     .child(Expanded::new(content).flex(2));
/// ```
pub struct Expanded {
    /// The child widget
    child: Box<dyn Widget>,
    /// Flex factor (default: 1)
    flex: u32,
}

impl Expanded {
    /// Create a new Expanded wrapping the given widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            flex: 1,
        }
    }

    /// Create from a boxed widget
    pub fn from_boxed(child: Box<dyn Widget>) -> Self {
        Self { child, flex: 1 }
    }

    /// Set the flex factor
    ///
    /// Higher values mean the widget takes up more space relative to other
    /// Expanded widgets. Default is 1.
    pub fn flex(mut self, flex: u32) -> Self {
        self.flex = flex;
        self
    }

    /// Get the flex factor
    pub fn flex_factor(&self) -> u32 {
        self.flex
    }
}

impl Widget for Expanded {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        // Expanded fills all available space
        let child_constraints = BoxConstraints {
            min_width: constraints.max_width,
            max_width: constraints.max_width,
            min_height: constraints.max_height,
            max_height: constraints.max_height,
        };
        self.child.layout(child_constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        self.child.paint(canvas, offset);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// FLEXIBLE
// ============================================================================

/// How a flexible widget should size itself
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlexFit {
    /// Take only the space needed (up to max allowed)
    #[default]
    Loose,
    /// Fill all available space
    Tight,
}

/// Widget with flexible sizing in Flex layouts
///
/// `Flexible` is like `Expanded` but doesn't force the child to fill all
/// available space. With `FlexFit::Loose`, the child takes only what it needs.
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{Row, Flexible, FlexFit, Text};
///
/// // Flexible with loose fit - takes only needed space
/// let row = Row::new()
///     .child(Flexible::new(Text::new("Short")))
///     .child(Flexible::new(Text::new("Longer text here")));
///
/// // With tight fit - same as Expanded
/// let row = Row::new()
///     .child(Flexible::new(content).fit(FlexFit::Tight));
/// ```
pub struct Flexible {
    /// The child widget
    child: Box<dyn Widget>,
    /// Flex factor
    flex: u32,
    /// How to fit
    fit: FlexFit,
}

impl Flexible {
    /// Create a new Flexible wrapping the given widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            flex: 1,
            fit: FlexFit::Loose,
        }
    }

    /// Create from a boxed widget
    pub fn from_boxed(child: Box<dyn Widget>) -> Self {
        Self {
            child,
            flex: 1,
            fit: FlexFit::Loose,
        }
    }

    /// Set the flex factor
    pub fn flex(mut self, flex: u32) -> Self {
        self.flex = flex;
        self
    }

    /// Set how the widget should fit in available space
    pub fn fit(mut self, fit: FlexFit) -> Self {
        self.fit = fit;
        self
    }

    /// Get the flex factor
    pub fn flex_factor(&self) -> u32 {
        self.flex
    }

    /// Get the fit mode
    pub fn fit_mode(&self) -> FlexFit {
        self.fit
    }
}

impl Widget for Flexible {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        match self.fit {
            FlexFit::Tight => {
                // Fill all available space
                let child_constraints = BoxConstraints {
                    min_width: constraints.max_width,
                    max_width: constraints.max_width,
                    min_height: constraints.max_height,
                    max_height: constraints.max_height,
                };
                self.child.layout(child_constraints)
            }
            FlexFit::Loose => {
                // Take only needed space, up to max
                let child_constraints = BoxConstraints {
                    min_width: 0.0,
                    max_width: constraints.max_width,
                    min_height: 0.0,
                    max_height: constraints.max_height,
                };
                self.child.layout(child_constraints)
            }
        }
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        self.child.paint(canvas, offset);
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ASPECT RATIO
// ============================================================================

/// Maintains a fixed aspect ratio for its child
///
/// `AspectRatio` sizes itself to maintain the given ratio (width / height).
/// The child is sized to fill the resulting box.
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{AspectRatio, Image};
///
/// // 16:9 video aspect ratio
/// let video = AspectRatio::new(16.0 / 9.0)
///     .child(video_player);
///
/// // Square thumbnail
/// let thumbnail = AspectRatio::square()
///     .child(image);
/// ```
pub struct AspectRatio {
    /// The aspect ratio (width / height)
    ratio: f32,
    /// The child widget
    child: Option<Box<dyn Widget>>,
}

impl AspectRatio {
    /// Create with a specific aspect ratio
    ///
    /// Ratio is width / height. For example:
    /// - 16:9 = 16.0 / 9.0 = 1.778
    /// - 4:3 = 4.0 / 3.0 = 1.333
    /// - Square = 1.0
    pub fn new(ratio: f32) -> Self {
        Self { ratio, child: None }
    }

    /// Create a square aspect ratio (1:1)
    pub fn square() -> Self {
        Self::new(1.0)
    }

    /// Create a 16:9 aspect ratio (common for video)
    pub fn widescreen() -> Self {
        Self::new(16.0 / 9.0)
    }

    /// Create a 4:3 aspect ratio
    pub fn standard() -> Self {
        Self::new(4.0 / 3.0)
    }

    /// Set the child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set a boxed child widget
    pub fn child_boxed(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }

    /// Get the aspect ratio
    pub fn ratio(&self) -> f32 {
        self.ratio
    }
}

impl Widget for AspectRatio {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        // Try to match the aspect ratio within constraints
        let max_width = constraints.max_width;
        let max_height = constraints.max_height;

        // Calculate size maintaining aspect ratio
        let (width, height) = if max_width.is_finite() && max_height.is_finite() {
            // Both constrained - fit within bounds
            let height_from_width = max_width / self.ratio;
            let width_from_height = max_height * self.ratio;

            if height_from_width <= max_height {
                (max_width, height_from_width)
            } else {
                (width_from_height, max_height)
            }
        } else if max_width.is_finite() {
            // Only width constrained
            (max_width, max_width / self.ratio)
        } else if max_height.is_finite() {
            // Only height constrained
            (max_height * self.ratio, max_height)
        } else {
            // No constraints - use minimum
            let min_width = constraints.min_width.max(0.0);
            let min_height = constraints.min_height.max(0.0);
            if min_width > 0.0 {
                (min_width, min_width / self.ratio)
            } else if min_height > 0.0 {
                (min_height * self.ratio, min_height)
            } else {
                (0.0, 0.0)
            }
        };

        // Layout child with tight constraints
        if let Some(child) = &self.child {
            let child_constraints = BoxConstraints {
                min_width: width,
                max_width: width,
                min_height: height,
                max_height: height,
            };
            child.layout(child_constraints);
        }

        Size::new(width, height)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if let Some(child) = &self.child {
            child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// FRACTIONALLY SIZED BOX
// ============================================================================

/// Widget that sizes itself as a fraction of its parent
///
/// `FractionallySizedBox` takes a fraction (0.0 to 1.0) of the available
/// space along each axis.
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{FractionallySizedBox, Container};
///
/// // Take half the width and full height
/// let half_width = FractionallySizedBox::new()
///     .width_factor(0.5)
///     .child(content);
///
/// // Take 80% of both dimensions
/// let centered = FractionallySizedBox::new()
///     .factors(0.8, 0.8)
///     .child(content);
/// ```
pub struct FractionallySizedBox {
    /// Width factor (0.0 to 1.0, None = child's intrinsic width)
    width_factor: Option<f32>,
    /// Height factor (0.0 to 1.0, None = child's intrinsic height)
    height_factor: Option<f32>,
    /// Child widget
    child: Option<Box<dyn Widget>>,
}

impl Default for FractionallySizedBox {
    fn default() -> Self {
        Self {
            width_factor: None,
            height_factor: None,
            child: None,
        }
    }
}

impl FractionallySizedBox {
    /// Create a new FractionallySizedBox
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the width factor (0.0 to 1.0)
    pub fn width_factor(mut self, factor: f32) -> Self {
        self.width_factor = Some(factor.clamp(0.0, 1.0));
        self
    }

    /// Set the height factor (0.0 to 1.0)
    pub fn height_factor(mut self, factor: f32) -> Self {
        self.height_factor = Some(factor.clamp(0.0, 1.0));
        self
    }

    /// Set both width and height factors
    pub fn factors(mut self, width: f32, height: f32) -> Self {
        self.width_factor = Some(width.clamp(0.0, 1.0));
        self.height_factor = Some(height.clamp(0.0, 1.0));
        self
    }

    /// Set the child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set a boxed child
    pub fn child_boxed(mut self, child: Box<dyn Widget>) -> Self {
        self.child = Some(child);
        self
    }
}

impl Widget for FractionallySizedBox {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let width = match self.width_factor {
            Some(factor) => constraints.max_width * factor,
            None => constraints.max_width,
        };

        let height = match self.height_factor {
            Some(factor) => constraints.max_height * factor,
            None => constraints.max_height,
        };

        let size = Size::new(
            width.min(constraints.max_width).max(constraints.min_width),
            height.min(constraints.max_height).max(constraints.min_height),
        );

        // Layout child with tight constraints
        if let Some(child) = &self.child {
            let child_constraints = BoxConstraints {
                min_width: size.width,
                max_width: size.width,
                min_height: size.height,
                max_height: size.height,
            };
            child.layout(child_constraints);
        }

        size
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if let Some(child) = &self.child {
            child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
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
    fn test_spacer_horizontal() {
        let spacer = Spacer::horizontal();
        assert!(spacer.flex_horizontal);
        assert!(!spacer.flex_vertical);
    }

    #[test]
    fn test_spacer_vertical() {
        let spacer = Spacer::vertical();
        assert!(!spacer.flex_horizontal);
        assert!(spacer.flex_vertical);
    }

    #[test]
    fn test_spacer_with_size() {
        let spacer = Spacer::new().width(100.0).height(50.0);
        assert_eq!(spacer.min_width, 100.0);
        assert_eq!(spacer.min_height, 50.0);
    }

    #[test]
    fn test_divider_horizontal() {
        let divider = Divider::horizontal();
        assert!(divider.is_horizontal);
        assert_eq!(divider.thickness, 1.0);
    }

    #[test]
    fn test_divider_vertical() {
        let divider = Divider::vertical();
        assert!(!divider.is_horizontal);
    }

    #[test]
    fn test_divider_custom() {
        let divider = Divider::horizontal()
            .color(Color::RED)
            .thickness(2.0)
            .indent(16.0)
            .end_indent(16.0);
        assert_eq!(divider.thickness, 2.0);
        assert_eq!(divider.indent, 16.0);
        assert_eq!(divider.end_indent, 16.0);
    }

    #[test]
    fn test_aspect_ratio() {
        let ar = AspectRatio::widescreen();
        assert!((ar.ratio - 16.0 / 9.0).abs() < 0.001);

        let ar = AspectRatio::square();
        assert_eq!(ar.ratio, 1.0);
    }

    #[test]
    fn test_fractionally_sized() {
        let fsb = FractionallySizedBox::new()
            .width_factor(0.5)
            .height_factor(0.8);
        assert_eq!(fsb.width_factor, Some(0.5));
        assert_eq!(fsb.height_factor, Some(0.8));
    }

    #[test]
    fn test_fractionally_sized_clamping() {
        let fsb = FractionallySizedBox::new()
            .width_factor(2.0)  // Should clamp to 1.0
            .height_factor(-0.5); // Should clamp to 0.0
        assert_eq!(fsb.width_factor, Some(1.0));
        assert_eq!(fsb.height_factor, Some(0.0));
    }
}
