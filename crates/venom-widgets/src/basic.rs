//! Basic widgets - Container, SizedBox, Padding, Center
//!
//! These are the fundamental building blocks for UI layouts.
//! They provide positioning, sizing, decoration, and spacing.
//!
//! # Available Widgets
//!
//! - **Container**: Versatile box with color, padding, margin, border
//! - **SizedBox**: Forces a specific size
//! - **Padding**: Adds padding around a child
//! - **Center**: Centers its child
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Container, Padding, Center, Color};
//!
//! let card = Container::new()
//!     .color(Color::hex("#1a1a2e"))
//!     .padding_all(16.0)
//!     .border_radius(8.0)
//!     .child(
//!         Center::new().child(
//!             Text::new("Hello, World!")
//!         )
//!     );
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Insets, Color, Rect, BorderRadius, Alignment};
use venom_render::{PaintCanvas, Paint};
use crate::{Widget, BoxedWidget};

// ============================================================================
// CONTAINER
// ============================================================================

/// Container widget - a versatile box with decoration and layout options
/// 
/// Container is the most commonly used widget for:
/// - Background colors and images
/// - Borders and rounded corners
/// - Padding and margin
/// - Fixed dimensions
/// - Child alignment
/// 
/// # Builder Pattern
/// 
/// Container uses a fluent builder API:
/// 
/// ```ignore
/// let container = Container::new()
///     .width(200.0)
///     .height(100.0)
///     .color(Color::hex("#1a1a2e"))
///     .padding(Insets::all(16.0))
///     .margin(Insets::symmetric(8.0, 0.0))
///     .border_radius(12.0)
///     .border(Color::WHITE, 1.0)
///     .alignment(Alignment::CENTER)
///     .child(my_content);
/// ```
/// 
/// # Layout Behavior
/// 
/// - If width/height are set, uses those dimensions
/// - Otherwise, sizes to fit child plus padding/margin
/// - If no child, uses constraint minimums
/// 
/// # Rendering Order
/// 
/// 1. Background color (with border radius)
/// 2. Border (with border radius)
/// 3. Child (at aligned position)
#[derive(Default)]
pub struct Container {
    /// Optional child widget
    child: Option<BoxedWidget>,
    
    /// Fixed width (None = size to child or constraints)
    pub width: Option<f32>,
    
    /// Fixed height (None = size to child or constraints)
    pub height: Option<f32>,
    
    /// Background color
    pub color: Option<Color>,
    
    /// Padding inside the container (between border and child)
    pub padding: Insets,
    
    /// Margin outside the container
    pub margin: Insets,
    
    /// Border radius for rounded corners
    pub border_radius: f32,
    
    /// Border color (None = no border)
    pub border_color: Option<Color>,
    
    /// Border width in pixels
    pub border_width: f32,
    
    /// Alignment of child within container
    pub alignment: Alignment,
}

impl Container {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a new empty container
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a container with fixed dimensions
    pub fn sized(width: f32, height: f32) -> Self {
        Self::new().width(width).height(height)
    }

    /// Create a square container
    pub fn square(size: f32) -> Self {
        Self::sized(size, size)
    }

    // ========================================================================
    // BUILDER METHODS - CHILD
    // ========================================================================

    /// Set a child widget
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Set a boxed child widget
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.child = Some(widget);
        self
    }

    // ========================================================================
    // BUILDER METHODS - DIMENSIONS
    // ========================================================================

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.max(0.0));
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height.max(0.0));
        self
    }

    /// Set both width and height
    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    // ========================================================================
    // BUILDER METHODS - DECORATION
    // ========================================================================

    /// Set background color
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Set border radius for all corners
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius.max(0.0);
        self
    }

    /// Set border
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width.max(0.0);
        self
    }

    /// Set just border color (keeps existing width)
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    /// Set just border width (keeps existing color)
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width.max(0.0);
        self
    }

    // ========================================================================
    // BUILDER METHODS - SPACING
    // ========================================================================

    /// Set padding with custom insets
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform padding on all sides
    pub fn padding_all(mut self, value: f32) -> Self {
        self.padding = Insets::all(value);
        self
    }

    /// Set horizontal and vertical padding
    pub fn padding_symmetric(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding = Insets::symmetric(horizontal, vertical);
        self
    }

    /// Set margin with custom insets
    pub fn margin(mut self, margin: Insets) -> Self {
        self.margin = margin;
        self
    }

    /// Set uniform margin on all sides
    pub fn margin_all(mut self, value: f32) -> Self {
        self.margin = Insets::all(value);
        self
    }

    // ========================================================================
    // BUILDER METHODS - ALIGNMENT
    // ========================================================================

    /// Set child alignment within container
    /// 
    /// Alignment uses -1 to 1 coordinates:
    /// - (-1, -1) = top-left
    /// - (0, 0) = center
    /// - (1, 1) = bottom-right
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Center the child (shortcut for alignment(Alignment::CENTER))
    pub fn center(mut self) -> Self {
        self.alignment = Alignment::CENTER;
        self
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    /// Calculate content size (size minus margin)
    fn content_size(&self, total_size: Size) -> Size {
        Size::new(
            (total_size.width - self.margin.horizontal_total()).max(0.0),
            (total_size.height - self.margin.vertical_total()).max(0.0),
        )
    }

    /// Calculate inner size (size minus margin and padding)
    fn inner_size(&self, total_size: Size) -> Size {
        let content = self.content_size(total_size);
        Size::new(
            (content.width - self.padding.horizontal_total()).max(0.0),
            (content.height - self.padding.vertical_total()).max(0.0),
        )
    }
}

impl Widget for Container {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        // Start with fixed dimensions or constraint max
        let mut width = self.width.unwrap_or(constraints.max_width);
        let mut height = self.height.unwrap_or(constraints.max_height);

        // If we have a child and no fixed dimensions, size to child
        if let Some(child) = &self.child {
            let available_width = width - self.padding.horizontal_total() - self.margin.horizontal_total();
            let available_height = height - self.padding.vertical_total() - self.margin.vertical_total();
            
            let child_constraints = BoxConstraints::new()
                .with_max_width(available_width.max(0.0))
                .with_max_height(available_height.max(0.0));
            
            let child_size = child.layout(child_constraints);
            
            if self.width.is_none() {
                width = child_size.width + self.padding.horizontal_total() + self.margin.horizontal_total();
            }
            if self.height.is_none() {
                height = child_size.height + self.padding.vertical_total() + self.margin.vertical_total();
            }
        }

        Size::new(
            constraints.constrain_width(width),
            constraints.constrain_height(height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        
        // Calculate content rect (inside margin)
        let content_offset = Offset::new(
            offset.dx + self.margin.left,
            offset.dy + self.margin.top,
        );
        let content_size = self.content_size(size);
        let content_rect = Rect::new(
            content_offset.dx, 
            content_offset.dy, 
            content_size.width, 
            content_size.height
        );

        // Draw background
        if let Some(color) = self.color {
            if self.border_radius > 0.0 {
                canvas.draw_rounded_rect(
                    content_rect, 
                    BorderRadius::all(self.border_radius), 
                    &Paint::fill(color)
                );
            } else {
                canvas.draw_rect(content_rect, &Paint::fill(color));
            }
        }

        // Draw border
        if let Some(border_color) = self.border_color {
            if self.border_width > 0.0 {
                if self.border_radius > 0.0 {
                    canvas.draw_rounded_rect(
                        content_rect, 
                        BorderRadius::all(self.border_radius), 
                        &Paint::stroke(border_color, self.border_width)
                    );
                } else {
                    canvas.draw_rect(content_rect, &Paint::stroke(border_color, self.border_width));
                }
            }
        }

        // Paint child
        if let Some(child) = &self.child {
            let child_size = child.layout(BoxConstraints::new());
            let inner_size = self.inner_size(size);
            
            // Compute child offset based on alignment
            let aligned_offset = self.alignment.compute_offset(inner_size, child_size);
            
            let child_position = Offset::new(
                content_offset.dx + self.padding.left + aligned_offset.dx,
                content_offset.dy + self.padding.top + aligned_offset.dy,
            );
            
            child.paint(canvas, child_position);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// SIZED BOX
// ============================================================================

/// A box with specific dimensions
/// 
/// SizedBox forces its child to have a specific size, or takes that size
/// if it has no child.
/// 
/// # Use Cases
/// 
/// - Adding fixed-size spacing
/// - Constraining a child to exact dimensions
/// - Creating placeholder boxes
/// 
/// # Example
/// 
/// ```ignore
/// // Fixed space between widgets
/// let spacer = SizedBox::width(16.0);
/// 
/// // Force child to specific size
/// let fixed_icon = SizedBox::square(24.0)
///     .child(icon);
/// ```
#[derive(Default)]
pub struct SizedBox {
    /// Width (None = size from child or 0)
    pub width: Option<f32>,
    
    /// Height (None = size from child or 0)
    pub height: Option<f32>,
    
    /// Optional child
    child: Option<BoxedWidget>,
}

impl SizedBox {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a sized box with specific dimensions
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            child: None,
        }
    }

    /// Create a square sized box
    pub fn square(size: f32) -> Self {
        Self::new(size, size)
    }

    /// Create a horizontal spacer (fixed width, shrink-wrapped height)
    pub fn width(width: f32) -> Self {
        Self {
            width: Some(width),
            height: None,
            child: None,
        }
    }

    /// Create a vertical spacer (shrink-wrapped width, fixed height)
    pub fn height(height: f32) -> Self {
        Self {
            width: None,
            height: Some(height),
            child: None,
        }
    }

    /// Create an expanding box that takes all available space
    pub fn expand() -> Self {
        Self {
            width: Some(f32::INFINITY),
            height: Some(f32::INFINITY),
            child: None,
        }
    }

    /// Create a shrink-wrapped box (sizes to child)
    pub fn shrink() -> Self {
        Self::default()
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Set child
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Set boxed child
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.child = Some(widget);
        self
    }
}

impl Widget for SizedBox {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let width = self.width.unwrap_or_else(|| {
            if let Some(child) = &self.child {
                child.layout(constraints).width
            } else {
                0.0
            }
        });
        
        let height = self.height.unwrap_or_else(|| {
            if let Some(child) = &self.child {
                child.layout(constraints).height
            } else {
                0.0
            }
        });
        
        Size::new(
            constraints.constrain_width(width),
            constraints.constrain_height(height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if let Some(child) = &self.child {
            child.paint(canvas, offset);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// PADDING
// ============================================================================

/// Widget that adds padding around its child
/// 
/// Padding creates space between the widget's edge and its child.
/// 
/// # Example
/// 
/// ```ignore
/// // Uniform padding
/// Padding::all(16.0).child(content);
/// 
/// // Horizontal and vertical
/// Padding::symmetric(24.0, 16.0).child(content);
/// 
/// // Custom each side
/// Padding::only(8.0, 16.0, 8.0, 16.0).child(content);
/// ```
#[derive(Default)]
pub struct Padding {
    /// Padding insets
    pub padding: Insets,
    
    /// Child widget
    child: Option<BoxedWidget>,
}

impl Padding {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create with uniform padding on all sides
    pub fn all(value: f32) -> Self {
        Self {
            padding: Insets::all(value),
            child: None,
        }
    }

    /// Create with symmetric horizontal and vertical padding
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            padding: Insets::symmetric(horizontal, vertical),
            child: None,
        }
    }

    /// Create with individual padding values (top, right, bottom, left)
    pub fn only(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            padding: Insets::new(top, right, bottom, left),
            child: None,
        }
    }

    /// Create from Insets
    pub fn from_insets(padding: Insets) -> Self {
        Self { padding, child: None }
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Set child
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Set boxed child
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.child = Some(widget);
        self
    }
}

impl Widget for Padding {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let child_constraints = constraints.deflate(self.padding);
        
        let child_size = if let Some(child) = &self.child {
            child.layout(child_constraints)
        } else {
            Size::ZERO
        };
        
        Size::new(
            constraints.constrain_width(child_size.width + self.padding.horizontal_total()),
            constraints.constrain_height(child_size.height + self.padding.vertical_total()),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if let Some(child) = &self.child {
            child.paint(canvas, Offset::new(
                offset.dx + self.padding.left,
                offset.dy + self.padding.top,
            ));
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// CENTER
// ============================================================================

/// Widget that centers its child
/// 
/// Center expands to fill available space and places its child
/// at the center.
/// 
/// # Example
/// 
/// ```ignore
/// Center::new().child(
///     Text::new("Centered Content")
/// );
/// ```
#[derive(Default)]
pub struct Center {
    /// Child widget
    child: Option<BoxedWidget>,
    
    /// Width factor (None = expand, Some(x) = fraction of available)
    pub width_factor: Option<f32>,
    
    /// Height factor (None = expand, Some(x) = fraction of available)
    pub height_factor: Option<f32>,
}

impl Center {
    /// Create a new center widget
    pub fn new() -> Self {
        Self::default()
    }

    /// Set child
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Set boxed child
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.child = Some(widget);
        self
    }

    /// Set width factor (0.0 to 1.0)
    /// 
    /// If set, Center will be this fraction of available width
    /// instead of expanding to fill.
    pub fn width_factor(mut self, factor: f32) -> Self {
        self.width_factor = Some(factor.clamp(0.0, 1.0));
        self
    }

    /// Set height factor (0.0 to 1.0)
    pub fn height_factor(mut self, factor: f32) -> Self {
        self.height_factor = Some(factor.clamp(0.0, 1.0));
        self
    }
}

impl Widget for Center {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let child_size = if let Some(child) = &self.child {
            child.layout(constraints.loosen())
        } else {
            Size::ZERO
        };

        let width = match self.width_factor {
            Some(factor) => (child_size.width / factor).min(constraints.max_width),
            None => constraints.biggest().width,
        };

        let height = match self.height_factor {
            Some(factor) => (child_size.height / factor).min(constraints.max_height),
            None => constraints.biggest().height,
        };

        constraints.constrain(Size::new(width, height))
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if let Some(child) = &self.child {
            let own_size = self.layout(BoxConstraints::new());
            let child_size = child.layout(BoxConstraints::new());
            
            // Compute centered offset
            let child_offset = Alignment::CENTER.compute_offset(own_size, child_size);
            
            child.paint(canvas, Offset::new(
                offset.dx + child_offset.dx,
                offset.dy + child_offset.dy,
            ));
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

    // Test widget for sizing
    struct TestBox(Size);
    
    impl Widget for TestBox {
        fn layout(&self, _: BoxConstraints) -> Size { self.0 }
        fn paint(&self, _: &mut dyn PaintCanvas, _: Offset) {}
        fn as_any(&self) -> &dyn Any { self }
    }

    #[test]
    fn test_container_fixed_size() {
        let container = Container::new()
            .width(100.0)
            .height(50.0);
        
        let size = container.layout(BoxConstraints::new());
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 50.0);
    }

    #[test]
    fn test_container_with_padding() {
        let container = Container::new()
            .width(100.0)
            .height(100.0)
            .padding_all(10.0);
        
        let size = container.layout(BoxConstraints::new());
        // Padding doesn't increase fixed size
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 100.0);
    }

    #[test]
    fn test_container_shrink_to_child() {
        let container = Container::new()
            .padding_all(10.0)
            .child(TestBox(Size::new(50.0, 30.0)));
        
        let size = container.layout(BoxConstraints::new());
        // Child + padding on each side
        assert_eq!(size.width, 70.0);  // 50 + 10 + 10
        assert_eq!(size.height, 50.0); // 30 + 10 + 10
    }

    #[test]
    fn test_sized_box_fixed() {
        let box_ = SizedBox::new(50.0, 30.0);
        let size = box_.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 50.0);
        assert_eq!(size.height, 30.0);
    }

    #[test]
    fn test_sized_box_square() {
        let box_ = SizedBox::square(64.0);
        let size = box_.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 64.0);
        assert_eq!(size.height, 64.0);
    }

    #[test]
    fn test_sized_box_width_only() {
        let box_ = SizedBox::width(100.0);
        let size = box_.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 0.0);
    }

    #[test]
    fn test_padding() {
        let padded = Padding::all(10.0)
            .child(TestBox(Size::new(40.0, 20.0)));
        
        let size = padded.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 60.0);  // 40 + 10 + 10
        assert_eq!(size.height, 40.0); // 20 + 10 + 10
    }

    #[test]
    fn test_padding_symmetric() {
        let padded = Padding::symmetric(20.0, 10.0)
            .child(TestBox(Size::new(50.0, 30.0)));
        
        let size = padded.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 90.0);  // 50 + 20 + 20
        assert_eq!(size.height, 50.0); // 30 + 10 + 10
    }

    #[test]
    fn test_center_expands() {
        let center = Center::new()
            .child(TestBox(Size::new(50.0, 30.0)));
        
        let constraints = BoxConstraints::tight(Size::new(200.0, 100.0));
        let size = center.layout(constraints);
        
        // Center expands to fill constraints
        assert_eq!(size.width, 200.0);
        assert_eq!(size.height, 100.0);
    }
}
