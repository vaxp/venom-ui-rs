//! Widget - Core widget trait
//!
//! Widgets are immutable descriptions of the UI.
//! They are used to build elements which hold mutable state.
//!
//! # Architecture
//!
//! VenomUI follows Flutter's widget philosophy:
//! - **Widgets** are configuration (immutable)
//! - **Elements** are instances (mutable, hold state)
//! - **RenderObjects** handle layout and painting
//!
//! This separation allows for efficient updates and reconciliation.

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Rect};
use venom_render::PaintCanvas;

// ============================================================================
// WIDGET ID
// ============================================================================

/// Unique identifier for a widget instance
/// 
/// Widget IDs are automatically generated and are unique across the
/// entire application lifetime. They are used for:
/// - Widget tree reconciliation
/// - State management
/// - Focus handling
/// - Accessibility
/// 
/// # Example
/// ```
/// use venom_widgets::WidgetId;
/// 
/// let id1 = WidgetId::new();
/// let id2 = WidgetId::new();
/// 
/// assert_ne!(id1, id2); // Each ID is unique
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a new unique widget ID
    /// 
    /// IDs are generated using an atomic counter to ensure uniqueness
    /// across threads.
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    /// 
    /// This is useful for debugging and logging.
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// WIDGET TRAIT
// ============================================================================

/// Core widget trait - all UI components implement this
/// 
/// Widgets are the building blocks of VenomUI applications. They are:
/// - **Immutable**: Widgets only hold configuration, not state
/// - **Hierarchical**: Widgets can have child widgets
/// - **Cheap to create**: Widgets are lightweight configuration objects
/// 
/// # Lifecycle
/// 
/// 1. Widget is created with configuration
/// 2. `layout()` is called to determine size
/// 3. `paint()` is called to render the widget
/// 4. When configuration changes, a new widget is created
/// 
/// # Implementing a Custom Widget
/// 
/// ```ignore
/// use venom_widgets::{Widget, BoxConstraints, Size, Offset};
/// use venom_render::PaintCanvas;
/// use venom_core::Color;
/// 
/// struct ColoredBox {
///     color: Color,
///     width: f32,
///     height: f32,
/// }
/// 
/// impl ColoredBox {
///     pub fn new(color: Color, width: f32, height: f32) -> Self {
///         Self { color, width, height }
///     }
/// }
/// 
/// impl Widget for ColoredBox {
///     fn layout(&self, constraints: BoxConstraints) -> Size {
///         Size::new(
///             constraints.constrain_width(self.width),
///             constraints.constrain_height(self.height),
///         )
///     }
///     
///     fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
///         let rect = Rect::new(offset.dx, offset.dy, self.width, self.height);
///         canvas.draw_rect(rect, &Paint::fill(self.color));
///     }
///     
///     fn as_any(&self) -> &dyn std::any::Any {
///         self
///     }
/// }
/// ```
/// 
/// # Layout Protocol
/// 
/// Layout follows Flutter's constraint-based system:
/// 1. Parent passes constraints (min/max width/height) to child
/// 2. Child chooses its size within constraints
/// 3. Parent positions child
/// 
/// # Paint Protocol
/// 
/// Painting is relative to the widget's position:
/// 1. Parent calculates child's offset
/// 2. Paint is called with that offset
/// 3. Widget paints at (offset.dx, offset.dy)
pub trait Widget: Send + Sync + 'static {
    /// Calculate the size of this widget given constraints
    /// 
    /// This is called during the layout phase. The widget must return
    /// a size that satisfies the given constraints.
    /// 
    /// # Arguments
    /// 
    /// * `constraints` - The minimum and maximum size constraints
    /// 
    /// # Returns
    /// 
    /// The actual size this widget wants to be
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// fn layout(&self, constraints: BoxConstraints) -> Size {
    ///     // Always be 100x50, respecting constraints
    ///     Size::new(
    ///         constraints.constrain_width(100.0),
    ///         constraints.constrain_height(50.0),
    ///     )
    /// }
    /// ```
    fn layout(&self, constraints: BoxConstraints) -> Size;

    /// Paint this widget to the canvas
    /// 
    /// This is called during the paint phase. The widget should draw
    /// itself at the given offset.
    /// 
    /// # Arguments
    /// 
    /// * `canvas` - The canvas to paint on (dyn PaintCanvas for type erasure)
    /// * `offset` - The top-left position where drawing should start
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
    ///     let rect = Rect::new(offset.dx, offset.dy, self.width, self.height);
    ///     canvas.draw_rect(rect, &Paint::fill(self.color));
    /// }
    /// ```
    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset);

    /// Get the children of this widget (if any)
    /// 
    /// Override this for composite widgets that have children.
    /// The default implementation returns an empty slice.
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }

    /// Get the widget type name (for debugging)
    /// 
    /// This is useful for debugging and error messages.
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Downcast to concrete type
    /// 
    /// Used for widget inspection and debugging tools.
    fn as_any(&self) -> &dyn Any;
}

// ============================================================================
// BOX WIDGET
// ============================================================================

/// A boxed widget for type-erased storage
/// 
/// This allows storing different widget types in containers.
pub type BoxedWidget = Box<dyn Widget>;

// ============================================================================
// LAYOUT RESULT
// ============================================================================

/// Result of layout - contains size and position information
/// 
/// This holds the computed layout for a widget instance:
/// - `size` is determined by the widget's `layout()` method
/// - `offset` is assigned by the parent during layout
/// 
/// # Example
/// 
/// ```
/// use venom_widgets::LayoutResult;
/// use venom_core::{Size, Offset};
/// 
/// let result = LayoutResult::new(
///     Size::new(100.0, 50.0),
///     Offset::new(10.0, 20.0),
/// );
/// 
/// let rect = result.rect();
/// assert_eq!(rect.x, 10.0);
/// assert_eq!(rect.y, 20.0);
/// assert_eq!(rect.width, 100.0);
/// assert_eq!(rect.height, 50.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutResult {
    /// Size determined by layout
    pub size: Size,
    /// Offset from parent (set by parent during layout)
    pub offset: Offset,
}

impl LayoutResult {
    /// Create a new layout result with size and offset
    pub fn new(size: Size, offset: Offset) -> Self {
        Self { size, offset }
    }

    /// Create layout result with just size (offset = 0,0)
    pub fn sized(size: Size) -> Self {
        Self { size, offset: Offset::ZERO }
    }

    /// Get the bounding rect
    /// 
    /// Combines offset and size into a Rect for hit testing and painting.
    pub fn rect(&self) -> Rect {
        Rect::new(self.offset.dx, self.offset.dy, self.size.width, self.size.height)
    }

    /// Check if a point is inside this layout result
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect().contains(venom_core::Point::new(x, y))
    }

    /// Get the center point
    pub fn center(&self) -> (f32, f32) {
        (
            self.offset.dx + self.size.width / 2.0,
            self.offset.dy + self.size.height / 2.0,
        )
    }
}

// ============================================================================
// WIDGET BUILDER TRAIT
// ============================================================================

/// Trait for widgets that can be built using builder pattern
/// 
/// This enables the fluent, no-macros API that VenomUI is designed around.
/// 
/// # Example
/// 
/// ```ignore
/// let column = Column::new()
///     .spacing(16.0)
///     .child(Text::new("Title"))
///     .child(Text::new("Subtitle"))
///     .children(items.into_iter().map(|i| Text::new(i)));
/// ```
pub trait WidgetBuilder: Sized {
    /// Add a child widget
    fn child<W: Widget>(self, widget: W) -> Self;

    /// Add multiple children from an iterator
    fn children<I>(self, widgets: I) -> Self
    where
        I: IntoIterator<Item = BoxedWidget>;
}

// ============================================================================
// PAINT CONTEXT
// ============================================================================

/// Context passed during paint operations
/// 
/// This wraps the canvas with additional state like the current offset,
/// making it easier to paint relative to parent positions.
pub struct PaintContext<'a> {
    /// The canvas to draw on
    pub canvas: &'a mut dyn PaintCanvas,
    /// Current offset from root
    pub offset: Offset,
}

impl<'a> PaintContext<'a> {
    /// Create a new paint context
    pub fn new(canvas: &'a mut dyn PaintCanvas, offset: Offset) -> Self {
        Self { canvas, offset }
    }

    /// Create a child context with additional offset
    /// 
    /// Used when painting children at relative positions.
    pub fn with_offset(&mut self, additional: Offset) -> PaintContext<'_> {
        PaintContext {
            canvas: self.canvas,
            offset: Offset::new(
                self.offset.dx + additional.dx,
                self.offset.dy + additional.dy,
            ),
        }
    }

    /// Get the absolute position for a relative offset
    pub fn absolute_offset(&self, relative: Offset) -> Offset {
        Offset::new(
            self.offset.dx + relative.dx,
            self.offset.dy + relative.dy,
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id_unique() {
        let id1 = WidgetId::new();
        let id2 = WidgetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_widget_id_value() {
        let id = WidgetId::new();
        assert!(id.value() > 0);
    }

    #[test]
    fn test_layout_result_new() {
        let result = LayoutResult::new(
            Size::new(100.0, 50.0),
            Offset::new(10.0, 20.0),
        );
        
        assert_eq!(result.size.width, 100.0);
        assert_eq!(result.size.height, 50.0);
        assert_eq!(result.offset.dx, 10.0);
        assert_eq!(result.offset.dy, 20.0);
    }

    #[test]
    fn test_layout_result_rect() {
        let result = LayoutResult::new(
            Size::new(100.0, 50.0),
            Offset::new(10.0, 20.0),
        );
        
        let rect = result.rect();
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_layout_result_contains() {
        let result = LayoutResult::new(
            Size::new(100.0, 50.0),
            Offset::new(10.0, 20.0),
        );
        
        // Inside
        assert!(result.contains(50.0, 40.0));
        // Outside
        assert!(!result.contains(5.0, 5.0));
        assert!(!result.contains(200.0, 200.0));
    }

    #[test]
    fn test_layout_result_center() {
        let result = LayoutResult::new(
            Size::new(100.0, 50.0),
            Offset::new(10.0, 20.0),
        );
        
        let (cx, cy) = result.center();
        assert_eq!(cx, 60.0); // 10 + 100/2
        assert_eq!(cy, 45.0); // 20 + 50/2
    }
}
