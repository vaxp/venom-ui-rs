//! Button widget - Interactive clickable widget
//!
//! Button provides a clickable widget with various styles and states.
//! It supports hover, pressed, and disabled states with customizable appearance.
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Button, Text, Color};
//!
//! // Simple button
//! let btn = Button::new("Click Me")
//!     .on_press(|| println!("Button clicked!"));
//!
//! // Styled button
//! let styled = Button::new("Submit")
//!     .color(Color::hex("#6366f1"))
//!     .text_color(Color::WHITE)
//!     .border_radius(8.0)
//!     .padding_all(16.0);
//! ```

use std::any::Any;
use std::sync::Arc;
use venom_core::{BoxConstraints, Size, Offset, Color, Rect, Insets, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::{Widget, BoxedWidget};
use crate::text::{Text, TextStyle, FontWeight};

// ============================================================================
// BUTTON STYLE
// ============================================================================

/// Visual style variant for buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonStyle {
    /// Filled background (primary action)
    #[default]
    Filled,
    /// Outlined border only
    Outlined,
    /// Text only, no background or border
    Text,
    /// Elevated with shadow
    Elevated,
}

// ============================================================================
// BUTTON STATE
// ============================================================================

/// Current interaction state of a button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ButtonState {
    /// Normal idle state
    #[default]
    Normal,
    /// Mouse is hovering over button
    Hovered,
    /// Button is being pressed
    Pressed,
    /// Button is disabled
    Disabled,
    /// Button has focus
    Focused,
}

// ============================================================================
// BUTTON COLORS
// ============================================================================

/// Color scheme for button states
#[derive(Debug, Clone, PartialEq)]
pub struct ButtonColors {
    /// Background color in normal state
    pub background: Color,
    /// Background color when hovered
    pub background_hovered: Color,
    /// Background color when pressed
    pub background_pressed: Color,
    /// Background color when disabled
    pub background_disabled: Color,
    /// Text/icon color
    pub foreground: Color,
    /// Text/icon color when disabled
    pub foreground_disabled: Color,
    /// Border color (for outlined style)
    pub border: Color,
}

impl ButtonColors {
    /// Create a primary (blue) color scheme
    pub fn primary() -> Self {
        let primary = Color::hex("#6366f1");
        Self {
            background: primary,
            background_hovered: Color::hex("#818cf8"),
            background_pressed: Color::hex("#4f46e5"),
            background_disabled: Color::hex("#9ca3af"),
            foreground: Color::WHITE,
            foreground_disabled: Color::hex("#d1d5db"),
            border: primary,
        }
    }

    /// Create a secondary (gray) color scheme
    pub fn secondary() -> Self {
        Self {
            background: Color::hex("#374151"),
            background_hovered: Color::hex("#4b5563"),
            background_pressed: Color::hex("#1f2937"),
            background_disabled: Color::hex("#9ca3af"),
            foreground: Color::WHITE,
            foreground_disabled: Color::hex("#d1d5db"),
            border: Color::hex("#4b5563"),
        }
    }

    /// Create a danger (red) color scheme
    pub fn danger() -> Self {
        let danger = Color::hex("#ef4444");
        Self {
            background: danger,
            background_hovered: Color::hex("#f87171"),
            background_pressed: Color::hex("#dc2626"),
            background_disabled: Color::hex("#9ca3af"),
            foreground: Color::WHITE,
            foreground_disabled: Color::hex("#d1d5db"),
            border: danger,
        }
    }

    /// Create a success (green) color scheme
    pub fn success() -> Self {
        let success = Color::hex("#22c55e");
        Self {
            background: success,
            background_hovered: Color::hex("#4ade80"),
            background_pressed: Color::hex("#16a34a"),
            background_disabled: Color::hex("#9ca3af"),
            foreground: Color::WHITE,
            foreground_disabled: Color::hex("#d1d5db"),
            border: success,
        }
    }

    /// Get background color for current state
    pub fn background_for_state(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Normal => self.background,
            ButtonState::Hovered => self.background_hovered,
            ButtonState::Pressed => self.background_pressed,
            ButtonState::Disabled => self.background_disabled,
            ButtonState::Focused => self.background_hovered,
        }
    }

    /// Get foreground color for current state
    pub fn foreground_for_state(&self, state: ButtonState) -> Color {
        match state {
            ButtonState::Disabled => self.foreground_disabled,
            _ => self.foreground,
        }
    }
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self::primary()
    }
}

// ============================================================================
// BUTTON CALLBACK
// ============================================================================

/// Type alias for button press callback
pub type OnPressCallback = Arc<dyn Fn() + Send + Sync>;

// ============================================================================
// BUTTON WIDGET
// ============================================================================

/// Interactive button widget
/// 
/// # Features
/// 
/// - Multiple visual styles (filled, outlined, text)
/// - State-based appearance (normal, hovered, pressed, disabled)
/// - Customizable colors, padding, and border radius
/// - Icon support (leading/trailing)
/// - Callback on press
/// 
/// # Example
/// 
/// ```ignore
/// // Basic button
/// let btn = Button::new("Click Me");
/// 
/// // With callback
/// let btn = Button::new("Save")
///     .on_press(|| save_data());
/// 
/// // Custom styled
/// let btn = Button::new("Delete")
///     .style(ButtonStyle::Outlined)
///     .colors(ButtonColors::danger())
///     .border_radius(4.0);
/// ```
pub struct Button {
    /// Button label text
    label: String,
    
    /// Visual style
    button_style: ButtonStyle,
    
    /// Color scheme
    colors: ButtonColors,
    
    /// Current state (for rendering)
    state: ButtonState,
    
    /// Padding around content
    padding: Insets,
    
    /// Border radius
    border_radius: f32,
    
    /// Border width (for outlined style)
    border_width: f32,
    
    /// Minimum width
    min_width: Option<f32>,
    
    /// Minimum height
    min_height: Option<f32>,
    
    /// Font size for label
    font_size: f32,
    
    /// Font weight for label
    font_weight: FontWeight,
    
    /// Is the button disabled?
    disabled: bool,
    
    /// Optional leading icon (widget)
    leading: Option<BoxedWidget>,
    
    /// Optional trailing icon (widget)
    trailing: Option<BoxedWidget>,
    
    /// Spacing between icon and text
    icon_spacing: f32,
    
    /// Press callback (stored for reference, not called during paint)
    on_press: Option<OnPressCallback>,
}

impl Button {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a new button with label text
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            button_style: ButtonStyle::Filled,
            colors: ButtonColors::primary(),
            state: ButtonState::Normal,
            padding: Insets::symmetric(16.0, 12.0),
            border_radius: 6.0,
            border_width: 1.5,
            min_width: Some(64.0),
            min_height: Some(36.0),
            font_size: 14.0,
            font_weight: FontWeight::Medium,
            disabled: false,
            leading: None,
            trailing: None,
            icon_spacing: 8.0,
            on_press: None,
        }
    }

    /// Create an icon-only button
    pub fn icon<W: Widget>(icon: W) -> Self {
        Self {
            label: String::new(),
            button_style: ButtonStyle::Filled,
            colors: ButtonColors::primary(),
            state: ButtonState::Normal,
            padding: Insets::all(8.0),
            border_radius: 6.0,
            border_width: 1.5,
            min_width: Some(36.0),
            min_height: Some(36.0),
            font_size: 14.0,
            font_weight: FontWeight::Medium,
            disabled: false,
            leading: Some(Box::new(icon)),
            trailing: None,
            icon_spacing: 0.0,
            on_press: None,
        }
    }

    // ========================================================================
    // BUILDER METHODS - STYLE
    // ========================================================================

    /// Set the visual style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.button_style = style;
        self
    }

    /// Set the color scheme
    pub fn colors(mut self, colors: ButtonColors) -> Self {
        self.colors = colors;
        self
    }

    /// Set background color (modifies colors)
    pub fn color(mut self, color: Color) -> Self {
        self.colors.background = color;
        self.colors.border = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.colors.foreground = color;
        self
    }

    // ========================================================================
    // BUILDER METHODS - LAYOUT
    // ========================================================================

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set uniform padding
    pub fn padding_all(mut self, value: f32) -> Self {
        self.padding = Insets::all(value);
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = Some(width);
        self
    }

    /// Set minimum height
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = Some(height);
        self
    }

    // ========================================================================
    // BUILDER METHODS - TEXT
    // ========================================================================

    /// Set font size
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    /// Set font weight
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    // ========================================================================
    // BUILDER METHODS - STATE
    // ========================================================================

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        if disabled {
            self.state = ButtonState::Disabled;
        }
        self
    }

    /// Set the current state (for testing/preview)
    pub fn with_state(mut self, state: ButtonState) -> Self {
        self.state = state;
        self
    }

    // ========================================================================
    // BUILDER METHODS - ICONS
    // ========================================================================

    /// Set leading icon
    pub fn leading<W: Widget>(mut self, icon: W) -> Self {
        self.leading = Some(Box::new(icon));
        self
    }

    /// Set trailing icon
    pub fn trailing<W: Widget>(mut self, icon: W) -> Self {
        self.trailing = Some(Box::new(icon));
        self
    }

    /// Set icon spacing
    pub fn icon_spacing(mut self, spacing: f32) -> Self {
        self.icon_spacing = spacing;
        self
    }

    // ========================================================================
    // BUILDER METHODS - CALLBACK
    // ========================================================================

    /// Set the press callback
    pub fn on_press<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_press = Some(Arc::new(callback));
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get the label text
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Check if button is disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Get current state
    pub fn get_state(&self) -> ButtonState {
        self.state
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    /// Calculate content size
    fn content_size(&self) -> Size {
        // Estimate text size
        let char_width = self.font_size * 0.6;
        let text_width = self.label.len() as f32 * char_width;
        let text_height = self.font_size * 1.2;

        // Add icon space if present
        let mut total_width = text_width;
        if self.leading.is_some() {
            total_width += self.font_size + self.icon_spacing;
        }
        if self.trailing.is_some() {
            total_width += self.font_size + self.icon_spacing;
        }

        Size::new(total_width, text_height)
    }
}

impl Widget for Button {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let content = self.content_size();
        
        let mut width = content.width + self.padding.horizontal_total();
        let mut height = content.height + self.padding.vertical_total();

        // Apply minimums
        if let Some(min_w) = self.min_width {
            width = width.max(min_w);
        }
        if let Some(min_h) = self.min_height {
            height = height.max(min_h);
        }

        Size::new(
            constraints.constrain_width(width),
            constraints.constrain_height(height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Get colors for current state
        let bg_color = self.colors.background_for_state(self.state);
        let fg_color = self.colors.foreground_for_state(self.state);

        // Draw based on style
        match self.button_style {
            ButtonStyle::Filled | ButtonStyle::Elevated => {
                // Fill background
                canvas.draw_rounded_rect(
                    rect,
                    BorderRadius::all(self.border_radius),
                    &Paint::fill(bg_color),
                );
            }
            ButtonStyle::Outlined => {
                // Draw border only
                canvas.draw_rounded_rect(
                    rect,
                    BorderRadius::all(self.border_radius),
                    &Paint::stroke(self.colors.border, self.border_width),
                );
            }
            ButtonStyle::Text => {
                // No background
            }
        }

        // Draw label text (centered)
        if !self.label.is_empty() {
            let text_paint = Paint::fill(fg_color);
            let text_x = offset.dx + size.width / 2.0 - (self.label.len() as f32 * self.font_size * 0.3);
            let text_y = offset.dy + size.height / 2.0 - self.font_size / 2.0;
            
            canvas.draw_text(
                &self.label,
                venom_core::Point::new(text_x, text_y),
                &text_paint,
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
    fn test_button_creation() {
        let btn = Button::new("Click");
        assert_eq!(btn.label(), "Click");
        assert!(!btn.is_disabled());
    }

    #[test]
    fn test_button_styling() {
        let btn = Button::new("Test")
            .style(ButtonStyle::Outlined)
            .colors(ButtonColors::danger())
            .border_radius(8.0);
        
        assert_eq!(btn.button_style, ButtonStyle::Outlined);
        assert_eq!(btn.border_radius, 8.0);
    }

    #[test]
    fn test_button_disabled() {
        let btn = Button::new("Disabled").disabled(true);
        assert!(btn.is_disabled());
        assert_eq!(btn.get_state(), ButtonState::Disabled);
    }

    #[test]
    fn test_button_layout() {
        let btn = Button::new("Click Me")
            .padding_all(16.0)
            .min_width(100.0);
        
        let size = btn.layout(BoxConstraints::new());
        assert!(size.width >= 100.0);
    }

    #[test]
    fn test_button_colors() {
        let colors = ButtonColors::primary();
        
        assert_eq!(
            colors.background_for_state(ButtonState::Normal),
            colors.background
        );
        assert_eq!(
            colors.background_for_state(ButtonState::Hovered),
            colors.background_hovered
        );
    }

    #[test]
    fn test_button_with_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        let btn = Button::new("Test")
            .on_press(move || {
                called_clone.store(true, Ordering::SeqCst);
            });
        
        // Callback is stored but not auto-called
        assert!(btn.on_press.is_some());
    }
}
