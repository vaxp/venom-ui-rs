//! Shell - Connection between widgets and application state
//!
//! Inspired by Iced's Shell pattern.
//!
//! The Shell provides a way for widgets to:
//! - Publish messages to the application
//! - Capture events to prevent bubbling
//! - Request redraws
//! - Invalidate layout

use crate::event::Event;

// ============================================================================
// EVENT STATUS
// ============================================================================

/// The status of an event after being processed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EventStatus {
    /// The event was NOT handled by any widget
    #[default]
    Ignored,
    /// The event was handled and processed by a widget
    Captured,
}

impl EventStatus {
    /// Merge two statuses - Captured takes precedence
    pub fn merge(self, other: Self) -> Self {
        match self {
            EventStatus::Ignored => other,
            EventStatus::Captured => EventStatus::Captured,
        }
    }
    
    /// Check if event was captured
    pub fn is_captured(&self) -> bool {
        matches!(self, EventStatus::Captured)
    }
}

// ============================================================================
// REDRAW REQUEST
// ============================================================================

/// When to redraw the window
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedrawRequest {
    /// Redraw on next frame
    NextFrame,
    /// Wait for events
    Wait,
    /// Redraw at specific time (ms from now)
    At(u64),
}

impl Default for RedrawRequest {
    fn default() -> Self {
        Self::Wait
    }
}

impl RedrawRequest {
    /// Get the minimum (soonest) redraw request
    pub fn min(self, other: Self) -> Self {
        match (self, other) {
            (Self::NextFrame, _) | (_, Self::NextFrame) => Self::NextFrame,
            (Self::At(a), Self::At(b)) => Self::At(a.min(b)),
            (Self::At(t), Self::Wait) | (Self::Wait, Self::At(t)) => Self::At(t),
            (Self::Wait, Self::Wait) => Self::Wait,
        }
    }
}

// ============================================================================
// SHELL
// ============================================================================

/// A connection to the application state from widgets
/// 
/// Widgets can use the Shell to:
/// - Publish messages
/// - Capture events (prevent bubbling)
/// - Request redraws
/// - Invalidate layout
/// 
/// # Example
/// 
/// ```ignore
/// fn handle_event(&mut self, event: &Event, shell: &mut Shell<MyMessage>) {
///     if let Event::Mouse(MouseEvent::ButtonPressed(Left)) = event {
///         shell.publish(MyMessage::ButtonClicked);
///         shell.capture_event();
///     }
/// }
/// ```
pub struct Shell<'a, Message> {
    /// Messages buffer
    messages: &'a mut Vec<Message>,
    /// Current event status
    event_status: EventStatus,
    /// Redraw request
    redraw_request: RedrawRequest,
    /// Whether layout needs recalculation
    layout_invalid: bool,
    /// Whether widgets need rebuilding
    widgets_invalid: bool,
}

impl<'a, Message> Shell<'a, Message> {
    /// Create a new Shell with the provided message buffer
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            event_status: EventStatus::Ignored,
            redraw_request: RedrawRequest::Wait,
            layout_invalid: false,
            widgets_invalid: false,
        }
    }

    // ========================================================================
    // MESSAGES
    // ========================================================================

    /// Publish a message for the application to process
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Check if the shell has any messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Drain all messages
    pub fn drain_messages(&mut self) -> impl Iterator<Item = Message> + '_ {
        self.messages.drain(..)
    }

    // ========================================================================
    // EVENT STATUS
    // ========================================================================

    /// Mark the current event as captured (prevents bubbling)
    /// 
    /// Call this when a widget has fully handled an event and
    /// no parent widgets should process it.
    pub fn capture_event(&mut self) {
        self.event_status = EventStatus::Captured;
    }

    /// Get the current event status
    pub fn event_status(&self) -> EventStatus {
        self.event_status
    }

    /// Check if the event was captured
    pub fn is_event_captured(&self) -> bool {
        self.event_status.is_captured()
    }

    // ========================================================================
    // REDRAW
    // ========================================================================

    /// Request a redraw on the next frame
    pub fn request_redraw(&mut self) {
        self.redraw_request = RedrawRequest::NextFrame;
    }

    /// Request a redraw at a specific time
    pub fn request_redraw_at(&mut self, ms_from_now: u64) {
        self.redraw_request = self.redraw_request.min(RedrawRequest::At(ms_from_now));
    }

    /// Get the current redraw request
    pub fn redraw_request(&self) -> RedrawRequest {
        self.redraw_request
    }

    // ========================================================================
    // LAYOUT
    // ========================================================================

    /// Invalidate the layout (causes relayout)
    pub fn invalidate_layout(&mut self) {
        self.layout_invalid = true;
    }

    /// Check if layout is invalid
    pub fn is_layout_invalid(&self) -> bool {
        self.layout_invalid
    }

    /// Invalidate widgets (causes rebuild)
    pub fn invalidate_widgets(&mut self) {
        self.widgets_invalid = true;
    }

    /// Check if widgets are invalid
    pub fn are_widgets_invalid(&self) -> bool {
        self.widgets_invalid
    }

    // ========================================================================
    // MERGING
    // ========================================================================

    /// Merge with another shell (for composition)
    pub fn merge<B>(&mut self, other: Shell<'_, B>, map: impl Fn(B) -> Message) {
        self.messages.extend(other.messages.drain(..).map(map));
        self.event_status = self.event_status.merge(other.event_status);
        self.redraw_request = self.redraw_request.min(other.redraw_request);
        self.layout_invalid = self.layout_invalid || other.layout_invalid;
        self.widgets_invalid = self.widgets_invalid || other.widgets_invalid;
    }
}

// ============================================================================
// CURSOR STATE
// ============================================================================

/// Current cursor/mouse state
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    /// Current position (None if outside window)
    pub position: Option<(f32, f32)>,
}

impl Cursor {
    /// Create cursor at position
    pub fn at(x: f32, y: f32) -> Self {
        Self { position: Some((x, y)) }
    }

    /// Create unavailable cursor
    pub fn unavailable() -> Self {
        Self { position: None }
    }

    /// Check if cursor is available
    pub fn is_available(&self) -> bool {
        self.position.is_some()
    }

    /// Check if cursor is over a rectangle
    pub fn is_over(&self, x: f32, y: f32, width: f32, height: f32) -> bool {
        if let Some((cx, cy)) = self.position {
            cx >= x && cx <= x + width && cy >= y && cy <= y + height
        } else {
            false
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
    fn test_event_status_merge() {
        assert_eq!(EventStatus::Ignored.merge(EventStatus::Ignored), EventStatus::Ignored);
        assert_eq!(EventStatus::Ignored.merge(EventStatus::Captured), EventStatus::Captured);
        assert_eq!(EventStatus::Captured.merge(EventStatus::Ignored), EventStatus::Captured);
        assert_eq!(EventStatus::Captured.merge(EventStatus::Captured), EventStatus::Captured);
    }

    #[test]
    fn test_shell_publish() {
        let mut messages: Vec<String> = vec![];
        let mut shell = Shell::new(&mut messages);
        
        shell.publish("hello".to_string());
        shell.publish("world".to_string());
        
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn test_shell_capture() {
        let mut messages: Vec<()> = vec![];
        let mut shell = Shell::new(&mut messages);
        
        assert!(!shell.is_event_captured());
        
        shell.capture_event();
        
        assert!(shell.is_event_captured());
    }

    #[test]
    fn test_redraw_request_min() {
        assert_eq!(RedrawRequest::NextFrame.min(RedrawRequest::Wait), RedrawRequest::NextFrame);
        assert_eq!(RedrawRequest::At(100).min(RedrawRequest::At(50)), RedrawRequest::At(50));
    }

    #[test]
    fn test_cursor() {
        let cursor = Cursor::at(50.0, 50.0);
        
        assert!(cursor.is_over(0.0, 0.0, 100.0, 100.0));
        assert!(!cursor.is_over(100.0, 100.0, 50.0, 50.0));
    }
}
