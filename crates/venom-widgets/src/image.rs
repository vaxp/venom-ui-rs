//! Image widget - Display images
//!
//! This module provides widgets for displaying images.
//! Currently supports placeholder/box representation until
//! a proper image backend is integrated.
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{Image, BoxFit};
//!
//! let img = Image::asset("assets/logo.png")
//!     .width(200.0)
//!     .height(100.0)
//!     .fit(BoxFit::Cover);
//!
//! let network_img = Image::network("https://example.com/image.png")
//!     .placeholder(Color::hex("#333"));
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Size, Offset, Color, Rect, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::Widget;

// ============================================================================
// BOX FIT
// ============================================================================

/// How an image should be inscribed into a box
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BoxFit {
    /// Fill the box, possibly distorting the aspect ratio
    Fill,
    /// Scale to contain within box, maintaining aspect ratio
    #[default]
    Contain,
    /// Scale to cover the box, maintaining aspect ratio (may crop)
    Cover,
    /// Scale down only if needed, maintain aspect ratio
    ScaleDown,
    /// Do not scale (use original size)
    None,
    /// Scale width to fit, height may overflow
    FitWidth,
    /// Scale height to fit, width may overflow
    FitHeight,
}

// ============================================================================
// IMAGE REPEAT
// ============================================================================

/// How an image should be repeated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageRepeat {
    /// Do not repeat
    #[default]
    NoRepeat,
    /// Repeat in both directions
    Repeat,
    /// Repeat only horizontally
    RepeatX,
    /// Repeat only vertically
    RepeatY,
}

// ============================================================================
// IMAGE SOURCE
// ============================================================================

/// Source of an image
#[derive(Debug, Clone, PartialEq)]
pub enum ImageSource {
    /// Asset from the app bundle
    Asset(String),
    /// Network URL
    Network(String),
    /// File path
    File(String),
    /// Memory bytes (raw image data)
    Memory(Vec<u8>),
}

// ============================================================================
// IMAGE WIDGET
// ============================================================================

/// Image display widget
/// 
/// Displays images from various sources with configurable fit and styling.
/// 
/// # Sources
/// 
/// - `Image::asset()` - Load from app assets
/// - `Image::network()` - Load from URL
/// - `Image::file()` - Load from file path
/// - `Image::memory()` - Load from bytes
/// 
/// # Example
/// 
/// ```ignore
/// // Asset image
/// let logo = Image::asset("images/logo.png")
///     .width(100.0)
///     .fit(BoxFit::Contain);
/// 
/// // Network image with placeholder
/// let avatar = Image::network("https://example.com/avatar.jpg")
///     .width(50.0)
///     .height(50.0)
///     .fit(BoxFit::Cover)
///     .border_radius(25.0)
///     .placeholder(Color::hex("#374151"));
/// ```
pub struct Image {
    /// Image source
    source: ImageSource,
    
    /// Optional fixed width
    width: Option<f32>,
    
    /// Optional fixed height  
    height: Option<f32>,
    
    /// How to fit image in box
    fit: BoxFit,
    
    /// How to repeat image
    repeat: ImageRepeat,
    
    /// Border radius for rounded corners
    border_radius: f32,
    
    /// Placeholder color (shown while loading)
    placeholder_color: Color,
    
    /// Error color (shown if image fails to load)
    error_color: Color,
    
    /// Opacity (0.0 to 1.0)
    opacity: f32,
    
    /// Original image dimensions (if known)
    intrinsic_size: Option<Size>,
    
    /// Image loaded state (would be managed by framework)
    is_loaded: bool,
    
    /// Image error state
    has_error: bool,
}

impl Image {
    // ========================================================================
    // CONSTRUCTORS
    // ========================================================================

    /// Create image from asset path
    pub fn asset(path: impl Into<String>) -> Self {
        Self {
            source: ImageSource::Asset(path.into()),
            width: None,
            height: None,
            fit: BoxFit::Contain,
            repeat: ImageRepeat::NoRepeat,
            border_radius: 0.0,
            placeholder_color: Color::hex("#374151"),
            error_color: Color::hex("#ef4444"),
            opacity: 1.0,
            intrinsic_size: None,
            is_loaded: false,
            has_error: false,
        }
    }

    /// Create image from network URL
    pub fn network(url: impl Into<String>) -> Self {
        Self {
            source: ImageSource::Network(url.into()),
            width: None,
            height: None,
            fit: BoxFit::Contain,
            repeat: ImageRepeat::NoRepeat,
            border_radius: 0.0,
            placeholder_color: Color::hex("#374151"),
            error_color: Color::hex("#ef4444"),
            opacity: 1.0,
            intrinsic_size: None,
            is_loaded: false,
            has_error: false,
        }
    }

    /// Create image from file path
    pub fn file(path: impl Into<String>) -> Self {
        Self {
            source: ImageSource::File(path.into()),
            width: None,
            height: None,
            fit: BoxFit::Contain,
            repeat: ImageRepeat::NoRepeat,
            border_radius: 0.0,
            placeholder_color: Color::hex("#374151"),
            error_color: Color::hex("#ef4444"),
            opacity: 1.0,
            intrinsic_size: None,
            is_loaded: false,
            has_error: false,
        }
    }

    /// Create image from memory bytes
    pub fn memory(data: Vec<u8>) -> Self {
        Self {
            source: ImageSource::Memory(data),
            width: None,
            height: None,
            fit: BoxFit::Contain,
            repeat: ImageRepeat::NoRepeat,
            border_radius: 0.0,
            placeholder_color: Color::hex("#374151"),
            error_color: Color::hex("#ef4444"),
            opacity: 1.0,
            intrinsic_size: None,
            is_loaded: false,
            has_error: false,
        }
    }

    // ========================================================================
    // BUILDER - DIMENSIONS
    // ========================================================================

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

    /// Set both width and height
    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    // ========================================================================
    // BUILDER - FIT AND REPEAT
    // ========================================================================

    /// Set how image fits in its box
    pub fn fit(mut self, fit: BoxFit) -> Self {
        self.fit = fit;
        self
    }

    /// Set image repeat mode
    pub fn repeat(mut self, repeat: ImageRepeat) -> Self {
        self.repeat = repeat;
        self
    }

    // ========================================================================
    // BUILDER - STYLING
    // ========================================================================

    /// Set border radius for rounded corners
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set placeholder color (shown while loading)
    pub fn placeholder(mut self, color: Color) -> Self {
        self.placeholder_color = color;
        self
    }

    /// Set error color (shown if image fails to load)
    pub fn error_color(mut self, color: Color) -> Self {
        self.error_color = color;
        self
    }

    /// Set opacity (0.0 to 1.0)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get the image source
    pub fn source(&self) -> &ImageSource {
        &self.source
    }

    /// Check if image is loaded
    pub fn is_loaded(&self) -> bool {
        self.is_loaded
    }

    /// Check if image failed to load
    pub fn has_error(&self) -> bool {
        self.has_error
    }
}

impl Widget for Image {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        // Use explicit dimensions if provided
        let width = self.width.unwrap_or_else(|| {
            self.intrinsic_size
                .map(|s| s.width)
                .unwrap_or(100.0) // Default placeholder size
        });
        
        let height = self.height.unwrap_or_else(|| {
            self.intrinsic_size
                .map(|s| s.height)
                .unwrap_or(100.0)
        });

        Size::new(
            constraints.constrain_width(width),
            constraints.constrain_height(height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Determine color based on state
        let color = if self.has_error {
            self.error_color
        } else if !self.is_loaded {
            self.placeholder_color
        } else {
            // Would draw actual image here
            self.placeholder_color
        };

        // Draw with opacity
        let color_with_opacity = Color::rgba(
            color.r,
            color.g,
            color.b,
            (color.a as f32 * self.opacity) as u8,
        );

        // Draw placeholder/image rectangle
        if self.border_radius > 0.0 {
            canvas.draw_rounded_rect(
                rect,
                BorderRadius::all(self.border_radius),
                &Paint::fill(color_with_opacity),
            );
        } else {
            canvas.draw_rect(rect, &Paint::fill(color_with_opacity));
        }

        // Draw loading indicator (simple X for now)
        if !self.is_loaded && !self.has_error {
            // Could draw loading animation
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ICON WIDGET
// ============================================================================

/// Icon widget - displays icons from an icon font or SVG
/// 
/// # Example
/// 
/// ```ignore
/// let icon = Icon::new(Icons::HOME)
///     .size(24.0)
///     .color(Color::WHITE);
/// ```
pub struct Icon {
    /// Icon identifier (would be codepoint or name)
    icon: String,
    
    /// Size in pixels
    size: f32,
    
    /// Icon color
    color: Color,
    
    /// Semantic label for accessibility
    semantic_label: Option<String>,
}

impl Icon {
    /// Create a new icon
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            size: 24.0,
            color: Color::BLACK,
            semantic_label: None,
        }
    }

    /// Set icon size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size.max(1.0);
        self
    }

    /// Set icon color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set semantic label for accessibility
    pub fn semantic_label(mut self, label: impl Into<String>) -> Self {
        self.semantic_label = Some(label.into());
        self
    }

    /// Get the icon identifier
    pub fn icon_name(&self) -> &str {
        &self.icon
    }
}

impl Widget for Icon {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        Size::new(
            constraints.constrain_width(self.size),
            constraints.constrain_height(self.size),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Draw icon as text (icon fonts work this way)
        // For SVG icons, would need path rendering
        canvas.draw_text(
            &self.icon,
            venom_core::Point::new(offset.dx, offset.dy),
            &Paint::fill(self.color),
            self.size,
        );
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
    fn test_image_asset() {
        let img = Image::asset("test.png")
            .width(100.0)
            .height(50.0);
        
        assert!(matches!(img.source, ImageSource::Asset(_)));
        assert_eq!(img.width, Some(100.0));
    }

    #[test]
    fn test_image_network() {
        let img = Image::network("https://example.com/image.png")
            .fit(BoxFit::Cover);
        
        assert!(matches!(img.source, ImageSource::Network(_)));
        assert_eq!(img.fit, BoxFit::Cover);
    }

    #[test]
    fn test_image_styling() {
        let img = Image::asset("test.png")
            .border_radius(8.0)
            .opacity(0.5);
        
        assert_eq!(img.border_radius, 8.0);
        assert_eq!(img.opacity, 0.5);
    }

    #[test]
    fn test_image_layout() {
        let img = Image::asset("test.png")
            .width(200.0)
            .height(150.0);
        
        let size = img.layout(BoxConstraints::new());
        assert_eq!(size.width, 200.0);
        assert_eq!(size.height, 150.0);
    }

    #[test]
    fn test_icon_creation() {
        let icon = Icon::new("home")
            .size(32.0)
            .color(Color::WHITE);
        
        assert_eq!(icon.icon_name(), "home");
        assert_eq!(icon.size, 32.0);
    }
}
