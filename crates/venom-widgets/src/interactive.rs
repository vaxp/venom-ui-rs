//! Interactive Widget System - Event handling and state management
//!
//! This module provides the bridge between raw input events and widget interactions.

use venom_core::{Point, Rect, Offset, Size};
use crate::event::{Event, MouseEvent, MouseButton};

// ============================================================================
// HIT TEST RESULT
// ============================================================================

/// Result of a hit test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestResult {
    /// The point is inside this widget and it handles the event
    Hit,
    /// The point is inside but the widget doesn't handle events (transparent)
    Transparent,
    /// The point is outside this widget
    Miss,
}

impl HitTestResult {
    /// Check if this is a hit
    pub fn is_hit(&self) -> bool {
        matches!(self, HitTestResult::Hit)
    }
}

// ============================================================================
// EVENT RESPONSE
// ============================================================================

/// How a widget responds to an event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventResponse {
    /// Event was handled, stop propagation
    Handled,
    /// Event was not handled, continue propagation
    #[default]
    Ignored,
    /// Request focus
    RequestFocus,
    /// Release focus
    ReleaseFocus,
}

impl EventResponse {
    /// Check if the event was handled
    pub fn is_handled(&self) -> bool {
        matches!(self, EventResponse::Handled)
    }
}

// ============================================================================
// WIDGET BOUNDS
// ============================================================================

/// Bounds of a widget for hit testing
#[derive(Debug, Clone, Copy, Default)]
pub struct WidgetBounds {
    /// Position relative to parent
    pub offset: Offset,
    /// Size of the widget
    pub size: Size,
}

impl WidgetBounds {
    /// Create new bounds
    pub fn new(offset: Offset, size: Size) -> Self {
        Self { offset, size }
    }

    /// Create from x, y, width, height
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            offset: Offset::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Get as rect
    pub fn rect(&self) -> Rect {
        Rect::new(self.offset.dx, self.offset.dy, self.size.width, self.size.height)
    }

    /// Check if point is inside
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.offset.dx
            && point.x <= self.offset.dx + self.size.width
            && point.y >= self.offset.dy
            && point.y <= self.offset.dy + self.size.height
    }

    /// Convert global point to local
    pub fn to_local(&self, global: Point) -> Point {
        Point::new(global.x - self.offset.dx, global.y - self.offset.dy)
    }
}

// ============================================================================
// INTERACTIVE WIDGET TRAIT
// ============================================================================

/// Trait for widgets that can handle user interaction
/// 
/// This extends the basic Widget trait with event handling capabilities.
/// Widgets that want to respond to mouse clicks, keyboard input, etc.
/// should implement this trait.
/// 
/// # Example
/// 
/// ```ignore
/// impl InteractiveWidget for MyButton {
///     fn hit_test(&self, point: Point, bounds: WidgetBounds) -> HitTestResult {
///         if bounds.contains(point) {
///             HitTestResult::Hit
///         } else {
///             HitTestResult::Miss
///         }
///     }
///     
///     fn handle_event(&mut self, event: &Event, bounds: WidgetBounds) -> EventResponse {
///         match event {
///             Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Left)) => {
///                 self.on_click();
///                 EventResponse::Handled
///             }
///             _ => EventResponse::Ignored,
///         }
///     }
/// }
/// ```
pub trait InteractiveWidget {
    /// Test if a point hits this widget
    /// 
    /// # Arguments
    /// * `point` - The point to test, in parent-relative coordinates
    /// * `bounds` - The bounds of this widget
    /// 
    /// # Returns
    /// The hit test result
    fn hit_test(&self, point: Point, bounds: WidgetBounds) -> HitTestResult {
        if bounds.contains(point) {
            HitTestResult::Hit
        } else {
            HitTestResult::Miss
        }
    }

    /// Handle an input event
    /// 
    /// # Arguments
    /// * `event` - The event to handle
    /// * `bounds` - The bounds of this widget
    /// 
    /// # Returns
    /// How the widget responded to the event
    fn handle_event(&mut self, event: &Event, bounds: WidgetBounds) -> EventResponse {
        let _ = (event, bounds); // Default: ignore all events
        EventResponse::Ignored
    }

    /// Check if this widget is focusable
    fn is_focusable(&self) -> bool {
        false
    }

    /// Called when widget gains focus
    fn on_focus(&mut self) {}

    /// Called when widget loses focus
    fn on_blur(&mut self) {}
}

// ============================================================================
// WIDGET STATE FOR INTERACTION
// ============================================================================

/// Common interaction states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InteractionState {
    /// Mouse is hovering over widget
    pub hovered: bool,
    /// Widget is being pressed
    pub pressed: bool,
    /// Widget has focus
    pub focused: bool,
    /// Widget is disabled
    pub disabled: bool,
}

impl InteractionState {
    /// Create new interaction state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if widget is in active state (pressed or focused)
    pub fn is_active(&self) -> bool {
        self.pressed || self.focused
    }

    /// Reset to default state
    pub fn reset(&mut self) {
        self.hovered = false;
        self.pressed = false;
    }
}

// ============================================================================
// WIDGET TREE NODE FOR EVENT DISPATCH
// ============================================================================

/// A node in the widget tree that can receive events
pub struct WidgetNode {
    /// Unique ID
    pub id: u64,
    /// Bounds for hit testing
    pub bounds: WidgetBounds,
    /// Whether this widget handles events
    pub interactive: bool,
    /// Child widget IDs (for traversal)
    pub children: Vec<u64>,
}

impl WidgetNode {
    /// Create a new widget node
    pub fn new(id: u64, bounds: WidgetBounds) -> Self {
        Self {
            id,
            bounds,
            interactive: true,
            children: Vec::new(),
        }
    }
}

// ============================================================================
// EVENT DISPATCHER
// ============================================================================

/// Dispatches events to widgets in the tree
pub struct EventDispatcher {
    /// Currently focused widget ID
    focused: Option<u64>,
    /// Currently hovered widget ID
    hovered: Option<u64>,
    /// Currently pressed widget ID
    pressed: Option<u64>,
    /// Last mouse position
    last_mouse_pos: Option<Point>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            focused: None,
            hovered: None,
            pressed: None,
            last_mouse_pos: None,
        }
    }

    /// Get the focused widget ID
    pub fn focused(&self) -> Option<u64> {
        self.focused
    }

    /// Get the hovered widget ID
    pub fn hovered(&self) -> Option<u64> {
        self.hovered
    }

    /// Get the pressed widget ID
    pub fn pressed(&self) -> Option<u64> {
        self.pressed
    }

    /// Set focus to a widget
    pub fn set_focus(&mut self, id: Option<u64>) {
        self.focused = id;
    }

    /// Update hover state based on mouse position
    pub fn update_hover(&mut self, mouse_pos: Point, node_id: u64, bounds: WidgetBounds) -> bool {
        let is_hovering = bounds.contains(mouse_pos);
        
        if is_hovering && self.hovered != Some(node_id) {
            self.hovered = Some(node_id);
            return true; // State changed
        } else if !is_hovering && self.hovered == Some(node_id) {
            self.hovered = None;
            return true;
        }
        
        false
    }

    /// Handle a press event
    pub fn handle_press(&mut self, mouse_pos: Point, node_id: u64, bounds: WidgetBounds) -> bool {
        if bounds.contains(mouse_pos) {
            self.pressed = Some(node_id);
            self.focused = Some(node_id);
            true
        } else {
            false
        }
    }

    /// Handle a release event
    pub fn handle_release(&mut self) -> Option<u64> {
        let was_pressed = self.pressed;
        self.pressed = None;
        was_pressed
    }

    /// Process a raw event and determine which widget should receive it
    pub fn dispatch(&mut self, event: &Event) -> Option<(u64, Event)> {
        match event {
            Event::Mouse(MouseEvent::CursorMoved { position }) => {
                self.last_mouse_pos = Some(*position);
                // Hover events would be dispatched here
                None
            }
            Event::Mouse(MouseEvent::ButtonPressed(button)) => {
                if let Some(pos) = self.last_mouse_pos {
                    // Would need widget tree to determine target
                    // For now, just track state
                }
                None
            }
            Event::Mouse(MouseEvent::ButtonReleased(button)) => {
                let released = self.handle_release();
                released.map(|id| (id, event.clone()))
            }
            _ => None,
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_bounds_contains() {
        let bounds = WidgetBounds::from_xywh(10.0, 20.0, 100.0, 50.0);
        
        assert!(bounds.contains(Point::new(50.0, 40.0)));
        assert!(!bounds.contains(Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_widget_bounds_to_local() {
        let bounds = WidgetBounds::from_xywh(10.0, 20.0, 100.0, 50.0);
        let local = bounds.to_local(Point::new(30.0, 45.0));
        
        assert_eq!(local.x, 20.0);
        assert_eq!(local.y, 25.0);
    }

    #[test]
    fn test_event_dispatcher_focus() {
        let mut dispatcher = EventDispatcher::new();
        
        assert!(dispatcher.focused().is_none());
        
        dispatcher.set_focus(Some(42));
        assert_eq!(dispatcher.focused(), Some(42));
    }

    #[test]
    fn test_interaction_state() {
        let mut state = InteractionState::new();
        
        assert!(!state.is_active());
        
        state.pressed = true;
        assert!(state.is_active());
        
        state.reset();
        assert!(!state.is_active());
    }
}
