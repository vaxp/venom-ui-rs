//! Text widget - Renders text with styling
//!
//! Text is a fundamental widget for displaying text content.
//! It supports various text styling options like font size, color, and alignment.
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Text, TextStyle, Color};
//!
//! let title = Text::new("Welcome to VenomUI")
//!     .size(24.0)
//!     .color(Color::WHITE)
//!     .bold();
//!
//! let body = Text::new("This is body text")
//!     .size(16.0)
//!     .color(Color::hex("#cccccc"));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Rect};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// TEXT ALIGN
// ============================================================================

/// Horizontal text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlign {
    /// Align to the left (start)
    #[default]
    Left,
    /// Align to the center
    Center,
    /// Align to the right (end)
    Right,
}

// ============================================================================
// FONT WEIGHT
// ============================================================================

/// Font weight (boldness)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontWeight {
    /// Thin (100)
    Thin,
    /// Light (300)
    Light,
    /// Normal/Regular (400)
    #[default]
    Normal,
    /// Medium (500)
    Medium,
    /// Semi-bold (600)
    SemiBold,
    /// Bold (700)
    Bold,
    /// Extra bold (800)
    ExtraBold,
    /// Black (900)
    Black,
}

impl FontWeight {
    /// Get numeric weight value (100-900)
    pub fn value(self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
        }
    }
}

// ============================================================================
// FONT STYLE
// ============================================================================

/// Font style (italic, normal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    /// Normal upright text
    #[default]
    Normal,
    /// Italic text
    Italic,
}

// ============================================================================
// TEXT OVERFLOW
// ============================================================================

/// How to handle text that overflows its container
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextOverflow {
    /// Clip the text
    #[default]
    Clip,
    /// Show ellipsis (...)
    Ellipsis,
    /// Allow text to overflow
    Visible,
}

// ============================================================================
// TEXT STYLE
// ============================================================================

/// Styling for text
/// 
/// # Example
/// 
/// ```ignore
/// let heading_style = TextStyle::new()
///     .size(32.0)
///     .weight(FontWeight::Bold)
///     .color(Color::WHITE);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font size in pixels
    pub font_size: f32,
    
    /// Text color
    pub color: Color,
    
    /// Font weight (boldness)
    pub weight: FontWeight,
    
    /// Font style (normal/italic)
    pub style: FontStyle,
    
    /// Letter spacing (additional space between characters)
    pub letter_spacing: f32,
    
    /// Line height multiplier (1.0 = normal)
    pub line_height: f32,
    
    /// Font family name
    pub font_family: Option<String>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            color: Color::BLACK,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            letter_spacing: 0.0,
            line_height: 1.2,
            font_family: None,
        }
    }
}

impl TextStyle {
    /// Create a new default text style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set font size
    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size.max(1.0);
        self
    }

    /// Set text color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set font weight
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set font style
    pub fn style(mut self, style: FontStyle) -> Self {
        self.style = style;
        self
    }

    /// Set letter spacing
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// Set line height
    pub fn line_height(mut self, height: f32) -> Self {
        self.line_height = height.max(0.1);
        self
    }

    /// Set font family
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    // Shorthand methods

    /// Make text bold
    pub fn bold(self) -> Self {
        self.weight(FontWeight::Bold)
    }

    /// Make text italic
    pub fn italic(self) -> Self {
        self.style(FontStyle::Italic)
    }

    /// Make text light
    pub fn light(self) -> Self {
        self.weight(FontWeight::Light)
    }
}

// ============================================================================
// TEXT WIDGET
// ============================================================================

/// Text widget - displays text with styling
/// 
/// # Features
/// 
/// - Single-line and multi-line text
/// - Text styling (size, color, weight)
/// - Text alignment
/// - Overflow handling
/// 
/// # Example
/// 
/// ```ignore
/// // Simple text
/// let text = Text::new("Hello, World!");
/// 
/// // Styled text
/// let styled = Text::new("Important")
///     .size(24.0)
///     .bold()
///     .color(Color::RED);
/// 
/// // With full style
/// let custom = Text::styled("Custom", TextStyle::new()
///     .size(18.0)
///     .weight(FontWeight::SemiBold)
///     .color(Color::hex("#6366f1"))
/// );
/// ```
#[derive(Clone)]
pub struct Text {
    /// The text content
    content: String,
    
    /// Text styling
    style: TextStyle,
    
    /// Text alignment
    align: TextAlign,
    
    /// Maximum number of lines (None = unlimited)
    max_lines: Option<usize>,
    
    /// Overflow handling
    overflow: TextOverflow,
    
    /// Soft wrap (allow word wrapping)
    soft_wrap: bool,
}

impl Text {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create a new text widget
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: TextStyle::default(),
            align: TextAlign::Left,
            max_lines: None,
            overflow: TextOverflow::Clip,
            soft_wrap: true,
        }
    }

    /// Create text with a custom style
    pub fn styled(content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
            align: TextAlign::Left,
            max_lines: None,
            overflow: TextOverflow::Clip,
            soft_wrap: true,
        }
    }

    // ========================================================================
    // BUILDER METHODS - CONTENT
    // ========================================================================

    /// Set the text content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    // ========================================================================
    // BUILDER METHODS - STYLE SHORTCUTS
    // ========================================================================

    /// Set font size
    pub fn size(mut self, size: f32) -> Self {
        self.style.font_size = size.max(1.0);
        self
    }

    /// Set text color
    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    /// Make text bold
    pub fn bold(mut self) -> Self {
        self.style.weight = FontWeight::Bold;
        self
    }

    /// Make text italic
    pub fn italic(mut self) -> Self {
        self.style.style = FontStyle::Italic;
        self
    }

    /// Set font weight
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.style.weight = weight;
        self
    }

    /// Set letter spacing
    pub fn letter_spacing(mut self, spacing: f32) -> Self {
        self.style.letter_spacing = spacing;
        self
    }

    // ========================================================================
    // BUILDER METHODS - LAYOUT
    // ========================================================================

    /// Set text alignment
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Center the text
    pub fn centered(mut self) -> Self {
        self.align = TextAlign::Center;
        self
    }

    /// Set maximum number of lines
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Set overflow handling
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }

    /// Enable/disable soft wrapping
    pub fn soft_wrap(mut self, wrap: bool) -> Self {
        self.soft_wrap = wrap;
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get the text content
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Get the text style
    pub fn get_style(&self) -> &TextStyle {
        &self.style
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    /// Estimate text size (simplified - real implementation needs font metrics)
    fn estimate_size(&self, max_width: f32) -> Size {
        // Simplified estimation: ~0.6 * font_size per character width
        let char_width = self.style.font_size * 0.6;
        let line_height = self.style.font_size * self.style.line_height;
        
        if self.soft_wrap && max_width.is_finite() {
            // Calculate wrapped text size
            let chars_per_line = (max_width / char_width).floor() as usize;
            if chars_per_line > 0 {
                let num_lines = (self.content.len() + chars_per_line - 1) / chars_per_line;
                let num_lines = match self.max_lines {
                    Some(max) => num_lines.min(max),
                    None => num_lines,
                };
                Size::new(
                    (self.content.len().min(chars_per_line) as f32 * char_width).min(max_width),
                    num_lines as f32 * line_height,
                )
            } else {
                Size::new(0.0, line_height)
            }
        } else {
            // Single line
            Size::new(
                self.content.len() as f32 * char_width,
                line_height,
            )
        }
    }
}

impl Widget for Text {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let estimated = self.estimate_size(constraints.max_width);
        Size::new(
            constraints.constrain_width(estimated.width),
            constraints.constrain_height(estimated.height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if self.content.is_empty() {
            return;
        }

        let paint = Paint::fill(self.style.color);
        
        // Calculate x offset based on alignment
        let size = self.estimate_size(f32::INFINITY);
        let x_offset = match self.align {
            TextAlign::Left => 0.0,
            TextAlign::Center => -size.width / 2.0,
            TextAlign::Right => -size.width,
        };

        canvas.draw_text(
            &self.content,
            venom_core::Point::new(offset.dx + x_offset, offset.dy),
            &paint,
            self.style.font_size,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// RICH TEXT SPAN
// ============================================================================

/// A span of styled text within RichText
#[derive(Clone)]
pub struct TextSpan {
    /// Text content
    pub text: String,
    /// Style for this span
    pub style: Option<TextStyle>,
    /// Child spans
    pub children: Vec<TextSpan>,
}

impl TextSpan {
    /// Create a new text span
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: None,
            children: Vec::new(),
        }
    }

    /// Create with style
    pub fn styled(text: impl Into<String>, style: TextStyle) -> Self {
        Self {
            text: text.into(),
            style: Some(style),
            children: Vec::new(),
        }
    }

    /// Add a child span
    pub fn child(mut self, span: TextSpan) -> Self {
        self.children.push(span);
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
    fn test_text_creation() {
        let text = Text::new("Hello");
        assert_eq!(text.text(), "Hello");
    }

    #[test]
    fn test_text_styling() {
        let text = Text::new("Test")
            .size(24.0)
            .bold()
            .color(Color::RED);
        
        assert_eq!(text.get_style().font_size, 24.0);
        assert_eq!(text.get_style().weight, FontWeight::Bold);
        assert_eq!(text.get_style().color, Color::RED);
    }

    #[test]
    fn test_text_style_builder() {
        let style = TextStyle::new()
            .size(18.0)
            .bold()
            .italic()
            .color(Color::BLUE);
        
        assert_eq!(style.font_size, 18.0);
        assert_eq!(style.weight, FontWeight::Bold);
        assert_eq!(style.style, FontStyle::Italic);
    }

    #[test]
    fn test_text_alignment() {
        let text = Text::new("Centered").centered();
        assert_eq!(text.align, TextAlign::Center);
    }

    #[test]
    fn test_text_layout() {
        let text = Text::new("Hello World").size(14.0);
        let size = text.layout(BoxConstraints::new());
        
        // Should have some size
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);
    }

    #[test]
    fn test_font_weight_value() {
        assert_eq!(FontWeight::Normal.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
        assert_eq!(FontWeight::Thin.value(), 100);
    }

    #[test]
    fn test_text_span() {
        let span = TextSpan::new("Parent")
            .child(TextSpan::new("Child"));
        
        assert_eq!(span.text, "Parent");
        assert_eq!(span.children.len(), 1);
    }
}
