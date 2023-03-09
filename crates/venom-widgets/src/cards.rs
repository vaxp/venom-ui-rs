//! Card and Decorative Widgets - Card, Chip, Badge
//!
//! This module provides decorative container widgets:
//!
//! - [`Card`] - Elevated container with shadow and rounded corners
//! - [`Chip`] - Small labeled element for tags, filters, or selections
//! - [`Badge`] - Small notification indicator
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Card, Chip, Badge, Text};
//!
//! let card = Card::new()
//!     .elevation(4)
//!     .child(content);
//!
//! let tag = Chip::new("Rust")
//!     .on_delete(|| remove_tag());
//!
//! let notification = Badge::new(5);
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, BorderRadius, Point};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// CARD
// ============================================================================

/// Elevated container with shadow and rounded corners
///
/// `Card` provides a Material Design-style container with:
/// - Background color
/// - Rounded corners
/// - Optional shadow/elevation
/// - Padding
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{Card, Column, Text};
///
/// let profile_card = Card::new()
///     .elevation(2)
///     .border_radius(12.0)
///     .padding(Insets::all(16.0))
///     .child(Column::new()
///         .child(Text::new("John Doe"))
///         .child(Text::new("Developer"))
///     );
/// ```
#[derive(Default)]
pub struct Card {
    /// Child widget
    child: Option<BoxedWidget>,
    /// Background color
    color: Color,
    /// Border radius
    border_radius: f32,
    /// Elevation (shadow depth)
    elevation: f32,
    /// Shadow color
    shadow_color: Color,
    /// Padding
    padding: Insets,
    /// Border color
    border_color: Option<Color>,
    /// Border width
    border_width: f32,
    /// Width constraint
    width: Option<f32>,
    /// Height constraint
    height: Option<f32>,
    /// Margin
    margin: Insets,
    /// Cached size for paint
    cached_size: Option<Size>,
}

impl Card {
    /// Create a new card
    pub fn new() -> Self {
        Self {
            color: Color::rgb(255, 255, 255),
            border_radius: 8.0,
            elevation: 1.0,
            shadow_color: Color::rgba(0, 0, 0, 51), // ~20% opacity
            padding: Insets::all(16.0),
            border_width: 0.0,
            ..Default::default()
        }
    }

    /// Create an outlined card (no elevation, with border)
    pub fn outlined() -> Self {
        Self {
            color: Color::TRANSPARENT,
            border_radius: 8.0,
            elevation: 0.0,
            shadow_color: Color::TRANSPARENT,
            padding: Insets::all(16.0),
            border_color: Some(Color::rgba(128, 128, 128, 77)),
            border_width: 1.0,
            ..Default::default()
        }
    }

    /// Set the child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set boxed child
    pub fn child_boxed(mut self, child: BoxedWidget) -> Self {
        self.child = Some(child);
        self
    }

    /// Set background color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set elevation (shadow depth)
    pub fn elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Set shadow color
    pub fn shadow_color(mut self, color: Color) -> Self {
        self.shadow_color = color;
        self
    }

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

    /// Set border
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: Insets) -> Self {
        self.margin = margin;
        self
    }

    /// Set uniform margin
    pub fn margin_all(mut self, value: f32) -> Self {
        self.margin = Insets::all(value);
        self
    }
}

impl Widget for Card {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let margin_width = self.margin.left + self.margin.right;
        let margin_height = self.margin.top + self.margin.bottom;
        let padding_width = self.padding.left + self.padding.right;
        let padding_height = self.padding.top + self.padding.bottom;

        let available_width = self.width
            .unwrap_or(constraints.max_width)
            .min(constraints.max_width)
            - margin_width - padding_width;
        let available_height = self.height
            .unwrap_or(constraints.max_height)
            .min(constraints.max_height)
            - margin_height - padding_height;

        let child_size = if let Some(child) = &self.child {
            let child_constraints = BoxConstraints {
                min_width: 0.0,
                max_width: available_width.max(0.0),
                min_height: 0.0,
                max_height: available_height.max(0.0),
            };
            child.layout(child_constraints)
        } else {
            Size::new(0.0, 0.0)
        };

        let content_width = self.width.unwrap_or(child_size.width + padding_width);
        let content_height = self.height.unwrap_or(child_size.height + padding_height);

        Size::new(
            (content_width + margin_width).min(constraints.max_width).max(constraints.min_width),
            (content_height + margin_height).min(constraints.max_height).max(constraints.min_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Use a default size for painting if not laid out
        let size = self.cached_size.unwrap_or(Size::new(200.0, 100.0));
        
        let card_rect = Rect::new(
            offset.dx + self.margin.left,
            offset.dy + self.margin.top,
            size.width - self.margin.left - self.margin.right,
            size.height - self.margin.top - self.margin.bottom,
        );

        // Draw shadow
        if self.elevation > 0.0 {
            let shadow_offset = self.elevation * 0.5;
            let shadow_rect = Rect::new(
                card_rect.x + shadow_offset,
                card_rect.y + shadow_offset,
                card_rect.width,
                card_rect.height,
            );
            canvas.draw_rounded_rect(
                shadow_rect,
                BorderRadius::all(self.border_radius),
                &Paint::fill(self.shadow_color),
            );
        }

        // Draw card background
        canvas.draw_rounded_rect(
            card_rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(self.color),
        );

        // Draw border
        if let Some(border_color) = self.border_color {
            if self.border_width > 0.0 {
                canvas.draw_rounded_rect(
                    card_rect,
                    BorderRadius::all(self.border_radius),
                    &Paint::stroke(border_color, self.border_width),
                );
            }
        }

        // Paint child
        if let Some(child) = &self.child {
            let child_offset = Offset::new(
                offset.dx + self.margin.left + self.padding.left,
                offset.dy + self.margin.top + self.padding.top,
            );
            child.paint(canvas, child_offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        match &self.child {
            Some(child) => std::slice::from_ref(child),
            None => &[],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// CHIP
// ============================================================================

/// Small labeled element for tags, filters, or selections
#[derive(Default)]
pub struct Chip {
    /// Label text
    label: String,
    /// Background color
    color: Color,
    /// Text color
    text_color: Color,
    /// Selected state
    selected: bool,
    /// Selected background color
    selected_color: Color,
    /// Whether chip can be deleted
    deletable: bool,
    /// Click callback
    on_tap: Option<Box<dyn Fn() + Send + Sync>>,
    /// Border radius
    border_radius: f32,
    /// Padding
    padding: Insets,
    /// Enabled state
    enabled: bool,
}

impl Chip {
    /// Create a new chip with label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: Color::rgba(200, 200, 200, 255),
            text_color: Color::rgb(0, 0, 0),
            selected: false,
            selected_color: Color::rgb(100, 150, 255),
            deletable: false,
            on_tap: None,
            border_radius: 16.0,
            padding: Insets::symmetric(12.0, 6.0),
            enabled: true,
        }
    }

    /// Set background color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set selected background color
    pub fn selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    /// Make chip deletable
    pub fn deletable(mut self) -> Self {
        self.deletable = true;
        self
    }

    /// Set tap callback
    pub fn on_tap<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap = Some(Box::new(callback));
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Chip height constant
    fn chip_height(&self) -> f32 {
        32.0
    }
}

impl Widget for Chip {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let height = self.chip_height();
        let text_width = self.label.len() as f32 * 8.0;
        let delete_width = if self.deletable { 18.0 } else { 0.0 };
        
        let width = self.padding.left + text_width + delete_width + self.padding.right;

        Size::new(
            width.min(constraints.max_width).max(constraints.min_width),
            height.min(constraints.max_height).max(constraints.min_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        let bg_color = if self.selected { self.selected_color } else { self.color };
        let bg_color = if !self.enabled {
            Color::rgba(bg_color.r, bg_color.g, bg_color.b, bg_color.a / 2)
        } else {
            bg_color
        };
        
        canvas.draw_rounded_rect(
            rect,
            BorderRadius::all(self.border_radius),
            &Paint::fill(bg_color),
        );

        // Draw delete X if deletable
        if self.deletable {
            let x_size = 12.0;
            let x_x = offset.dx + size.width - self.padding.right - x_size;
            let x_y = offset.dy + (size.height - x_size) / 2.0;
            
            let paint = Paint::stroke(self.text_color, 1.5);
            canvas.draw_line(
                Point::new(x_x, x_y),
                Point::new(x_x + x_size, x_y + x_size),
                &paint,
            );
            canvas.draw_line(
                Point::new(x_x + x_size, x_y),
                Point::new(x_x, x_y + x_size),
                &paint,
            );
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// BADGE
// ============================================================================

/// Small notification indicator
#[derive(Default)]
pub struct Badge {
    /// Count to display
    count: Option<u32>,
    /// Maximum count
    max_count: u32,
    /// Background color
    color: Color,
    /// Text color
    text_color: Color,
    /// Size
    size: f32,
    /// Dot only mode
    is_dot: bool,
}

impl Badge {
    /// Create a badge with count
    pub fn new(count: u32) -> Self {
        Self {
            count: Some(count),
            max_count: 99,
            color: Color::rgb(255, 59, 48),
            text_color: Color::WHITE,
            size: 20.0,
            is_dot: false,
        }
    }

    /// Create a dot badge
    pub fn dot() -> Self {
        Self {
            count: None,
            max_count: 99,
            color: Color::rgb(255, 59, 48),
            text_color: Color::WHITE,
            size: 8.0,
            is_dot: true,
        }
    }

    /// Set the count
    pub fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self.is_dot = false;
        self
    }

    /// Set maximum displayed count
    pub fn max_count(mut self, max: u32) -> Self {
        self.max_count = max;
        self
    }

    /// Set background color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Get display text
    fn display_text(&self) -> String {
        match self.count {
            Some(n) if n > self.max_count => format!("{}+", self.max_count),
            Some(n) => n.to_string(),
            None => String::new(),
        }
    }
}

impl Widget for Badge {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        if self.is_dot {
            let size = self.size.min(constraints.max_width).min(constraints.max_height);
            Size::new(size, size)
        } else {
            let text = self.display_text();
            let min_width = if text.len() > 2 { self.size * 1.5 } else { self.size };
            Size::new(
                min_width.min(constraints.max_width).max(constraints.min_width),
                self.size.min(constraints.max_height).max(constraints.min_height),
            )
        }
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        
        if self.is_dot {
            let center = Point::new(
                offset.dx + size.width / 2.0,
                offset.dy + size.height / 2.0,
            );
            canvas.draw_circle(center, size.width / 2.0, &Paint::fill(self.color));
        } else {
            let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
            let radius = size.height / 2.0;
            canvas.draw_rounded_rect(rect, BorderRadius::all(radius), &Paint::fill(self.color));
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
    fn test_card_default() {
        let card = Card::new();
        assert_eq!(card.elevation, 1.0);
        assert_eq!(card.border_radius, 8.0);
    }

    #[test]
    fn test_card_outlined() {
        let card = Card::outlined();
        assert_eq!(card.elevation, 0.0);
        assert!(card.border_color.is_some());
    }

    #[test]
    fn test_chip() {
        let chip = Chip::new("Test")
            .selected(true)
            .deletable();
        assert!(chip.selected);
        assert!(chip.deletable);
    }

    #[test]
    fn test_badge_count() {
        let badge = Badge::new(5);
        assert_eq!(badge.count, Some(5));
        assert!(!badge.is_dot);
    }

    #[test]
    fn test_badge_dot() {
        let badge = Badge::dot();
        assert!(badge.is_dot);
        assert_eq!(badge.count, None);
    }

    #[test]
    fn test_badge_max_count() {
        let badge = Badge::new(150).max_count(99);
        assert_eq!(badge.display_text(), "99+");
    }
}
