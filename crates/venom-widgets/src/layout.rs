//! Layout widgets - Row, Column, Stack, Flex
//!
//! These widgets arrange their children in different ways, similar to
//! Flutter's layout widgets. They implement the constraint-based layout
//! system that VenomUI uses.
//!
//! # Layout Model
//!
//! VenomUI uses a constraint-based layout model:
//! 1. Parent passes constraints (min/max width/height) to child
//! 2. Child chooses its size within those constraints
//! 3. Parent positions child
//!
//! # Available Layout Widgets
//!
//! - **Flex/Row/Column**: Linear layout with flexible sizing
//! - **Stack**: Overlapping children (like CSS absolute positioning)
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Flex, Container, MainAxisAlignment};
//!
//! // Create a row with three items
//! let row = Flex::row()
//!     .spacing(16.0)
//!     .main_axis_alignment(MainAxisAlignment::SpaceBetween)
//!     .child(Container::new().width(50.0).height(50.0).color(Color::RED))
//!     .child(Container::new().width(50.0).height(50.0).color(Color::GREEN))
//!     .child(Container::new().width(50.0).height(50.0).color(Color::BLUE));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, MainAxisAlignment, CrossAxisAlignment};
use venom_render::PaintCanvas;
use crate::{Widget, BoxedWidget};

// ============================================================================
// AXIS
// ============================================================================

/// Direction of layout for Flex widgets
/// 
/// # Example
/// 
/// ```
/// use venom_widgets::Axis;
/// 
/// let horizontal = Axis::Horizontal; // Row-like (left to right)
/// let vertical = Axis::Vertical;     // Column-like (top to bottom)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Axis {
    /// Left to right (Row)
    #[default]
    Horizontal,
    /// Top to bottom (Column)
    Vertical,
}

impl Axis {
    /// Get the opposite axis
    pub fn flip(self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

// ============================================================================
// FLEX
// ============================================================================

/// Flexible layout widget - base for Row and Column
/// 
/// Flex arranges children along a single axis (horizontal or vertical).
/// It supports:
/// - Spacing between children
/// - Main axis alignment (start, end, center, space-between, etc.)
/// - Cross axis alignment (start, end, center, stretch)
/// 
/// # Builder Pattern
/// 
/// ```ignore
/// let flex = Flex::row()
///     .spacing(16.0)
///     .main_axis_alignment(MainAxisAlignment::Center)
///     .cross_axis_alignment(CrossAxisAlignment::Stretch)
///     .child(widget1)
///     .child(widget2)
///     .child(widget3);
/// ```
/// 
/// # Layout Algorithm
/// 
/// 1. Layout each child with loosened constraints
/// 2. Sum up main axis sizes + spacing
/// 3. Distribute remaining space according to main_axis_alignment
/// 4. Position children along main axis
/// 5. Align children on cross axis according to cross_axis_alignment
#[derive(Default)]
pub struct Flex {
    /// Layout direction
    pub axis: Axis,
    
    /// Space between children (in pixels)
    pub spacing: f32,
    
    /// How to align children along the main axis
    pub main_axis_alignment: MainAxisAlignment,
    
    /// How to align children along the cross axis
    pub cross_axis_alignment: CrossAxisAlignment,
    
    /// Children widgets
    children: Vec<BoxedWidget>,
}

impl Flex {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a new horizontal flex (Row)
    /// 
    /// Children are laid out from left to right.
    pub fn row() -> Self {
        Self {
            axis: Axis::Horizontal,
            ..Default::default()
        }
    }

    /// Create a new vertical flex (Column)
    /// 
    /// Children are laid out from top to bottom.
    pub fn column() -> Self {
        Self {
            axis: Axis::Vertical,
            ..Default::default()
        }
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Set spacing between children
    /// 
    /// # Arguments
    /// 
    /// * `spacing` - Gap between children in pixels
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    /// Set main axis alignment
    /// 
    /// Controls how children are distributed along the main axis
    /// when there is extra space.
    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set cross axis alignment
    /// 
    /// Controls how children are positioned along the cross axis.
    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    /// Add a child widget
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Add a boxed child widget
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }

    /// Add multiple children
    pub fn children<I, W>(mut self, widgets: I) -> Self
    where
        I: IntoIterator<Item = W>,
        W: Widget,
    {
        for widget in widgets {
            self.children.push(Box::new(widget));
        }
        self
    }

    // ========================================================================
    // LAYOUT HELPERS
    // ========================================================================

    /// Internal layout implementation
    /// 
    /// Returns (own_size, [(child_size, child_offset), ...])
    fn layout_children(&self, constraints: BoxConstraints) -> (Size, Vec<(Size, Offset)>) {
        if self.children.is_empty() {
            return (Size::ZERO, Vec::new());
        }

        let mut child_layouts = Vec::with_capacity(self.children.len());
        let mut total_main = 0.0f32;
        let mut max_cross = 0.0f32;

        // First pass: layout children to get their natural sizes
        for child in &self.children {
            let child_size = child.layout(constraints.loosen());
            
            match self.axis {
                Axis::Horizontal => {
                    total_main += child_size.width;
                    max_cross = max_cross.max(child_size.height);
                }
                Axis::Vertical => {
                    total_main += child_size.height;
                    max_cross = max_cross.max(child_size.width);
                }
            }
            
            child_layouts.push((child_size, Offset::ZERO));
        }

        // Add spacing between children
        let spacing_total = self.spacing * (self.children.len() - 1) as f32;
        total_main += spacing_total;

        // Calculate own size
        let own_size = match self.axis {
            Axis::Horizontal => Size::new(
                constraints.constrain_width(total_main),
                constraints.constrain_height(max_cross),
            ),
            Axis::Vertical => Size::new(
                constraints.constrain_width(max_cross),
                constraints.constrain_height(total_main),
            ),
        };

        // Calculate extra space for alignment
        let extra_space = match self.axis {
            Axis::Horizontal => (own_size.width - total_main).max(0.0),
            Axis::Vertical => (own_size.height - total_main).max(0.0),
        };

        // Compute initial offset and spacing based on alignment
        let (initial_offset, between_spacing) = self.compute_alignment_offsets(extra_space);

        // Second pass: calculate offsets
        let mut main_offset = initial_offset;
        for (i, (child_size, offset)) in child_layouts.iter_mut().enumerate() {
            // Cross axis alignment
            let cross_offset = self.compute_cross_offset(*child_size, own_size);

            // Set offset based on axis
            *offset = match self.axis {
                Axis::Horizontal => Offset::new(main_offset, cross_offset),
                Axis::Vertical => Offset::new(cross_offset, main_offset),
            };

            // Move to next position
            main_offset += match self.axis {
                Axis::Horizontal => child_size.width,
                Axis::Vertical => child_size.height,
            };
            
            if i < self.children.len() - 1 {
                main_offset += between_spacing;
            }
        }

        (own_size, child_layouts)
    }

    /// Compute alignment offsets based on MainAxisAlignment
    fn compute_alignment_offsets(&self, extra_space: f32) -> (f32, f32) {
        match self.main_axis_alignment {
            MainAxisAlignment::Start => (0.0, self.spacing),
            MainAxisAlignment::End => (extra_space, self.spacing),
            MainAxisAlignment::Center => (extra_space / 2.0, self.spacing),
            MainAxisAlignment::SpaceBetween => {
                if self.children.len() > 1 {
                    let between = extra_space / (self.children.len() - 1) as f32;
                    (0.0, self.spacing + between)
                } else {
                    (0.0, self.spacing)
                }
            }
            MainAxisAlignment::SpaceAround => {
                let space = extra_space / self.children.len() as f32;
                (space / 2.0, self.spacing + space)
            }
            MainAxisAlignment::SpaceEvenly => {
                let space = extra_space / (self.children.len() + 1) as f32;
                (space, self.spacing + space)
            }
        }
    }

    /// Compute cross axis offset for a child
    fn compute_cross_offset(&self, child_size: Size, own_size: Size) -> f32 {
        let (cross_max, child_cross) = match self.axis {
            Axis::Horizontal => (own_size.height, child_size.height),
            Axis::Vertical => (own_size.width, child_size.width),
        };

        match self.cross_axis_alignment {
            CrossAxisAlignment::Start => 0.0,
            CrossAxisAlignment::End => cross_max - child_cross,
            CrossAxisAlignment::Center => (cross_max - child_cross) / 2.0,
            CrossAxisAlignment::Stretch => 0.0, // Child should fill cross axis
        }
    }
}

impl Widget for Flex {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.layout_children(constraints).0
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let (_, child_layouts) = self.layout_children(BoxConstraints::new());
        
        for (i, child) in self.children.iter().enumerate() {
            if let Some((_, child_offset)) = child_layouts.get(i) {
                let final_offset = Offset::new(
                    offset.dx + child_offset.dx,
                    offset.dy + child_offset.dy,
                );
                child.paint(canvas, final_offset);
            }
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.children
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ROW AND COLUMN TYPE ALIASES
// ============================================================================

/// Horizontal layout widget (alias for Flex::row())
/// 
/// Lays out children from left to right.
/// 
/// # Example
/// 
/// ```ignore
/// let row = Row::new()
///     .spacing(8.0)
///     .child(icon)
///     .child(label);
/// ```
pub type Row = Flex;

impl Row {
    /// Create a new row (convenience method)
    pub fn new() -> Self {
        Flex::row()
    }
}

/// Vertical layout widget (alias for Flex::column())
/// 
/// Lays out children from top to bottom.
/// 
/// # Example
/// 
/// ```ignore
/// let column = Column::new()
///     .spacing(16.0)
///     .child(header)
///     .child(content)
///     .child(footer);
/// ```
pub type Column = Flex;

// ============================================================================
// STACK
// ============================================================================

/// Stack widget - overlays children on top of each other
/// 
/// Similar to CSS absolute/relative positioning, or Flutter's Stack widget.
/// The first child is at the bottom, last child is on top.
/// 
/// # Example
/// 
/// ```ignore
/// let stack = Stack::new()
///     .alignment(Alignment::CENTER)
///     .child(background_image)
///     .child(overlay_text);
/// ```
/// 
/// # Layout
/// 
/// Stack takes the size of its largest child (unless constrained).
/// Each child is positioned according to the stack's alignment.
#[derive(Default)]
pub struct Stack {
    /// Children widgets (first = bottom, last = top)
    children: Vec<BoxedWidget>,
    
    /// Default alignment for children
    pub alignment: venom_core::Alignment,
}

impl Stack {
    /// Create a new stack
    pub fn new() -> Self {
        Self::default()
    }

    /// Set alignment for children
    /// 
    /// Children without explicit positioning will use this alignment.
    pub fn alignment(mut self, alignment: venom_core::Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Add a child widget (stacked on top of previous children)
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Add a boxed child widget
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.children.push(widget);
        self
    }
}

impl Widget for Stack {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let mut max_width = 0.0f32;
        let mut max_height = 0.0f32;

        // Stack takes the size of its largest child
        for child in &self.children {
            let child_size = child.layout(constraints.loosen());
            max_width = max_width.max(child_size.width);
            max_height = max_height.max(child_size.height);
        }

        Size::new(
            constraints.constrain_width(max_width),
            constraints.constrain_height(max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let own_size = self.layout(BoxConstraints::new());
        
        // Paint children in order (first = bottom, last = top)
        for child in &self.children {
            let child_size = child.layout(BoxConstraints::new());
            let child_offset = self.alignment.compute_offset(own_size, child_size);
            
            child.paint(canvas, Offset::new(
                offset.dx + child_offset.dx,
                offset.dy + child_offset.dy,
            ));
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.children
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

    // Simple sized box widget for testing
    struct TestBox(Size);
    
    impl Widget for TestBox {
        fn layout(&self, _: BoxConstraints) -> Size { 
            self.0 
        }
        
        fn paint(&self, _: &mut dyn PaintCanvas, _: Offset) {}
        
        fn as_any(&self) -> &dyn Any { 
            self 
        }
    }

    #[test]
    fn test_axis_flip() {
        assert_eq!(Axis::Horizontal.flip(), Axis::Vertical);
        assert_eq!(Axis::Vertical.flip(), Axis::Horizontal);
    }

    #[test]
    fn test_row_layout_sizing() {
        let row = Flex::row()
            .spacing(10.0)
            .child(TestBox(Size::new(50.0, 30.0)))
            .child(TestBox(Size::new(50.0, 40.0)));
        
        let size = row.layout(BoxConstraints::new());
        
        // Width: 50 + 10 + 50 = 110
        assert_eq!(size.width, 110.0);
        // Height: max(30, 40) = 40
        assert_eq!(size.height, 40.0);
    }

    #[test]
    fn test_column_layout_sizing() {
        let column = Flex::column()
            .spacing(5.0)
            .child(TestBox(Size::new(100.0, 30.0)))
            .child(TestBox(Size::new(80.0, 30.0)));
        
        let size = column.layout(BoxConstraints::new());
        
        // Width: max(100, 80) = 100
        assert_eq!(size.width, 100.0);
        // Height: 30 + 5 + 30 = 65
        assert_eq!(size.height, 65.0);
    }

    #[test]
    fn test_empty_flex() {
        let row = Flex::row();
        let size = row.layout(BoxConstraints::new());
        
        assert_eq!(size, Size::ZERO);
    }

    #[test]
    fn test_single_child_flex() {
        let row = Flex::row()
            .spacing(10.0) // Spacing shouldn't affect single child
            .child(TestBox(Size::new(50.0, 30.0)));
        
        let size = row.layout(BoxConstraints::new());
        
        assert_eq!(size.width, 50.0);
        assert_eq!(size.height, 30.0);
    }

    #[test]
    fn test_stack_layout_sizing() {
        let stack = Stack::new()
            .child(TestBox(Size::new(100.0, 100.0)))
            .child(TestBox(Size::new(50.0, 50.0)));
        
        let size = stack.layout(BoxConstraints::new());
        
        // Stack takes size of largest child
        assert_eq!(size.width, 100.0);
        assert_eq!(size.height, 100.0);
    }

    #[test]
    fn test_empty_stack() {
        let stack = Stack::new();
        let size = stack.layout(BoxConstraints::new());
        
        assert_eq!(size, Size::ZERO);
    }

    #[test]
    fn test_constrained_flex() {
        let row = Flex::row()
            .child(TestBox(Size::new(1000.0, 50.0))); // Very wide
        
        let constraints = BoxConstraints::new()
            .with_max_width(500.0);
        
        let size = row.layout(constraints);
        
        // Should be constrained to max width
        assert_eq!(size.width, 500.0);
    }
}
