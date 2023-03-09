//! Event types for user interaction
//!
//! This module provides simple event types for handling user input.
//! The complexity is hidden - users just handle callbacks.

use venom_core::Point;

// ============================================================================
// MOUSE EVENTS
// ============================================================================

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Right mouse button  
    Right,
    /// Middle mouse button
    Middle,
}

/// Mouse event types
#[derive(Debug, Clone, Copy)]
pub enum MouseEvent {
    /// Mouse button pressed
    ButtonPressed(MouseButton),
    /// Mouse button released
    ButtonReleased(MouseButton),
    /// Mouse cursor moved
    CursorMoved { position: Point },
    /// Mouse cursor entered widget
    CursorEntered,
    /// Mouse cursor left widget
    CursorLeft,
    /// Mouse wheel scrolled
    WheelScrolled { delta_x: f32, delta_y: f32 },
}

// ============================================================================
// KEYBOARD EVENTS
// ============================================================================

/// Keyboard modifier keys
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift key pressed
    pub shift: bool,
    /// Control key pressed
    pub ctrl: bool,
    /// Alt key pressed
    pub alt: bool,
    /// Meta/Super/Windows key pressed
    pub meta: bool,
}

impl Modifiers {
    /// No modifiers
    pub const NONE: Self = Self { shift: false, ctrl: false, alt: false, meta: false };
    
    /// Check if any modifier is pressed
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.meta
    }
}

/// Common key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Special keys
    Enter,
    Tab,
    Space,
    Backspace,
    Delete,
    Escape,
    
    // Arrow keys
    Left,
    Right,
    Up,
    Down,
    
    // Modifier keys (when pressed alone)
    Shift,
    Control,
    Alt,
    
    // Other
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    
    /// Unknown key
    Unknown,
}

/// Keyboard event
#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    /// Key pressed
    KeyPressed {
        key: Key,
        modifiers: Modifiers,
    },
    /// Key released
    KeyReleased {
        key: Key,
        modifiers: Modifiers,
    },
    /// Character typed (for text input)
    CharacterReceived(char),
}

// ============================================================================
// TOUCH EVENTS
// ============================================================================

/// Touch event
#[derive(Debug, Clone, Copy)]
pub enum TouchEvent {
    /// Finger touched screen
    Started { id: u64, position: Point },
    /// Finger moved
    Moved { id: u64, position: Point },
    /// Finger lifted
    Ended { id: u64 },
    /// Touch cancelled
    Cancelled { id: u64 },
}

// ============================================================================
// UNIFIED EVENT
// ============================================================================

/// All possible events
#[derive(Debug, Clone)]
pub enum Event {
    /// Mouse event
    Mouse(MouseEvent),
    /// Keyboard event
    Keyboard(KeyboardEvent),
    /// Touch event
    Touch(TouchEvent),
}

impl Event {
    /// Check if this is a press event (mouse or touch)
    pub fn is_press(&self) -> bool {
        matches!(
            self,
            Event::Mouse(MouseEvent::ButtonPressed(_)) | Event::Touch(TouchEvent::Started { .. })
        )
    }
    
    /// Check if this is a release event
    pub fn is_release(&self) -> bool {
        matches!(
            self,
            Event::Mouse(MouseEvent::ButtonReleased(_)) | Event::Touch(TouchEvent::Ended { .. })
        )
    }
}

// ============================================================================
// CONVENIENT SHORTCUTS
// ============================================================================

impl From<MouseEvent> for Event {
    fn from(e: MouseEvent) -> Self {
        Event::Mouse(e)
    }
}

impl From<KeyboardEvent> for Event {
    fn from(e: KeyboardEvent) -> Self {
        Event::Keyboard(e)
    }
}

impl From<TouchEvent> for Event {
    fn from(e: TouchEvent) -> Self {
        Event::Touch(e)
    }
}
