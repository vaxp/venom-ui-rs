//! Core geometric and color types for VenomUI
//!
//! All types are designed to be:
//! - Copy-able for efficiency
//! - Debug-printable for easy debugging
//! - Comparable for testing
//! - Builder-pattern friendly (no macros needed)

use std::ops::{Add, Sub, Mul};

// ============================================================================
// COLOR
// ============================================================================

/// RGBA Color with 8-bit components
/// 
/// # Example
/// ```
/// use venom_core::Color;
/// 
/// let red = Color::rgb(255, 0, 0);
/// let semi_transparent = Color::rgba(255, 255, 255, 128);
/// let from_hex = Color::hex("#1a1a2e");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255, 255 = fully opaque)
    pub a: u8,
}

impl Color {
    /// Create a new color with RGBA components
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color with RGB components (fully opaque)
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Create color from hex string (e.g., "#FF5733" or "FF5733")
    pub fn hex(s: &str) -> Self {
        let s = s.trim_start_matches('#');
        
        let (r, g, b, a) = match s.len() {
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
                (r, g, b, 255)
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
                let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
                (r, g, b, a)
            }
            _ => (0, 0, 0, 255),
        };
        
        Self::rgba(r, g, b, a)
    }

    /// Create color from 32-bit ARGB value
    pub const fn from_argb(argb: u32) -> Self {
        Self {
            a: ((argb >> 24) & 0xFF) as u8,
            r: ((argb >> 16) & 0xFF) as u8,
            g: ((argb >> 8) & 0xFF) as u8,
            b: (argb & 0xFF) as u8,
        }
    }

    /// Convert to 32-bit ARGB value
    pub const fn to_argb(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }

    /// Create a new color with modified alpha
    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    /// Linearly interpolate between two colors
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;
        
        Self {
            r: (self.r as f32 * inv_t + other.r as f32 * t) as u8,
            g: (self.g as f32 * inv_t + other.g as f32 * t) as u8,
            b: (self.b as f32 * inv_t + other.b as f32 * t) as u8,
            a: (self.a as f32 * inv_t + other.a as f32 * t) as u8,
        }
    }

    // Predefined colors
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
}

// ============================================================================
// POINT
// ============================================================================

/// 2D point with floating-point coordinates
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Origin point (0, 0)
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Calculate distance to another point
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate squared distance (faster, avoids sqrt)
    pub fn distance_squared_to(self, other: Self) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        dx * dx + dy * dy
    }
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

// ============================================================================
// SIZE
// ============================================================================

/// 2D size with non-negative dimensions
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size (clamps negative values to 0)
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    /// Create a square size
    pub fn square(dimension: f32) -> Self {
        Self::new(dimension, dimension)
    }

    /// Zero size
    pub const ZERO: Self = Self { width: 0.0, height: 0.0 };

    /// Infinite size (for unconstrained layouts)
    pub const INFINITY: Self = Self { width: f32::INFINITY, height: f32::INFINITY };

    /// Check if size is finite
    pub fn is_finite(self) -> bool {
        self.width.is_finite() && self.height.is_finite()
    }

    /// Check if size is empty (zero area)
    pub fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Calculate area
    pub fn area(self) -> f32 {
        self.width * self.height
    }

    /// Get the aspect ratio (width / height)
    pub fn aspect_ratio(self) -> f32 {
        if self.height == 0.0 {
            0.0
        } else {
            self.width / self.height
        }
    }
}

impl Mul<f32> for Size {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

// ============================================================================
// OFFSET
// ============================================================================

/// 2D offset/displacement vector
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Offset {
    pub dx: f32,
    pub dy: f32,
}

impl Offset {
    /// Create a new offset
    pub const fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    /// Zero offset
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Convert to Point
    pub const fn to_point(self) -> Point {
        Point::new(self.dx, self.dy)
    }
}

impl Add for Offset {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.dx + rhs.dx, self.dy + rhs.dy)
    }
}

impl Sub for Offset {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.dx - rhs.dx, self.dy - rhs.dy)
    }
}

// ============================================================================
// RECT
// ============================================================================

/// Axis-aligned rectangle defined by position and size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left X coordinate
    pub x: f32,
    /// Top-left Y coordinate
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(0.0),
            height: height.max(0.0),
        }
    }

    /// Create from position and size
    pub fn from_point_size(point: Point, size: Size) -> Self {
        Self::new(point.x, point.y, size.width, size.height)
    }

    /// Create from two corner points
    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p2.x - p1.x).abs();
        let height = (p2.y - p1.y).abs();
        Self::new(x, y, width, height)
    }

    /// Create from left, top, right, bottom (LTRB)
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self::new(left, top, right - left, bottom - top)
    }

    /// Zero rectangle at origin
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };

    // Getters for edges
    pub fn left(&self) -> f32 { self.x }
    pub fn top(&self) -> f32 { self.y }
    pub fn right(&self) -> f32 { self.x + self.width }
    pub fn bottom(&self) -> f32 { self.y + self.height }

    // Getters for corners
    pub fn top_left(&self) -> Point { Point::new(self.x, self.y) }
    pub fn top_right(&self) -> Point { Point::new(self.right(), self.y) }
    pub fn bottom_left(&self) -> Point { Point::new(self.x, self.bottom()) }
    pub fn bottom_right(&self) -> Point { Point::new(self.right(), self.bottom()) }

    /// Get center point
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Get size
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Check if point is inside rectangle
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.right()
            && point.y >= self.y
            && point.y < self.bottom()
    }

    /// Check if rectangles overlap
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    /// Get intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if x < right && y < bottom {
            Some(Rect::new(x, y, right - x, bottom - y))
        } else {
            None
        }
    }

    /// Calculate the union of two rectangles
    pub fn union(&self, other: Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Rect::new(x, y, right - x, bottom - y)
    }

    /// Expand rectangle by given amount on all sides
    pub fn inflate(&self, delta: f32) -> Self {
        Self::new(
            self.x - delta,
            self.y - delta,
            self.width + delta * 2.0,
            self.height + delta * 2.0,
        )
    }

    /// Translate rectangle by offset
    pub fn translate(&self, offset: Offset) -> Self {
        Self::new(self.x + offset.dx, self.y + offset.dy, self.width, self.height)
    }
}

// ============================================================================
// INSETS (EdgeInsets in Flutter)
// ============================================================================

/// Edge insets (padding/margin)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Insets {
    /// Create insets with individual values
    pub const fn new(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }

    /// Create uniform insets on all sides
    pub const fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    /// Create symmetric insets
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// Create horizontal-only insets
    pub const fn horizontal(value: f32) -> Self {
        Self::new(0.0, value, 0.0, value)
    }

    /// Create vertical-only insets
    pub const fn vertical(value: f32) -> Self {
        Self::new(value, 0.0, value, 0.0)
    }

    /// Zero insets
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Total horizontal insets
    pub fn horizontal_total(&self) -> f32 {
        self.left + self.right
    }

    /// Total vertical insets
    pub fn vertical_total(&self) -> f32 {
        self.top + self.bottom
    }

    /// Deflate a rectangle by these insets
    pub fn deflate_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.left,
            rect.y + self.top,
            rect.width - self.horizontal_total(),
            rect.height - self.vertical_total(),
        )
    }

    /// Inflate a rectangle by these insets
    pub fn inflate_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - self.left,
            rect.y - self.top,
            rect.width + self.horizontal_total(),
            rect.height + self.vertical_total(),
        )
    }
}

// ============================================================================
// BORDER RADIUS
// ============================================================================

/// Border radius for rounded rectangles
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl BorderRadius {
    /// Create border radius with individual values
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self { top_left, top_right, bottom_right, bottom_left }
    }

    /// Create uniform border radius
    pub const fn all(radius: f32) -> Self {
        Self::new(radius, radius, radius, radius)
    }

    /// Zero border radius (sharp corners)
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Check if all radii are zero
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0
            && self.top_right == 0.0
            && self.bottom_right == 0.0
            && self.bottom_left == 0.0
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_hex() {
        let color = Color::hex("#FF5733");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_lerp() {
        let black = Color::BLACK;
        let white = Color::WHITE;
        let gray = black.lerp(white, 0.5);
        assert_eq!(gray.r, 127);
        assert_eq!(gray.g, 127);
        assert_eq!(gray.b, 127);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert!((p1.distance_to(p2) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 100.0, 100.0);
        assert!(rect.contains(Point::new(50.0, 50.0)));
        assert!(!rect.contains(Point::new(5.0, 50.0)));
    }

    #[test]
    fn test_rect_intersection() {
        let r1 = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r2 = Rect::new(50.0, 50.0, 100.0, 100.0);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection, Rect::new(50.0, 50.0, 50.0, 50.0));
    }

    #[test]
    fn test_insets_deflate() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let insets = Insets::all(10.0);
        let deflated = insets.deflate_rect(rect);
        assert_eq!(deflated, Rect::new(10.0, 10.0, 80.0, 80.0));
    }
}
