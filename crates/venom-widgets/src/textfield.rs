//! TextField widget - text input field
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::TextField;
//!
//! let input = TextField::new(&text)
//!     .placeholder("Enter your name")
//!     .on_change(|new_text| set_text(new_text));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Point, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// TEXT FIELD
// ============================================================================

/// A single-line text input field
/// 
/// # Example
/// 
/// ```ignore
/// // Simple input
/// let name = TextField::new(&name_value)
///     .placeholder("Enter name");
/// 
/// // With callbacks
/// let email = TextField::new(&email_value)
///     .placeholder("Email address")
///     .on_change(|text| set_email(text))
///     .on_submit(|| submit_form());
/// 
/// // Password field
/// let password = TextField::new(&password_value)
///     .placeholder("Password")
///     .obscure_text(true);
/// 
/// // Styled input
/// let search = TextField::new(&query)
///     .placeholder("Search...")
///     .prefix_icon("🔍")
///     .border_radius(20.0);
/// ```
pub struct TextField {
    /// Current text value
    value: String,
    /// Placeholder text
    placeholder: String,
    /// Cursor position
    cursor: usize,
    /// Selection start (if any)
    selection_start: Option<usize>,
    /// Whether text is obscured (password mode)
    obscure_text: bool,
    /// Maximum length (0 = unlimited)
    max_length: usize,
    /// Read-only mode
    read_only: bool,
    /// Disabled state
    disabled: bool,
    /// Is focused
    focused: bool,

    // Appearance
    /// Width
    width: f32,
    /// Height
    height: f32,
    /// Font size
    font_size: f32,
    /// Text color
    text_color: Color,
    /// Placeholder color
    placeholder_color: Color,
    /// Background color
    background: Color,
    /// Border color
    border_color: Color,
    /// Focused border color
    focus_border_color: Color,
    /// Border radius
    border_radius: f32,
    /// Padding
    padding: f32,

    // Icons
    /// Prefix icon
    prefix_icon: Option<String>,
    /// Suffix icon
    suffix_icon: Option<String>,

    // Callbacks
    /// Change callback
    on_change: Option<Box<dyn Fn(String) + Send + Sync>>,
    /// Submit callback (Enter key)
    on_submit: Option<Box<dyn Fn() + Send + Sync>>,
    /// Focus callback
    on_focus: Option<Box<dyn Fn(bool) + Send + Sync>>,
}

impl Default for TextField {
    fn default() -> Self {
        Self::new("")
    }
}

impl TextField {
    /// Create a new text field with the given value
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
            placeholder: String::new(),
            cursor: value.len(),
            selection_start: None,
            obscure_text: false,
            max_length: 0,
            read_only: false,
            disabled: false,
            focused: false,

            width: 200.0,
            height: 40.0,
            font_size: 14.0,
            text_color: Color::WHITE,
            placeholder_color: Color::hex("#6b7280"),
            background: Color::hex("#1f2937"),
            border_color: Color::hex("#374151"),
            focus_border_color: Color::hex("#6366f1"),
            border_radius: 8.0,
            padding: 12.0,

            prefix_icon: None,
            suffix_icon: None,

            on_change: None,
            on_submit: None,
            on_focus: None,
        }
    }

    // ========================================================================
    // BUILDER
    // ========================================================================

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Enable password mode
    pub fn obscure_text(mut self, obscure: bool) -> Self {
        self.obscure_text = obscure;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = max;
        self
    }

    /// Set read-only mode
    pub fn read_only(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width.max(50.0);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(20.0);
        self
    }

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size.max(8.0);
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set focus border color
    pub fn focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set prefix icon
    pub fn prefix_icon(mut self, icon: impl Into<String>) -> Self {
        self.prefix_icon = Some(icon.into());
        self
    }

    /// Set suffix icon
    pub fn suffix_icon(mut self, icon: impl Into<String>) -> Self {
        self.suffix_icon = Some(icon.into());
        self
    }

    /// Set change callback
    pub fn on_change<F: Fn(String) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set submit callback
    pub fn on_submit<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_submit = Some(Box::new(callback));
        self
    }

    /// Set focus callback
    pub fn on_focus<F: Fn(bool) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_focus = Some(Box::new(callback));
        self
    }

    // ========================================================================
    // TEXT MANIPULATION
    // ========================================================================

    /// Get current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value
    pub fn set_value(&mut self, value: impl Into<String>) {
        let new_value = value.into();
        let limited = if self.max_length > 0 && new_value.len() > self.max_length {
            new_value[..self.max_length].to_string()
        } else {
            new_value
        };
        
        self.value = limited;
        self.cursor = self.cursor.min(self.value.len());
        
        if let Some(callback) = &self.on_change {
            callback(self.value.clone());
        }
    }

    /// Insert text at cursor
    pub fn insert(&mut self, text: &str) {
        if self.read_only || self.disabled {
            return;
        }

        // Delete selection first
        self.delete_selection();

        // Check max length
        if self.max_length > 0 && self.value.len() + text.len() > self.max_length {
            let available = self.max_length.saturating_sub(self.value.len());
            if available == 0 {
                return;
            }
            let text = &text[..available.min(text.len())];
            self.value.insert_str(self.cursor, text);
            self.cursor += text.len();
        } else {
            self.value.insert_str(self.cursor, text);
            self.cursor += text.len();
        }

        if let Some(callback) = &self.on_change {
            callback(self.value.clone());
        }
    }

    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.read_only || self.disabled {
            return;
        }

        if self.selection_start.is_some() {
            self.delete_selection();
        } else if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
            
            if let Some(callback) = &self.on_change {
                callback(self.value.clone());
            }
        }
    }

    /// Delete character at cursor
    pub fn delete(&mut self) {
        if self.read_only || self.disabled {
            return;
        }

        if self.selection_start.is_some() {
            self.delete_selection();
        } else if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
            
            if let Some(callback) = &self.on_change {
                callback(self.value.clone());
            }
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self) {
        if let Some(start) = self.selection_start {
            let (from, to) = if start < self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            };
            
            self.value.drain(from..to);
            self.cursor = from;
            self.selection_start = None;
            
            if let Some(callback) = &self.on_change {
                callback(self.value.clone());
            }
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        self.selection_start = None;
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
        self.selection_start = None;
    }

    /// Move cursor to start
    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
        self.selection_start = None;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor = self.value.len();
        self.selection_start = None;
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor = self.value.len();
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if let Some(callback) = &self.on_focus {
            callback(focused);
        }
    }

    /// Get display text (obscured if password)
    fn display_text(&self) -> String {
        if self.obscure_text {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }
}

impl Widget for TextField {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            if constraints.max_width < self.width { constraints.max_width } else { self.width },
            if constraints.max_height < self.height { constraints.max_height } else { self.height },
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let rect = Rect::new(offset.dx, offset.dy, self.width, self.height);
        let radius = BorderRadius::all(self.border_radius);

        // Background
        let bg_color = if self.disabled {
            Color::hex("#111827")
        } else {
            self.background
        };
        canvas.draw_rounded_rect(rect, radius, &Paint::fill(bg_color));

        // Border
        let border_color = if self.disabled {
            Color::hex("#1f2937")
        } else if self.focused {
            self.focus_border_color
        } else {
            self.border_color
        };
        canvas.draw_rounded_rect(rect, radius, &Paint::stroke(border_color, 1.5));

        // Calculate text area
        let text_x = offset.dx + self.padding;
        let text_y = offset.dy + (self.height - self.font_size) / 2.0;
        let text_width = self.width - self.padding * 2.0;

        // Draw prefix icon
        let mut text_offset_x = 0.0;
        if let Some(icon) = &self.prefix_icon {
            canvas.draw_text(
                icon,
                Point::new(text_x, text_y),
                &Paint::fill(self.placeholder_color),
                self.font_size,
            );
            text_offset_x += self.font_size + 8.0;
        }

        // Draw text or placeholder
        let display = self.display_text();
        if display.is_empty() {
            // Placeholder
            if !self.placeholder.is_empty() {
                canvas.draw_text(
                    &self.placeholder,
                    Point::new(text_x + text_offset_x, text_y),
                    &Paint::fill(self.placeholder_color),
                    self.font_size,
                );
            }
        } else {
            // Actual text
            let text_color = if self.disabled {
                Color::hex("#6b7280")
            } else {
                self.text_color
            };
            canvas.draw_text(
                &display,
                Point::new(text_x + text_offset_x, text_y),
                &Paint::fill(text_color),
                self.font_size,
            );
        }

        // Draw cursor if focused
        if self.focused && !self.disabled && !self.read_only {
            let cursor_x = text_x + text_offset_x + (self.cursor as f32 * self.font_size * 0.6);
            let cursor_y = offset.dy + 6.0;
            let cursor_height = self.height - 12.0;

            canvas.draw_rect(
                Rect::new(cursor_x, cursor_y, 2.0, cursor_height),
                &Paint::fill(self.focus_border_color),
            );
        }

        // Draw suffix icon
        if let Some(icon) = &self.suffix_icon {
            let icon_x = offset.dx + self.width - self.padding - self.font_size;
            canvas.draw_text(
                icon,
                Point::new(icon_x, text_y),
                &Paint::fill(self.placeholder_color),
                self.font_size,
            );
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
    fn test_textfield_creation() {
        let tf = TextField::new("hello");
        assert_eq!(tf.value(), "hello");
    }

    #[test]
    fn test_textfield_insert() {
        let mut tf = TextField::new("");
        tf.insert("hello");
        assert_eq!(tf.value(), "hello");
        
        tf.cursor = 5;
        tf.insert(" world");
        assert_eq!(tf.value(), "hello world");
    }

    #[test]
    fn test_textfield_backspace() {
        let mut tf = TextField::new("hello");
        tf.cursor = 5;
        tf.backspace();
        assert_eq!(tf.value(), "hell");
    }

    #[test]
    fn test_textfield_max_length() {
        let mut tf = TextField::new("").max_length(5);
        tf.insert("hello world");
        assert_eq!(tf.value().len(), 5);
    }

    #[test]
    fn test_textfield_obscure() {
        let tf = TextField::new("password").obscure_text(true);
        assert_eq!(tf.display_text(), "••••••••");
    }

    #[test]
    fn test_textfield_cursor_movement() {
        let mut tf = TextField::new("hello");
        tf.cursor = 3;
        
        tf.move_cursor_left();
        assert_eq!(tf.cursor, 2);
        
        tf.move_cursor_right();
        assert_eq!(tf.cursor, 3);
        
        tf.move_cursor_home();
        assert_eq!(tf.cursor, 0);
        
        tf.move_cursor_end();
        assert_eq!(tf.cursor, 5);
    }
}
