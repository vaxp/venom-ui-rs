//! Element - Mutable widget instances in the tree
//!
//! Elements are the runtime representation of widgets.
//! They hold state and manage the widget lifecycle.
//!
//! # Widget vs Element
//!
//! - **Widget**: Immutable configuration (what to build)
//! - **Element**: Mutable instance (the built thing)
//!
//! This separation allows:
//! - Efficient diffing (compare widgets to see what changed)
//! - State preservation (elements keep state across rebuilds)
//! - Lazy updates (only update what changed)
//!
//! # Element Tree
//!
//! The element tree mirrors the widget tree but:
//! - Persists across frames
//! - Holds layout results
//! - Manages lifecycle

use std::collections::HashMap;
use venom_core::{BoxConstraints, Size, Offset};
use venom_render::PaintCanvas;
use crate::{Widget, BoxedWidget, WidgetId, LayoutResult};

// ============================================================================
// ELEMENT
// ============================================================================

/// An element in the widget tree
/// 
/// Elements are the mutable counterparts to immutable widgets.
/// They hold:
/// - The widget configuration
/// - Layout results (size and position)
/// - Child elements
/// - State (for stateful widgets)
/// 
/// # Lifecycle
/// 
/// 1. Created from a widget via `Element::new()`
/// 2. Layout is performed via `perform_layout()`
/// 3. Painting is done via `paint()`
/// 4. When parent rebuilds, element is updated or replaced
/// 
/// # Example
/// 
/// ```ignore
/// let widget = Container::new().width(100.0).height(50.0);
/// let mut element = Element::new(widget);
/// 
/// // Layout
/// let size = element.perform_layout(BoxConstraints::tight(Size::new(100.0, 50.0)));
/// 
/// // Paint
/// element.paint(&mut canvas, Offset::ZERO);
/// ```
pub struct Element {
    /// Unique ID for this element
    pub id: WidgetId,
    
    /// The widget configuration
    widget: BoxedWidget,
    
    /// Layout result after layout phase
    layout: LayoutResult,
    
    /// Child elements
    children: Vec<Element>,
    
    /// Whether this element needs layout
    needs_layout: bool,
    
    /// Whether this element needs paint
    needs_paint: bool,
}

impl Element {
    /// Create a new element from a widget
    /// 
    /// The element is initially marked as needing layout.
    pub fn new<W: Widget>(widget: W) -> Self {
        Self {
            id: WidgetId::new(),
            widget: Box::new(widget),
            layout: LayoutResult::default(),
            children: Vec::new(),
            needs_layout: true,
            needs_paint: true,
        }
    }

    /// Create from a boxed widget
    pub fn from_boxed(widget: BoxedWidget) -> Self {
        Self {
            id: WidgetId::new(),
            widget,
            layout: LayoutResult::default(),
            children: Vec::new(),
            needs_layout: true,
            needs_paint: true,
        }
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get the widget
    pub fn widget(&self) -> &dyn Widget {
        self.widget.as_ref()
    }

    /// Get the layout result
    pub fn layout(&self) -> &LayoutResult {
        &self.layout
    }

    /// Get the computed size
    pub fn size(&self) -> Size {
        self.layout.size
    }

    /// Get the offset from parent
    pub fn offset(&self) -> Offset {
        self.layout.offset
    }

    /// Get children
    pub fn children(&self) -> &[Element] {
        &self.children
    }

    /// Get mutable children
    pub fn children_mut(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }

    /// Check if this element needs layout
    pub fn needs_layout(&self) -> bool {
        self.needs_layout
    }

    /// Check if this element needs paint
    pub fn needs_paint(&self) -> bool {
        self.needs_paint
    }

    // ========================================================================
    // TREE MANIPULATION
    // ========================================================================

    /// Add a child element
    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
    }

    /// Remove all children
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    /// Find a child element by ID
    pub fn find_child(&self, id: WidgetId) -> Option<&Element> {
        for child in &self.children {
            if child.id == id {
                return Some(child);
            }
            if let Some(found) = child.find_child(id) {
                return Some(found);
            }
        }
        None
    }

    // ========================================================================
    // LAYOUT
    // ========================================================================

    /// Perform layout on this element
    /// 
    /// Calculates the size based on the widget's layout method
    /// and the given constraints.
    /// 
    /// # Returns
    /// 
    /// The computed size of this element
    pub fn perform_layout(&mut self, constraints: BoxConstraints) -> Size {
        let size = self.widget.layout(constraints);
        self.layout.size = size;
        self.needs_layout = false;
        self.needs_paint = true;
        size
    }

    /// Set the offset (called by parent during layout)
    pub fn set_offset(&mut self, offset: Offset) {
        self.layout.offset = offset;
    }

    /// Mark this element as needing layout
    pub fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }

    /// Mark this element as needing paint
    pub fn mark_needs_paint(&mut self) {
        self.needs_paint = true;
    }

    // ========================================================================
    // PAINTING
    // ========================================================================

    /// Paint this element and its children
    /// 
    /// Paints are performed in tree order (parent first, then children).
    /// 
    /// # Arguments
    /// 
    /// * `canvas` - The canvas to paint on
    /// * `parent_offset` - The offset of the parent element
    pub fn paint(&self, canvas: &mut dyn PaintCanvas, parent_offset: Offset) {
        let offset = Offset::new(
            parent_offset.dx + self.layout.offset.dx,
            parent_offset.dy + self.layout.offset.dy,
        );
        
        // Paint self
        self.widget.paint(canvas, offset);
        
        // Paint children
        for child in &self.children {
            child.paint(canvas, offset);
        }
    }

    // ========================================================================
    // HIT TESTING
    // ========================================================================

    /// Hit test at the given position
    /// 
    /// Returns the deepest element that contains the point.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&Element> {
        // Check children first (in reverse order - topmost first)
        for child in self.children.iter().rev() {
            if let Some(hit) = child.hit_test(x, y) {
                return Some(hit);
            }
        }
        
        // Check self
        if self.layout.contains(x, y) {
            Some(self)
        } else {
            None
        }
    }
}

// ============================================================================
// ELEMENT TREE
// ============================================================================

/// The root of the element tree
/// 
/// The ElementTree manages the entire UI tree and provides:
/// - Root element access
/// - Layout orchestration
/// - Paint orchestration
/// - Element lookup by ID
/// 
/// # Example
/// 
/// ```ignore
/// let mut tree = ElementTree::new();
/// tree.set_root(my_root_widget);
/// 
/// // Layout with screen constraints
/// tree.layout(BoxConstraints::tight(Size::new(1920.0, 1080.0)));
/// 
/// // Paint
/// tree.paint(&mut canvas);
/// ```
pub struct ElementTree {
    /// Root element
    root: Option<Element>,
    
    /// Cached element lookup by ID (for fast access)
    id_cache: HashMap<WidgetId, ()>, // TODO: proper lookup
}

impl ElementTree {
    /// Create a new empty tree
    pub fn new() -> Self {
        Self {
            root: None,
            id_cache: HashMap::new(),
        }
    }

    /// Set the root widget
    /// 
    /// This creates a new element tree from the given widget.
    pub fn set_root<W: Widget>(&mut self, widget: W) {
        self.root = Some(Element::new(widget));
        self.id_cache.clear();
    }

    /// Set root from boxed widget
    pub fn set_root_boxed(&mut self, widget: BoxedWidget) {
        self.root = Some(Element::from_boxed(widget));
        self.id_cache.clear();
    }

    /// Get the root element
    pub fn root(&self) -> Option<&Element> {
        self.root.as_ref()
    }

    /// Get mutable root
    pub fn root_mut(&mut self) -> Option<&mut Element> {
        self.root.as_mut()
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Perform layout from root
    /// 
    /// Lays out the entire tree with the given constraints.
    pub fn layout(&mut self, constraints: BoxConstraints) {
        if let Some(root) = &mut self.root {
            root.perform_layout(constraints);
        }
    }

    /// Paint the entire tree
    pub fn paint(&self, canvas: &mut dyn PaintCanvas) {
        if let Some(root) = &self.root {
            root.paint(canvas, Offset::ZERO);
        }
    }

    /// Hit test at position
    pub fn hit_test(&self, x: f32, y: f32) -> Option<&Element> {
        self.root.as_ref().and_then(|root| root.hit_test(x, y))
    }

    /// Find element by ID
    pub fn find_element(&self, id: WidgetId) -> Option<&Element> {
        self.root.as_ref().and_then(|root| {
            if root.id == id {
                Some(root)
            } else {
                root.find_child(id)
            }
        })
    }
}

impl Default for ElementTree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use venom_core::Color;
    use std::any::Any;

    // Simple test widget
    struct TestWidget {
        size: Size,
    }

    impl Widget for TestWidget {
        fn layout(&self, _constraints: BoxConstraints) -> Size {
            self.size
        }

        fn paint(&self, _canvas: &mut dyn PaintCanvas, _offset: Offset) {
            // No-op for testing
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_element_creation() {
        let widget = TestWidget { size: Size::new(100.0, 50.0) };
        let element = Element::new(widget);
        
        // Before layout
        assert_eq!(element.layout().size, Size::ZERO);
        assert!(element.needs_layout());
    }

    #[test]
    fn test_element_layout() {
        let widget = TestWidget { size: Size::new(100.0, 50.0) };
        let mut element = Element::new(widget);
        
        let size = element.perform_layout(BoxConstraints::new());
        
        assert_eq!(size, Size::new(100.0, 50.0));
        assert_eq!(element.layout().size, Size::new(100.0, 50.0));
        assert!(!element.needs_layout());
    }

    #[test]
    fn test_element_offset() {
        let widget = TestWidget { size: Size::new(100.0, 50.0) };
        let mut element = Element::new(widget);
        
        element.set_offset(Offset::new(10.0, 20.0));
        
        assert_eq!(element.offset().dx, 10.0);
        assert_eq!(element.offset().dy, 20.0);
    }

    #[test]
    fn test_element_tree_creation() {
        let widget = TestWidget { size: Size::new(800.0, 600.0) };
        let mut tree = ElementTree::new();
        
        assert!(tree.is_empty());
        
        tree.set_root(widget);
        
        assert!(!tree.is_empty());
        assert!(tree.root().is_some());
    }

    #[test]
    fn test_element_tree_layout() {
        let widget = TestWidget { size: Size::new(800.0, 600.0) };
        let mut tree = ElementTree::new();
        tree.set_root(widget);
        
        tree.layout(BoxConstraints::loose(Size::new(1920.0, 1080.0)));
        
        let root = tree.root().unwrap();
        assert_eq!(root.size(), Size::new(800.0, 600.0));
    }

    #[test]
    fn test_element_children() {
        let parent = TestWidget { size: Size::new(200.0, 200.0) };
        let child = TestWidget { size: Size::new(50.0, 50.0) };
        
        let mut parent_element = Element::new(parent);
        let child_element = Element::new(child);
        
        parent_element.add_child(child_element);
        
        assert_eq!(parent_element.children().len(), 1);
    }
}
