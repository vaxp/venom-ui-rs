//! List Widgets - ListView, ListTile, GridView, ExpansionTile
//!
//! Widgets for displaying lists and grids of items.

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, Point};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// LIST TILE
// ============================================================================

/// Standard list item with title, subtitle, leading, and trailing widgets
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{ListTile, Text, Checkbox};
///
/// let tile = ListTile::new()
///     .title("Item Title")
///     .subtitle("Secondary text")
///     .leading(icon)
///     .trailing(Checkbox::new());
/// ```
#[derive(Default)]
pub struct ListTile {
    /// Title text
    title: Option<String>,
    /// Subtitle text
    subtitle: Option<String>,
    /// Leading widget (icon, avatar)
    leading: Option<BoxedWidget>,
    /// Trailing widget (checkbox, switch)
    trailing: Option<BoxedWidget>,
    /// Background color
    background: Color,
    /// Text color
    text_color: Color,
    /// Subtitle color
    subtitle_color: Color,
    /// Padding
    padding: Insets,
    /// Height (None = auto)
    height: Option<f32>,
    /// Whether enabled
    enabled: bool,
    /// Tap callback
    on_tap: Option<Box<dyn Fn() + Send + Sync>>,
    /// Selected state
    selected: bool,
    /// Selected background color
    selected_color: Color,
    /// Dense mode (smaller)
    dense: bool,
}

impl ListTile {
    /// Create a new list tile
    pub fn new() -> Self {
        Self {
            background: Color::TRANSPARENT,
            text_color: Color::rgb(0, 0, 0),
            subtitle_color: Color::rgb(128, 128, 128),
            padding: Insets::symmetric(16.0, 12.0),
            enabled: true,
            selected_color: Color::rgba(33, 150, 243, 30),
            ..Default::default()
        }
    }

    /// Set title text
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set subtitle text
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set leading widget
    pub fn leading<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.leading = Some(Box::new(widget));
        self
    }

    /// Set trailing widget
    pub fn trailing<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.trailing = Some(Box::new(widget));
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

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set tap callback
    pub fn on_tap<F: Fn() + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_tap = Some(Box::new(callback));
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Enable dense mode
    pub fn dense(mut self) -> Self {
        self.dense = true;
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    fn computed_height(&self) -> f32 {
        if let Some(h) = self.height {
            return h;
        }
        
        let base = if self.dense { 48.0 } else { 56.0 };
        if self.subtitle.is_some() {
            base + 20.0
        } else {
            base
        }
    }
}

impl Widget for ListTile {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            self.computed_height().min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Background
        let bg = if self.selected { self.selected_color } else { self.background };
        canvas.draw_rect(rect, &Paint::fill(bg));

        let mut content_x = offset.dx + self.padding.left;

        // Leading
        if let Some(leading) = &self.leading {
            let leading_y = offset.dy + (size.height - 40.0) / 2.0;
            leading.paint(canvas, Offset::new(content_x, leading_y));
            content_x += 56.0;
        }

        // Title and subtitle
        let title_y = if self.subtitle.is_some() {
            offset.dy + self.padding.top
        } else {
            offset.dy + size.height / 2.0 - 8.0
        };

        if let Some(title) = &self.title {
            let color = if self.enabled { self.text_color } else {
                Color::rgba(self.text_color.r, self.text_color.g, self.text_color.b, 128)
            };
            canvas.draw_text(
                title,
                Point::new(content_x, title_y),
                &Paint::fill(color),
                16.0,
            );
        }

        if let Some(subtitle) = &self.subtitle {
            let subtitle_y = title_y + 22.0;
            canvas.draw_text(
                subtitle,
                Point::new(content_x, subtitle_y),
                &Paint::fill(self.subtitle_color),
                14.0,
            );
        }

        // Trailing
        if let Some(trailing) = &self.trailing {
            let trailing_x = offset.dx + size.width - self.padding.right - 40.0;
            let trailing_y = offset.dy + (size.height - 40.0) / 2.0;
            trailing.paint(canvas, Offset::new(trailing_x, trailing_y));
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &[]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// LIST VIEW
// ============================================================================

/// Scrollable list of widgets
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{ListView, ListTile};
///
/// let list = ListView::new()
///     .children(items.iter().map(|item| {
///         ListTile::new().title(item.name)
///     }).collect());
/// ```
#[derive(Default)]
pub struct ListView {
    /// Children widgets
    children: Vec<BoxedWidget>,
    /// Padding
    padding: Insets,
    /// Item spacing
    spacing: f32,
    /// Background
    background: Option<Color>,
    /// Scroll offset
    scroll_offset: f32,
    /// Dividers between items
    show_dividers: bool,
    /// Divider color
    divider_color: Color,
}

impl ListView {
    /// Create a new list view
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            divider_color: Color::rgba(0, 0, 0, 30),
            ..Default::default()
        }
    }

    /// Add a child
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Set children
    pub fn children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set item spacing
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Show dividers between items
    pub fn dividers(mut self) -> Self {
        self.show_dividers = true;
        self
    }

    /// Set divider color
    pub fn divider_color(mut self, color: Color) -> Self {
        self.divider_color = color;
        self
    }
}

impl Widget for ListView {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let mut total_height = self.padding.top + self.padding.bottom;
        
        for (i, child) in self.children.iter().enumerate() {
            let child_constraints = BoxConstraints {
                min_width: 0.0,
                max_width: constraints.max_width - self.padding.left - self.padding.right,
                min_height: 0.0,
                max_height: f32::INFINITY,
            };
            let child_size = child.layout(child_constraints);
            total_height += child_size.height;
            
            if i < self.children.len() - 1 {
                total_height += self.spacing;
            }
        }

        Size::new(
            constraints.max_width,
            total_height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));

        // Background
        if let Some(bg) = self.background {
            let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
            canvas.draw_rect(rect, &Paint::fill(bg));
        }

        let mut y = offset.dy + self.padding.top - self.scroll_offset;
        let content_width = size.width - self.padding.left - self.padding.right;

        for (i, child) in self.children.iter().enumerate() {
            let child_constraints = BoxConstraints {
                min_width: 0.0,
                max_width: content_width,
                min_height: 0.0,
                max_height: f32::INFINITY,
            };
            let child_size = child.layout(child_constraints);

            child.paint(canvas, Offset::new(offset.dx + self.padding.left, y));
            y += child_size.height;

            // Draw divider
            if self.show_dividers && i < self.children.len() - 1 {
                let divider_rect = Rect::new(
                    offset.dx + self.padding.left,
                    y,
                    content_width,
                    1.0,
                );
                canvas.draw_rect(divider_rect, &Paint::fill(self.divider_color));
                y += 1.0;
            }

            y += self.spacing;
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.children
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// GRID VIEW
// ============================================================================

/// Grid layout of widgets
///
/// # Example
///
/// ```ignore
/// use venom_widgets::GridView;
///
/// let grid = GridView::count(3, items);
/// ```
#[derive(Default)]
pub struct GridView {
    /// Children
    children: Vec<BoxedWidget>,
    /// Number of columns
    columns: usize,
    /// Child aspect ratio
    child_aspect_ratio: f32,
    /// Padding
    padding: Insets,
    /// Horizontal spacing
    horizontal_spacing: f32,
    /// Vertical spacing
    vertical_spacing: f32,
    /// Background
    background: Option<Color>,
}

impl GridView {
    /// Create a grid with fixed column count
    pub fn count(columns: usize, children: Vec<BoxedWidget>) -> Self {
        Self {
            children,
            columns: columns.max(1),
            child_aspect_ratio: 1.0,
            horizontal_spacing: 8.0,
            vertical_spacing: 8.0,
            ..Default::default()
        }
    }

    /// Create empty grid
    pub fn new() -> Self {
        Self {
            columns: 2,
            child_aspect_ratio: 1.0,
            horizontal_spacing: 8.0,
            vertical_spacing: 8.0,
            ..Default::default()
        }
    }

    /// Set column count
    pub fn columns(mut self, columns: usize) -> Self {
        self.columns = columns.max(1);
        self
    }

    /// Add a child
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Set children
    pub fn children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }

    /// Set child aspect ratio
    pub fn aspect_ratio(mut self, ratio: f32) -> Self {
        self.child_aspect_ratio = ratio;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set spacing
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.horizontal_spacing = spacing;
        self.vertical_spacing = spacing;
        self
    }

    /// Set background
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl Widget for GridView {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let content_width = constraints.max_width - self.padding.left - self.padding.right;
        let total_spacing = self.horizontal_spacing * (self.columns - 1) as f32;
        let cell_width = (content_width - total_spacing) / self.columns as f32;
        let cell_height = cell_width / self.child_aspect_ratio;
        
        let num_rows = (self.children.len() + self.columns - 1) / self.columns;
        let total_height = self.padding.top 
            + self.padding.bottom 
            + num_rows as f32 * cell_height 
            + (num_rows.saturating_sub(1)) as f32 * self.vertical_spacing;

        Size::new(
            constraints.max_width,
            total_height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));

        if let Some(bg) = self.background {
            let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
            canvas.draw_rect(rect, &Paint::fill(bg));
        }

        let content_width = size.width - self.padding.left - self.padding.right;
        let total_spacing = self.horizontal_spacing * (self.columns - 1) as f32;
        let cell_width = (content_width - total_spacing) / self.columns as f32;
        let cell_height = cell_width / self.child_aspect_ratio;

        for (i, child) in self.children.iter().enumerate() {
            let row = i / self.columns;
            let col = i % self.columns;
            
            let x = offset.dx + self.padding.left 
                + col as f32 * (cell_width + self.horizontal_spacing);
            let y = offset.dy + self.padding.top 
                + row as f32 * (cell_height + self.vertical_spacing);

            child.paint(canvas, Offset::new(x, y));
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.children
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// EXPANSION TILE
// ============================================================================

/// Expandable list tile
pub struct ExpansionTile {
    /// Header content
    title: String,
    /// Subtitle
    subtitle: Option<String>,
    /// Leading widget
    leading: Option<BoxedWidget>,
    /// Children (shown when expanded)
    children: Vec<BoxedWidget>,
    /// Expanded state
    expanded: bool,
    /// Background
    background: Color,
    /// Header height
    header_height: f32,
}

impl ExpansionTile {
    /// Create a new expansion tile
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            leading: None,
            children: Vec::new(),
            expanded: false,
            background: Color::TRANSPARENT,
            header_height: 56.0,
        }
    }

    /// Set subtitle
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set leading widget
    pub fn leading<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.leading = Some(Box::new(widget));
        self
    }

    /// Add child
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.children.push(Box::new(child));
        self
    }

    /// Set children
    pub fn children(mut self, children: Vec<BoxedWidget>) -> Self {
        self.children = children;
        self
    }

    /// Set expanded state
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl Widget for ExpansionTile {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let mut height = self.header_height;
        
        if self.expanded {
            for child in &self.children {
                let child_constraints = BoxConstraints {
                    min_width: 0.0,
                    max_width: constraints.max_width - 32.0, // Indent
                    min_height: 0.0,
                    max_height: f32::INFINITY,
                };
                height += child.layout(child_constraints).height;
            }
        }

        Size::new(constraints.max_width, height.min(constraints.max_height))
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));

        // Background
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
        canvas.draw_rect(rect, &Paint::fill(self.background));

        // Header
        let header_rect = Rect::new(offset.dx, offset.dy, size.width, self.header_height);
        canvas.draw_rect(header_rect, &Paint::fill(Color::TRANSPARENT));

        // Title
        canvas.draw_text(
            &self.title,
            Point::new(offset.dx + 16.0, offset.dy + self.header_height / 2.0 - 8.0),
            &Paint::fill(Color::rgb(0, 0, 0)),
            16.0,
        );

        // Expand indicator
        let indicator = if self.expanded { "▼" } else { "▶" };
        canvas.draw_text(
            indicator,
            Point::new(offset.dx + size.width - 32.0, offset.dy + self.header_height / 2.0 - 8.0),
            &Paint::fill(Color::rgb(128, 128, 128)),
            14.0,
        );

        // Children (if expanded)
        if self.expanded {
            let mut y = offset.dy + self.header_height;
            for child in &self.children {
                child.paint(canvas, Offset::new(offset.dx + 32.0, y));
                let child_size = child.layout(BoxConstraints::new().with_max_width(size.width - 32.0));
                y += child_size.height;
            }
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.children
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
    use crate::SizedBox;

    #[test]
    fn test_list_tile() {
        let tile = ListTile::new()
            .title("Title")
            .subtitle("Subtitle")
            .selected(true);
        assert!(tile.selected);
    }

    #[test]
    fn test_list_view() {
        let view = ListView::new()
            .child(SizedBox::square(50.0))
            .child(SizedBox::square(50.0))
            .spacing(8.0);
        assert_eq!(view.children.len(), 2);
    }

    #[test]
    fn test_grid_view() {
        let items: Vec<BoxedWidget> = vec![
            Box::new(SizedBox::square(50.0)),
            Box::new(SizedBox::square(50.0)),
            Box::new(SizedBox::square(50.0)),
        ];
        let grid = GridView::count(2, items);
        assert_eq!(grid.columns, 2);
        assert_eq!(grid.children.len(), 3);
    }

    #[test]
    fn test_expansion_tile() {
        let tile = ExpansionTile::new("Title")
            .expanded(true)
            .child(SizedBox::square(50.0));
        assert!(tile.expanded);
        assert_eq!(tile.children.len(), 1);
    }
}
