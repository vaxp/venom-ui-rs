//! Gesture detection and input handling
//!
//! This module provides gesture detection widgets that can respond to
//! user input like taps, drags, and other pointer events.
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{GestureDetector, Container, Color};
//!
//! let interactive = GestureDetector::new()
//!     .on_tap(|| println!("Tapped!"))
//!     .on_tap_down(|pos| println!("Tap down at {:?}", pos))
//!     .child(
//!         Container::new()
//!             .width(100.0)
//!             .height(50.0)
//!             .color(Color::BLUE)
//!     );
//! ```

use std::any::Any;
use std::sync::Arc;
use venom_core::{BoxConstraints, Size, Offset, Point};
use venom_render::PaintCanvas;
use crate::{Widget, BoxedWidget};

// ============================================================================
// POINTER BUTTON
// ============================================================================

/// Mouse/pointer button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PointerButton {
    /// Left mouse button or primary touch
    #[default]
    Primary,
    /// Right mouse button
    Secondary,
    /// Middle mouse button
    Middle,
}

// ============================================================================
// POINTER EVENT
// ============================================================================

/// Information about a pointer event (mouse/touch)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    /// Position relative to widget
    pub local_position: Point,
    /// Position relative to window/screen
    pub global_position: Point,
    /// Button that triggered the event
    pub button: PointerButton,
    /// Pointer ID (for multi-touch)
    pub pointer_id: u32,
    /// Time of event (in milliseconds)
    pub timestamp: u64,
    /// Change in position (for drag)
    pub delta: Offset,
}

impl PointerEvent {
    /// Create a new pointer event
    pub fn new(local: Point, global: Point) -> Self {
        Self {
            local_position: local,
            global_position: global,
            button: PointerButton::Primary,
            pointer_id: 0,
            timestamp: 0,
            delta: Offset::ZERO,
        }
    }

    /// Set the button
    pub fn with_button(mut self, button: PointerButton) -> Self {
        self.button = button;
        self
    }

    /// Set pointer ID
    pub fn with_pointer_id(mut self, id: u32) -> Self {
        self.pointer_id = id;
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set delta
    pub fn with_delta(mut self, delta: Offset) -> Self {
        self.delta = delta;
        self
    }
}

// ============================================================================
// TAP DETAILS
// ============================================================================

/// Details about a tap gesture
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TapDetails {
    /// Position of tap relative to widget
    pub local_position: Point,
    /// Position of tap relative to window
    pub global_position: Point,
    /// Which tap in a sequence (1 for single tap, 2 for double tap)
    pub tap_count: u32,
}

// ============================================================================
// DRAG DETAILS
// ============================================================================

/// Details about a drag gesture
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragDetails {
    /// Position relative to widget
    pub local_position: Point,
    /// Position relative to window
    pub global_position: Point,
    /// Change since last event
    pub delta: Offset,
    /// Velocity of drag (pixels per second)
    pub velocity: Offset,
}

impl DragDetails {
    /// Create from current and previous positions
    pub fn new(local: Point, global: Point, delta: Offset) -> Self {
        Self {
            local_position: local,
            global_position: global,
            delta,
            velocity: Offset::ZERO,
        }
    }

    /// Set velocity
    pub fn with_velocity(mut self, velocity: Offset) -> Self {
        self.velocity = velocity;
        self
    }
}

// ============================================================================
// CALLBACK TYPES
// ============================================================================

/// Callback for simple tap
pub type TapCallback = Arc<dyn Fn() + Send + Sync>;

/// Callback for tap with position
pub type TapDownCallback = Arc<dyn Fn(TapDetails) + Send + Sync>;

/// Callback for drag events
pub type DragCallback = Arc<dyn Fn(DragDetails) + Send + Sync>;

/// Callback for hover events
pub type HoverCallback = Arc<dyn Fn(Point) + Send + Sync>;

// ============================================================================
// GESTURE DETECTOR
// ============================================================================

/// Gesture detection widget
/// 
/// Wraps a child widget and detects various gestures:
/// - Tap (with tap down, tap up, tap cancel)
/// - Double tap
/// - Long press
/// - Drag (pan)
/// - Hover
/// 
/// # Example
/// 
/// ```ignore
/// let detector = GestureDetector::new()
///     .on_tap(|| {
///         println!("Widget tapped!");
///     })
///     .on_double_tap(|| {
///         println!("Double tapped!");
///     })
///     .on_pan_update(|details| {
///         println!("Dragging: {:?}", details.delta);
///     })
///     .child(my_widget);
/// ```
pub struct GestureDetector {
    /// Child widget
    child: Option<BoxedWidget>,
    
    /// Tap callback (simple)
    on_tap: Option<TapCallback>,
    
    /// Tap down callback
    on_tap_down: Option<TapDownCallback>,
    
    /// Tap up callback
    on_tap_up: Option<TapDownCallback>,
    
    /// Tap cancel callback
    on_tap_cancel: Option<TapCallback>,
    
    /// Double tap callback
    on_double_tap: Option<TapCallback>,
    
    /// Long press callback
    on_long_press: Option<TapCallback>,
    
    /// Long press start callback
    on_long_press_start: Option<TapDownCallback>,
    
    /// Drag/pan start callback
    on_pan_start: Option<DragCallback>,
    
    /// Drag/pan update callback
    on_pan_update: Option<DragCallback>,
    
    /// Drag/pan end callback
    on_pan_end: Option<DragCallback>,
    
    /// Hover enter callback
    on_hover_enter: Option<HoverCallback>,
    
    /// Hover exit callback
    on_hover_exit: Option<TapCallback>,
    
    /// Hover move callback
    on_hover: Option<HoverCallback>,
    
    /// Hit test behavior
    behavior: HitTestBehavior,
}

/// How to handle hit testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HitTestBehavior {
    /// Only children are hit testable
    DeferToChild,
    /// Both this widget and children are hit testable
    #[default]
    Opaque,
    /// This widget receives events but they pass through
    Translucent,
}

impl GestureDetector {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a new gesture detector
    pub fn new() -> Self {
        Self {
            child: None,
            on_tap: None,
            on_tap_down: None,
            on_tap_up: None,
            on_tap_cancel: None,
            on_double_tap: None,
            on_long_press: None,
            on_long_press_start: None,
            on_pan_start: None,
            on_pan_update: None,
            on_pan_end: None,
            on_hover_enter: None,
            on_hover_exit: None,
            on_hover: None,
            behavior: HitTestBehavior::Opaque,
        }
    }

    // ========================================================================
    // BUILDER - CHILD
    // ========================================================================

    /// Set the child widget
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.child = Some(Box::new(widget));
        self
    }

    /// Set boxed child widget
    pub fn child_boxed(mut self, widget: BoxedWidget) -> Self {
        self.child = Some(widget);
        self
    }

    // ========================================================================
    // BUILDER - TAP CALLBACKS
    // ========================================================================

    /// Set tap callback (called on tap up after tap down)
    pub fn on_tap<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap = Some(Arc::new(callback));
        self
    }

    /// Set tap down callback
    pub fn on_tap_down<F: Fn(TapDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap_down = Some(Arc::new(callback));
        self
    }

    /// Set tap up callback
    pub fn on_tap_up<F: Fn(TapDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap_up = Some(Arc::new(callback));
        self
    }

    /// Set tap cancel callback
    pub fn on_tap_cancel<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap_cancel = Some(Arc::new(callback));
        self
    }

    /// Set double tap callback
    pub fn on_double_tap<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_double_tap = Some(Arc::new(callback));
        self
    }

    // ========================================================================
    // BUILDER - LONG PRESS CALLBACKS
    // ========================================================================

    /// Set long press callback
    pub fn on_long_press<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_long_press = Some(Arc::new(callback));
        self
    }

    /// Set long press start callback
    pub fn on_long_press_start<F: Fn(TapDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_long_press_start = Some(Arc::new(callback));
        self
    }

    // ========================================================================
    // BUILDER - DRAG/PAN CALLBACKS
    // ========================================================================

    /// Set pan/drag start callback
    pub fn on_pan_start<F: Fn(DragDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_pan_start = Some(Arc::new(callback));
        self
    }

    /// Set pan/drag update callback
    pub fn on_pan_update<F: Fn(DragDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_pan_update = Some(Arc::new(callback));
        self
    }

    /// Set pan/drag end callback
    pub fn on_pan_end<F: Fn(DragDetails) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_pan_end = Some(Arc::new(callback));
        self
    }

    // ========================================================================
    // BUILDER - HOVER CALLBACKS
    // ========================================================================

    /// Set hover enter callback
    pub fn on_hover_enter<F: Fn(Point) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_hover_enter = Some(Arc::new(callback));
        self
    }

    /// Set hover exit callback
    pub fn on_hover_exit<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_hover_exit = Some(Arc::new(callback));
        self
    }

    /// Set hover move callback
    pub fn on_hover<F: Fn(Point) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_hover = Some(Arc::new(callback));
        self
    }

    // ========================================================================
    // BUILDER - BEHAVIOR
    // ========================================================================

    /// Set hit test behavior
    pub fn behavior(mut self, behavior: HitTestBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Check if detector has any tap callbacks
    pub fn has_tap_callbacks(&self) -> bool {
        self.on_tap.is_some() 
            || self.on_tap_down.is_some() 
            || self.on_tap_up.is_some()
    }

    /// Check if detector has any drag callbacks
    pub fn has_drag_callbacks(&self) -> bool {
        self.on_pan_start.is_some() 
            || self.on_pan_update.is_some() 
            || self.on_pan_end.is_some()
    }

    /// Check if detector has any hover callbacks
    pub fn has_hover_callbacks(&self) -> bool {
        self.on_hover_enter.is_some() 
            || self.on_hover_exit.is_some() 
            || self.on_hover.is_some()
    }

    // ========================================================================
    // EVENT HANDLING (would be called by framework)
    // ========================================================================

    /// Handle pointer down event
    pub fn handle_pointer_down(&self, event: &PointerEvent) {
        if let Some(ref callback) = self.on_tap_down {
            callback(TapDetails {
                local_position: event.local_position,
                global_position: event.global_position,
                tap_count: 1,
            });
        }
    }

    /// Handle pointer up event
    pub fn handle_pointer_up(&self, event: &PointerEvent) {
        if let Some(ref callback) = self.on_tap_up {
            callback(TapDetails {
                local_position: event.local_position,
                global_position: event.global_position,
                tap_count: 1,
            });
        }
        
        // Simple tap (up after down without drag)
        if let Some(ref callback) = self.on_tap {
            callback();
        }
    }

    /// Handle pointer move event (during drag)
    pub fn handle_pointer_move(&self, event: &PointerEvent) {
        if let Some(ref callback) = self.on_pan_update {
            callback(DragDetails::new(
                event.local_position,
                event.global_position,
                event.delta,
            ));
        }
    }
}

impl Default for GestureDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for GestureDetector {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        if let Some(child) = &self.child {
            child.layout(constraints)
        } else {
            Size::ZERO
        }
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
// INKWELL (Material-style ripple effect)
// ============================================================================

/// InkWell - Material-style button with ripple effect
/// 
/// Similar to GestureDetector but provides visual feedback (ink splash).
/// 
/// # Example
/// 
/// ```ignore
/// let inkwell = InkWell::new()
///     .on_tap(|| println!("Pressed!"))
///     .splash_color(Color::rgba(255, 255, 255, 50))
///     .child(my_content);
/// ```
pub struct InkWell {
    /// Inner gesture detector
    gesture: GestureDetector,
    
    /// Splash/ripple color
    pub splash_color: venom_core::Color,
    
    /// Highlight color (on hover)
    pub highlight_color: venom_core::Color,
    
    /// Border radius for ripple
    pub border_radius: f32,
}

impl InkWell {
    /// Create a new InkWell
    pub fn new() -> Self {
        Self {
            gesture: GestureDetector::new(),
            splash_color: venom_core::Color::rgba(255, 255, 255, 30),
            highlight_color: venom_core::Color::rgba(255, 255, 255, 20),
            border_radius: 0.0,
        }
    }

    /// Set child widget
    pub fn child<W: Widget>(mut self, widget: W) -> Self {
        self.gesture = self.gesture.child(widget);
        self
    }

    /// Set tap callback
    pub fn on_tap<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.gesture = self.gesture.on_tap(callback);
        self
    }

    /// Set splash color
    pub fn splash_color(mut self, color: venom_core::Color) -> Self {
        self.splash_color = color;
        self
    }

    /// Set highlight color
    pub fn highlight_color(mut self, color: venom_core::Color) -> Self {
        self.highlight_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
}

impl Default for InkWell {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for InkWell {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.gesture.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Paint child through gesture detector
        self.gesture.paint(canvas, offset);
        
        // Note: Actual ripple animation would be handled by animation system
        // This would overlay a ripple effect during interaction
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
    fn test_pointer_event_creation() {
        let event = PointerEvent::new(
            Point::new(10.0, 20.0),
            Point::new(110.0, 220.0),
        );
        
        assert_eq!(event.local_position, Point::new(10.0, 20.0));
        assert_eq!(event.global_position, Point::new(110.0, 220.0));
        assert_eq!(event.button, PointerButton::Primary);
    }

    #[test]
    fn test_gesture_detector_creation() {
        let detector = GestureDetector::new();
        assert!(!detector.has_tap_callbacks());
        assert!(!detector.has_drag_callbacks());
    }

    #[test]
    fn test_gesture_detector_with_tap() {
        let detector = GestureDetector::new()
            .on_tap(|| println!("tap"));
        
        assert!(detector.has_tap_callbacks());
    }

    #[test]
    fn test_gesture_detector_with_drag() {
        let detector = GestureDetector::new()
            .on_pan_update(|_| {});
        
        assert!(detector.has_drag_callbacks());
    }

    #[test]
    fn test_drag_details() {
        let details = DragDetails::new(
            Point::new(50.0, 50.0),
            Point::new(150.0, 150.0),
            Offset::new(5.0, 5.0),
        );
        
        assert_eq!(details.delta.dx, 5.0);
        assert_eq!(details.delta.dy, 5.0);
    }

    #[test]
    fn test_inkwell_creation() {
        let inkwell = InkWell::new()
            .border_radius(8.0);
        
        assert_eq!(inkwell.border_radius, 8.0);
    }
}
