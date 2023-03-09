//! BoxConstraints - Flutter-like layout constraint system
//!
//! BoxConstraints define the minimum and maximum dimensions a widget can have.
//! The layout algorithm works by:
//! 1. Parent passes constraints DOWN to children
//! 2. Children return their chosen SIZE back UP to parent
//! 3. Parent positions children based on their sizes

use crate::Size;

// ============================================================================
// BOX CONSTRAINTS
// ============================================================================

/// Layout constraints that define min/max dimensions for a widget
/// 
/// # Example
/// ```
/// use venom_core::{BoxConstraints, Size};
/// 
/// // Exact size constraint
/// let tight = BoxConstraints::tight(Size::new(100.0, 50.0));
/// 
/// // Size can be anything up to 200x200
/// let loose = BoxConstraints::loose(Size::new(200.0, 200.0));
/// 
/// // Must be at least 50 wide
/// let min_width = BoxConstraints::new()
///     .with_min_width(50.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxConstraints {
    /// Minimum width the widget can have
    pub min_width: f32,
    /// Maximum width the widget can have
    pub max_width: f32,
    /// Minimum height the widget can have
    pub min_height: f32,
    /// Maximum height the widget can have
    pub max_height: f32,
}

impl Default for BoxConstraints {
    fn default() -> Self {
        Self::new()
    }
}

impl BoxConstraints {
    /// Create unconstrained box constraints (0 to infinity)
    pub const fn new() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    // ========================================================================
    // BUILDER PATTERN METHODS
    // ========================================================================

    /// Set minimum width
    pub const fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Set maximum width
    pub const fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Set minimum height
    pub const fn with_min_height(mut self, min_height: f32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Set maximum height
    pub const fn with_max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    // ========================================================================
    // FACTORY METHODS
    // ========================================================================

    /// Create tight constraints that force exact size
    pub fn tight(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// Create tight constraints with width only
    pub fn tight_width(width: f32) -> Self {
        Self::new()
            .with_min_width(width)
            .with_max_width(width)
    }

    /// Create tight constraints with height only
    pub fn tight_height(height: f32) -> Self {
        Self::new()
            .with_min_height(height)
            .with_max_height(height)
    }

    /// Create loose constraints (0 to given size)
    pub fn loose(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }

    /// Create constraints that expand to fill available space
    pub fn expand() -> Self {
        Self {
            min_width: f32::INFINITY,
            max_width: f32::INFINITY,
            min_height: f32::INFINITY,
            max_height: f32::INFINITY,
        }
    }

    /// Create constraints that expand width but not height
    pub fn expand_width() -> Self {
        Self::new()
            .with_min_width(f32::INFINITY)
            .with_max_width(f32::INFINITY)
    }

    /// Create constraints that expand height but not width
    pub fn expand_height() -> Self {
        Self::new()
            .with_min_height(f32::INFINITY)
            .with_max_height(f32::INFINITY)
    }

    // ========================================================================
    // QUERY METHODS
    // ========================================================================

    /// Check if constraints have exact width
    pub fn has_tight_width(&self) -> bool {
        self.min_width == self.max_width
    }

    /// Check if constraints have exact height
    pub fn has_tight_height(&self) -> bool {
        self.min_height == self.max_height
    }

    /// Check if constraints have exact size
    pub fn is_tight(&self) -> bool {
        self.has_tight_width() && self.has_tight_height()
    }

    /// Check if there are any size constraints at all
    pub fn is_unconstrained(&self) -> bool {
        self.min_width == 0.0
            && self.max_width == f32::INFINITY
            && self.min_height == 0.0
            && self.max_height == f32::INFINITY
    }

    /// Check if constraints are valid (min <= max)
    pub fn is_normalized(&self) -> bool {
        self.min_width <= self.max_width
            && self.min_height <= self.max_height
            && self.min_width >= 0.0
            && self.min_height >= 0.0
    }

    /// Check if constraints have finite max dimensions
    pub fn has_bounded_width(&self) -> bool {
        self.max_width.is_finite()
    }

    /// Check if constraints have finite max height
    pub fn has_bounded_height(&self) -> bool {
        self.max_height.is_finite()
    }

    /// Check if both dimensions are bounded
    pub fn is_bounded(&self) -> bool {
        self.has_bounded_width() && self.has_bounded_height()
    }

    // ========================================================================
    // SIZE METHODS
    // ========================================================================

    /// Get the smallest size that satisfies constraints
    pub fn smallest(&self) -> Size {
        Size::new(self.min_width, self.min_height)
    }

    /// Get the biggest size that satisfies constraints
    /// Returns Size::INFINITY if unbounded
    pub fn biggest(&self) -> Size {
        Size::new(self.max_width, self.max_height)
    }

    /// Constrain a size to fit within these constraints
    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.min_width, self.max_width),
            size.height.clamp(self.min_height, self.max_height),
        )
    }

    /// Constrain width only
    pub fn constrain_width(&self, width: f32) -> f32 {
        width.clamp(self.min_width, self.max_width)
    }

    /// Constrain height only
    pub fn constrain_height(&self, height: f32) -> f32 {
        height.clamp(self.min_height, self.max_height)
    }

    /// Get the size with the given aspect ratio that fits within constraints
    pub fn constrain_dimensions_with_aspect_ratio(&self, aspect_ratio: f32) -> Size {
        if aspect_ratio <= 0.0 {
            return self.smallest();
        }

        // Try to fit width first
        let mut width = self.max_width;
        let mut height = width / aspect_ratio;

        // If height is too small, fit height instead
        if height < self.min_height {
            height = self.min_height;
            width = height * aspect_ratio;
        }

        // If height is too big, constrain it
        if height > self.max_height {
            height = self.max_height;
            width = height * aspect_ratio;
        }

        // If width is now too big, constrain it
        if width > self.max_width {
            width = self.max_width;
            height = width / aspect_ratio;
        }

        Size::new(
            width.clamp(self.min_width, self.max_width),
            height.clamp(self.min_height, self.max_height),
        )
    }

    // ========================================================================
    // TRANSFORMATION METHODS
    // ========================================================================

    /// Create new constraints with reduced max dimensions
    pub fn deflate(&self, insets: crate::Insets) -> Self {
        let horizontal = insets.horizontal_total();
        let vertical = insets.vertical_total();
        
        Self {
            min_width: (self.min_width - horizontal).max(0.0),
            max_width: (self.max_width - horizontal).max(0.0),
            min_height: (self.min_height - vertical).max(0.0),
            max_height: (self.max_height - vertical).max(0.0),
        }
    }

    /// Create loosened constraints (min becomes 0)
    pub fn loosen(&self) -> Self {
        Self {
            min_width: 0.0,
            max_width: self.max_width,
            min_height: 0.0,
            max_height: self.max_height,
        }
    }

    /// Create tightened constraints (force size to fit)
    pub fn tighten(&self, size: Size) -> Self {
        Self {
            min_width: size.width.clamp(self.min_width, self.max_width),
            max_width: size.width.clamp(self.min_width, self.max_width),
            min_height: size.height.clamp(self.min_height, self.max_height),
            max_height: size.height.clamp(self.min_height, self.max_height),
        }
    }

    /// Enforce constraints (ensure min <= max and min >= 0)
    pub fn normalize(&self) -> Self {
        Self {
            min_width: self.min_width.max(0.0),
            max_width: self.max_width.max(self.min_width),
            min_height: self.min_height.max(0.0),
            max_height: self.max_height.max(self.min_height),
        }
    }
}

// ============================================================================
// MAIN AXIS ALIGNMENT (for Row/Column)
// ============================================================================

/// Alignment along the main axis of a flex layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MainAxisAlignment {
    /// Place children at the start
    #[default]
    Start,
    /// Place children at the end
    End,
    /// Place children at the center
    Center,
    /// Evenly distribute space between children
    SpaceBetween,
    /// Evenly distribute space around children
    SpaceAround,
    /// Evenly distribute space, including before first and after last
    SpaceEvenly,
}

// ============================================================================
// CROSS AXIS ALIGNMENT
// ============================================================================

/// Alignment along the cross axis of a flex layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CrossAxisAlignment {
    /// Align children at the start
    Start,
    /// Align children at the end
    End,
    /// Center children
    #[default]
    Center,
    /// Stretch children to fill cross axis
    Stretch,
}

// ============================================================================
// ALIGNMENT (for Align widget)
// ============================================================================

/// Alignment position within a container
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Alignment {
    /// X position (-1.0 = left, 0.0 = center, 1.0 = right)
    pub x: f32,
    /// Y position (-1.0 = top, 0.0 = center, 1.0 = bottom)
    pub y: f32,
}

impl Alignment {
    /// Create custom alignment
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    // Predefined alignments
    pub const TOP_LEFT: Self = Self::new(-1.0, -1.0);
    pub const TOP_CENTER: Self = Self::new(0.0, -1.0);
    pub const TOP_RIGHT: Self = Self::new(1.0, -1.0);
    pub const CENTER_LEFT: Self = Self::new(-1.0, 0.0);
    pub const CENTER: Self = Self::new(0.0, 0.0);
    pub const CENTER_RIGHT: Self = Self::new(1.0, 0.0);
    pub const BOTTOM_LEFT: Self = Self::new(-1.0, 1.0);
    pub const BOTTOM_CENTER: Self = Self::new(0.0, 1.0);
    pub const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);

    /// Calculate the offset for a child of `child_size` inside a container of `container_size`
    pub fn compute_offset(&self, container_size: Size, child_size: Size) -> crate::Offset {
        let x = (container_size.width - child_size.width) * (self.x + 1.0) / 2.0;
        let y = (container_size.height - child_size.height) * (self.y + 1.0) / 2.0;
        crate::Offset::new(x, y)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tight_constraints() {
        let size = Size::new(100.0, 50.0);
        let constraints = BoxConstraints::tight(size);
        
        assert!(constraints.is_tight());
        assert_eq!(constraints.constrain(Size::new(200.0, 200.0)), size);
    }

    #[test]
    fn test_loose_constraints() {
        let constraints = BoxConstraints::loose(Size::new(100.0, 100.0));
        
        assert!(!constraints.is_tight());
        assert_eq!(constraints.smallest(), Size::ZERO);
        assert_eq!(constraints.biggest(), Size::new(100.0, 100.0));
    }

    #[test]
    fn test_constrain() {
        let constraints = BoxConstraints::new()
            .with_min_width(50.0)
            .with_max_width(150.0)
            .with_min_height(30.0)
            .with_max_height(100.0);

        // Too small
        assert_eq!(
            constraints.constrain(Size::new(10.0, 10.0)),
            Size::new(50.0, 30.0)
        );

        // Too big
        assert_eq!(
            constraints.constrain(Size::new(200.0, 200.0)),
            Size::new(150.0, 100.0)
        );

        // Just right
        assert_eq!(
            constraints.constrain(Size::new(100.0, 60.0)),
            Size::new(100.0, 60.0)
        );
    }

    #[test]
    fn test_alignment_offset() {
        let container = Size::new(200.0, 200.0);
        let child = Size::new(100.0, 100.0);

        let top_left = Alignment::TOP_LEFT.compute_offset(container, child);
        assert_eq!(top_left.dx, 0.0);
        assert_eq!(top_left.dy, 0.0);

        let center = Alignment::CENTER.compute_offset(container, child);
        assert_eq!(center.dx, 50.0);
        assert_eq!(center.dy, 50.0);

        let bottom_right = Alignment::BOTTOM_RIGHT.compute_offset(container, child);
        assert_eq!(bottom_right.dx, 100.0);
        assert_eq!(bottom_right.dy, 100.0);
    }
}
