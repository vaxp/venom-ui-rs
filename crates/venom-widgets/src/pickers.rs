//! Picker Widgets - DropdownButton, SearchField
//!
//! Input widgets for selecting values from lists.

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, BorderRadius, Point};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// DROPDOWN BUTTON
// ============================================================================

/// Dropdown selection button
///
/// # Example
///
/// ```ignore
/// use venom_widgets::DropdownButton;
///
/// let dropdown = DropdownButton::new(options, selected_index)
///     .on_change(|index| handle_selection(index));
/// ```
pub struct DropdownButton {
    /// Option labels
    options: Vec<String>,
    /// Selected index
    selected_index: Option<usize>,
    /// Placeholder text
    placeholder: String,
    /// Is dropdown open
    is_open: bool,
    /// Background color
    background: Color,
    /// Text color
    text_color: Color,
    /// Border color
    border_color: Color,
    /// Border radius
    border_radius: f32,
    /// Height
    height: f32,
    /// Width
    width: Option<f32>,
    /// On change callback
    on_change: Option<Box<dyn Fn(usize) + Send + Sync>>,
    /// Enabled state
    enabled: bool,
}

impl Default for DropdownButton {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            selected_index: None,
            placeholder: "Select...".to_string(),
            is_open: false,
            background: Color::WHITE,
            text_color: Color::rgb(0, 0, 0),
            border_color: Color::rgba(0, 0, 0, 77),
            border_radius: 4.0,
            height: 48.0,
            width: None,
            on_change: None,
            enabled: true,
        }
    }
}

impl DropdownButton {
    /// Create a new dropdown
    pub fn new(options: Vec<String>, selected: Option<usize>) -> Self {
        Self {
            options,
            selected_index: selected,
            ..Default::default()
        }
    }

    /// Create from string slices
    pub fn from_strs(options: &[&str], selected: Option<usize>) -> Self {
        Self {
            options: options.iter().map(|s| s.to_string()).collect(),
            selected_index: selected,
            ..Default::default()
        }
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set selected index
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set change callback
    pub fn on_change<F: Fn(usize) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get selected value
    pub fn selected_value(&self) -> Option<&str> {
        self.selected_index.and_then(|i| self.options.get(i).map(|s| s.as_str()))
    }

    /// Display text
    fn display_text(&self) -> &str {
        self.selected_value().unwrap_or(&self.placeholder)
    }
}

impl Widget for DropdownButton {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let width = self.width.unwrap_or(200.0).min(constraints.max_width);
        Size::new(width, self.height.min(constraints.max_height))
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Background
        canvas.draw_rounded_rect(
            rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(self.background),
        );

        // Border
        canvas.draw_rounded_rect(
            rect,
            BorderRadius::all(self.border_radius),
            &Paint::stroke(self.border_color, 1.0),
        );

        // Text
        let text_color = if self.enabled {
            if self.selected_index.is_some() {
                self.text_color
            } else {
                Color::rgba(self.text_color.r, self.text_color.g, self.text_color.b, 128)
            }
        } else {
            Color::rgba(self.text_color.r, self.text_color.g, self.text_color.b, 77)
        };

        canvas.draw_text(
            self.display_text(),
            Point::new(offset.dx + 12.0, offset.dy + size.height / 2.0 - 8.0),
            &Paint::fill(text_color),
            14.0,
        );

        // Dropdown arrow
        let arrow_x = offset.dx + size.width - 24.0;
        let arrow_y = offset.dy + size.height / 2.0;
        canvas.draw_text(
            "▼",
            Point::new(arrow_x, arrow_y - 6.0),
            &Paint::fill(self.text_color),
            12.0,
        );

        // Dropdown menu (if open)
        if self.is_open {
            let menu_y = offset.dy + size.height + 2.0;
            let menu_height = self.options.len() as f32 * 40.0;
            let menu_rect = Rect::new(offset.dx, menu_y, size.width, menu_height);
            
            // Shadow
            canvas.draw_rounded_rect(
                Rect::new(offset.dx + 2.0, menu_y + 2.0, size.width, menu_height),
                BorderRadius::all(self.border_radius),
                &Paint::fill(Color::rgba(0, 0, 0, 30)),
            );

            // Menu background
            canvas.draw_rounded_rect(
                menu_rect,
                BorderRadius::all(self.border_radius),
                &Paint::fill(self.background),
            );

            // Options
            for (i, option) in self.options.iter().enumerate() {
                let option_y = menu_y + i as f32 * 40.0;
                let is_selected = Some(i) == self.selected_index;

                if is_selected {
                    let highlight_rect = Rect::new(
                        offset.dx + 2.0,
                        option_y + 2.0,
                        size.width - 4.0,
                        36.0,
                    );
                    canvas.draw_rect(highlight_rect, &Paint::fill(Color::rgba(33, 150, 243, 30)));
                }

                canvas.draw_text(
                    option,
                    Point::new(offset.dx + 12.0, option_y + 20.0 - 8.0),
                    &Paint::fill(self.text_color),
                    14.0,
                );
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// SEARCH FIELD
// ============================================================================

/// Search input field
pub struct SearchField {
    /// Current value
    value: String,
    /// Placeholder
    placeholder: String,
    /// Background color
    background: Color,
    /// Text color
    text_color: Color,
    /// Border radius
    border_radius: f32,
    /// Height
    height: f32,
    /// On change callback
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// On submit callback
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    /// Show clear button
    show_clear: bool,
    /// Show search icon
    show_icon: bool,
}

impl Default for SearchField {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: "Search...".to_string(),
            background: Color::rgba(0, 0, 0, 13),
            text_color: Color::rgb(0, 0, 0),
            border_radius: 24.0,
            height: 48.0,
            on_change: None,
            on_submit: None,
            show_clear: true,
            show_icon: true,
        }
    }
}

impl SearchField {
    /// Create a new search field
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set placeholder
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set change callback
    pub fn on_change<F: Fn(&str) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set submit callback
    pub fn on_submit<F: Fn(&str) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_submit = Some(Box::new(callback));
        self
    }

    /// Show/hide clear button
    pub fn show_clear(mut self, show: bool) -> Self {
        self.show_clear = show;
        self
    }

    /// Show/hide search icon
    pub fn show_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }
}

impl Widget for SearchField {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            self.height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(300.0));
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Background
        canvas.draw_rounded_rect(
            rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(self.background),
        );

        let mut text_x = offset.dx + 16.0;

        // Search icon
        if self.show_icon {
            canvas.draw_text(
                "🔍",
                Point::new(text_x, offset.dy + size.height / 2.0 - 8.0),
                &Paint::fill(Color::rgb(128, 128, 128)),
                16.0,
            );
            text_x += 28.0;
        }

        // Text or placeholder
        let display_text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };
        let text_color = if self.value.is_empty() {
            Color::rgba(self.text_color.r, self.text_color.g, self.text_color.b, 128)
        } else {
            self.text_color
        };

        canvas.draw_text(
            display_text,
            Point::new(text_x, offset.dy + size.height / 2.0 - 8.0),
            &Paint::fill(text_color),
            16.0,
        );

        // Clear button
        if self.show_clear && !self.value.is_empty() {
            let clear_x = offset.dx + size.width - 40.0;
            canvas.draw_text(
                "✕",
                Point::new(clear_x, offset.dy + size.height / 2.0 - 8.0),
                &Paint::fill(Color::rgb(128, 128, 128)),
                16.0,
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
    fn test_dropdown() {
        let dropdown = DropdownButton::from_strs(
            &["Option 1", "Option 2", "Option 3"],
            Some(1),
        );
        assert_eq!(dropdown.selected_value(), Some("Option 2"));
    }

    #[test]
    fn test_dropdown_placeholder() {
        let dropdown = DropdownButton::new(vec![], None)
            .placeholder("Choose one");
        assert_eq!(dropdown.display_text(), "Choose one");
    }

    #[test]
    fn test_search_field() {
        let field = SearchField::new()
            .value("test query")
            .placeholder("Search...");
        assert_eq!(field.value, "test query");
    }
}
