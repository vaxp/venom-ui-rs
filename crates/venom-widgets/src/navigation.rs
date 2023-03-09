//! Navigation Widgets - AppBar, TabBar, BottomNavBar
//!
//! This module provides navigation widgets for app structure:
//!
//! - [`AppBar`] - Top app bar with title and actions
//! - [`TabBar`] - Tab navigation
//! - [`Tab`] - Individual tab
//! - [`BottomNavBar`] - Bottom navigation bar
//! - [`BottomNavItem`] - Navigation item

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, BorderRadius, Point};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// APP BAR
// ============================================================================

/// Top application bar
///
/// `AppBar` provides a top bar with:
/// - Title
/// - Leading widget (usually back button)
/// - Action buttons
/// - Background color
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{AppBar, Button, Text};
///
/// let app_bar = AppBar::new()
///     .title("My App")
///     .leading(back_button)
///     .actions(vec![settings_button, profile_button]);
/// ```
#[derive(Default)]
pub struct AppBar {
    /// Title text
    title: Option<String>,
    /// Title widget (alternative to text)
    title_widget: Option<BoxedWidget>,
    /// Leading widget
    leading: Option<BoxedWidget>,
    /// Action widgets
    actions: Vec<BoxedWidget>,
    /// Background color
    background: Color,
    /// Foreground (text) color
    foreground: Color,
    /// Height
    height: f32,
    /// Padding
    padding: Insets,
    /// Elevation
    elevation: f32,
    /// Center title
    center_title: bool,
}

impl AppBar {
    /// Create a new app bar
    pub fn new() -> Self {
        Self {
            background: Color::rgb(33, 150, 243), // Material Blue
            foreground: Color::WHITE,
            height: 56.0,
            padding: Insets::symmetric(16.0, 8.0),
            elevation: 4.0,
            center_title: false,
            ..Default::default()
        }
    }

    /// Set title text
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set title widget
    pub fn title_widget<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.title_widget = Some(Box::new(widget));
        self
    }

    /// Set leading widget
    pub fn leading<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.leading = Some(Box::new(widget));
        self
    }

    /// Add action widget
    pub fn action<W: Widget + 'static>(mut self, widget: W) -> Self {
        self.actions.push(Box::new(widget));
        self
    }

    /// Set multiple actions
    pub fn actions(mut self, widgets: Vec<BoxedWidget>) -> Self {
        self.actions = widgets;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set foreground color
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set elevation
    pub fn elevation(mut self, elevation: f32) -> Self {
        self.elevation = elevation;
        self
    }

    /// Center the title
    pub fn center_title(mut self, center: bool) -> Self {
        self.center_title = center;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }
}

impl Widget for AppBar {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            self.height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));
        
        // Draw shadow
        if self.elevation > 0.0 {
            let shadow_rect = Rect::new(
                offset.dx,
                offset.dy + self.height,
                size.width,
                self.elevation,
            );
            canvas.draw_rect(shadow_rect, &Paint::fill(Color::rgba(0, 0, 0, 30)));
        }

        // Draw background
        let rect = Rect::new(offset.dx, offset.dy, size.width, self.height);
        canvas.draw_rect(rect, &Paint::fill(self.background));

        // Draw leading
        let mut content_x = offset.dx + self.padding.left;
        if let Some(leading) = &self.leading {
            let leading_offset = Offset::new(content_x, offset.dy + self.padding.top);
            leading.paint(canvas, leading_offset);
            content_x += 48.0; // Standard leading width
        }

        // Draw title
        if let Some(title) = &self.title {
            let title_x = if self.center_title {
                offset.dx + size.width / 2.0 - (title.len() as f32 * 5.0)
            } else {
                content_x + 16.0
            };
            let title_y = offset.dy + self.height / 2.0 - 8.0;
            canvas.draw_text(
                title,
                Point::new(title_x, title_y),
                &Paint::fill(self.foreground),
                18.0,
            );
        } else if let Some(widget) = &self.title_widget {
            let widget_x = if self.center_title {
                offset.dx + size.width / 2.0 - 50.0
            } else {
                content_x + 16.0
            };
            widget.paint(canvas, Offset::new(widget_x, offset.dy + self.padding.top));
        }

        // Draw actions (from right)
        let mut action_x = offset.dx + size.width - self.padding.right;
        for action in self.actions.iter().rev() {
            action_x -= 48.0;
            let action_offset = Offset::new(action_x, offset.dy + self.padding.top);
            action.paint(canvas, action_offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &self.actions
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// TAB
// ============================================================================

/// Individual tab for TabBar
pub struct Tab {
    /// Tab label
    label: String,
    /// Tab icon (optional)
    icon: Option<BoxedWidget>,
    /// Whether tab is selected
    selected: bool,
}

impl Tab {
    /// Create a new tab
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            selected: false,
        }
    }

    /// Set icon widget
    pub fn icon<W: Widget + 'static>(mut self, icon: W) -> Self {
        self.icon = Some(Box::new(icon));
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Get the label
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Check if selected
    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

// ============================================================================
// TAB BAR
// ============================================================================

/// Tab navigation bar
pub struct TabBar {
    /// Tabs
    tabs: Vec<Tab>,
    /// Selected index
    selected_index: usize,
    /// Background color
    background: Color,
    /// Selected tab color
    selected_color: Color,
    /// Unselected tab color
    unselected_color: Color,
    /// Indicator color
    indicator_color: Color,
    /// Height
    height: f32,
    /// On tab selected callback
    on_select: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Default for TabBar {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            selected_index: 0,
            background: Color::rgb(33, 150, 243),
            selected_color: Color::WHITE,
            unselected_color: Color::rgba(255, 255, 255, 178),
            indicator_color: Color::WHITE,
            height: 48.0,
            on_select: None,
        }
    }
}

impl TabBar {
    /// Create a new tab bar
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tab
    pub fn tab(mut self, tab: Tab) -> Self {
        self.tabs.push(tab);
        self
    }

    /// Set tabs
    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        self.tabs = tabs;
        self
    }

    /// Set selected index
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set selected tab color
    pub fn selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    /// Set indicator color
    pub fn indicator_color(mut self, color: Color) -> Self {
        self.indicator_color = color;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set selection callback
    pub fn on_select<F: Fn(usize) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }
}

impl Widget for TabBar {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            self.height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));
        
        // Background
        let rect = Rect::new(offset.dx, offset.dy, size.width, self.height);
        canvas.draw_rect(rect, &Paint::fill(self.background));

        if self.tabs.is_empty() {
            return;
        }

        let tab_width = size.width / self.tabs.len() as f32;

        for (i, tab) in self.tabs.iter().enumerate() {
            let tab_x = offset.dx + i as f32 * tab_width;
            let is_selected = i == self.selected_index;
            
            let text_color = if is_selected {
                self.selected_color
            } else {
                self.unselected_color
            };

            // Draw tab label
            let label_x = tab_x + tab_width / 2.0 - (tab.label.len() as f32 * 4.0);
            let label_y = offset.dy + self.height / 2.0 - 6.0;
            canvas.draw_text(
                &tab.label,
                Point::new(label_x, label_y),
                &Paint::fill(text_color),
                14.0,
            );

            // Draw indicator for selected tab
            if is_selected {
                let indicator_rect = Rect::new(
                    tab_x,
                    offset.dy + self.height - 3.0,
                    tab_width,
                    3.0,
                );
                canvas.draw_rect(indicator_rect, &Paint::fill(self.indicator_color));
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// BOTTOM NAV ITEM
// ============================================================================

/// Item for bottom navigation bar
pub struct BottomNavItem {
    /// Label
    label: String,
    /// Icon widget
    icon: Option<BoxedWidget>,
    /// Selected icon (optional)
    selected_icon: Option<BoxedWidget>,
}

impl BottomNavItem {
    /// Create a new nav item
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            selected_icon: None,
        }
    }

    /// Set icon widget
    pub fn icon<W: Widget + 'static>(mut self, icon: W) -> Self {
        self.icon = Some(Box::new(icon));
        self
    }

    /// Set selected icon
    pub fn selected_icon<W: Widget + 'static>(mut self, icon: W) -> Self {
        self.selected_icon = Some(Box::new(icon));
        self
    }

    /// Get label
    pub fn label(&self) -> &str {
        &self.label
    }
}

// ============================================================================
// BOTTOM NAV BAR
// ============================================================================

/// Bottom navigation bar
pub struct BottomNavBar {
    /// Navigation items
    items: Vec<BottomNavItem>,
    /// Selected index
    selected_index: usize,
    /// Background color
    background: Color,
    /// Selected item color
    selected_color: Color,
    /// Unselected item color
    unselected_color: Color,
    /// Height
    height: f32,
    /// Show labels
    show_labels: bool,
    /// On item selected
    on_select: Option<Box<dyn Fn(usize) + Send + Sync>>,
}

impl Default for BottomNavBar {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            background: Color::WHITE,
            selected_color: Color::rgb(33, 150, 243),
            unselected_color: Color::rgb(128, 128, 128),
            height: 56.0,
            show_labels: true,
            on_select: None,
        }
    }
}

impl BottomNavBar {
    /// Create a new bottom nav bar
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an item
    pub fn item(mut self, item: BottomNavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set items
    pub fn items(mut self, items: Vec<BottomNavItem>) -> Self {
        self.items = items;
        self
    }

    /// Set selected index
    pub fn selected(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set selected color
    pub fn selected_color(mut self, color: Color) -> Self {
        self.selected_color = color;
        self
    }

    /// Set unselected color
    pub fn unselected_color(mut self, color: Color) -> Self {
        self.unselected_color = color;
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show or hide labels
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
        self
    }

    /// Set selection callback
    pub fn on_select<F: Fn(usize) + Send + Sync + 'static>(mut self, callback: F) -> Self {
        self.on_select = Some(Box::new(callback));
        self
    }
}

impl Widget for BottomNavBar {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.max_width,
            self.height.min(constraints.max_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new().with_max_width(1000.0));
        
        // Draw top shadow
        let shadow_rect = Rect::new(offset.dx, offset.dy - 1.0, size.width, 1.0);
        canvas.draw_rect(shadow_rect, &Paint::fill(Color::rgba(0, 0, 0, 30)));

        // Background
        let rect = Rect::new(offset.dx, offset.dy, size.width, self.height);
        canvas.draw_rect(rect, &Paint::fill(self.background));

        if self.items.is_empty() {
            return;
        }

        let item_width = size.width / self.items.len() as f32;

        for (i, item) in self.items.iter().enumerate() {
            let item_x = offset.dx + i as f32 * item_width;
            let is_selected = i == self.selected_index;
            
            let color = if is_selected {
                self.selected_color
            } else {
                self.unselected_color
            };

            // Draw icon placeholder (circle)
            let icon_size = 24.0;
            let icon_x = item_x + item_width / 2.0;
            let icon_y = offset.dy + if self.show_labels { 12.0 } else { self.height / 2.0 - icon_size / 2.0 };
            canvas.draw_circle(
                Point::new(icon_x, icon_y + icon_size / 2.0),
                icon_size / 2.0,
                &Paint::fill(color),
            );

            // Draw label
            if self.show_labels {
                let label_x = item_x + item_width / 2.0 - (item.label.len() as f32 * 3.0);
                let label_y = offset.dy + self.height - 12.0;
                canvas.draw_text(
                    &item.label,
                    Point::new(label_x, label_y),
                    &Paint::fill(color),
                    11.0,
                );
            }
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
    fn test_app_bar() {
        let bar = AppBar::new()
            .title("Test")
            .background(Color::RED);
        assert_eq!(bar.title, Some("Test".to_string()));
    }

    #[test]
    fn test_tab_bar() {
        let bar = TabBar::new()
            .tab(Tab::new("Tab 1"))
            .tab(Tab::new("Tab 2"))
            .selected(1);
        assert_eq!(bar.tabs.len(), 2);
        assert_eq!(bar.selected_index, 1);
    }

    #[test]
    fn test_bottom_nav() {
        let nav = BottomNavBar::new()
            .item(BottomNavItem::new("Home"))
            .item(BottomNavItem::new("Search"))
            .selected(0);
        assert_eq!(nav.items.len(), 2);
    }
}
