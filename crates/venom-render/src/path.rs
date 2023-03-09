//! Path - Vector path for complex shapes
//!
//! Paths allow drawing arbitrary shapes using lines, curves, and arcs.
//! Based on your C framework's VenomPath API.

use venom_core::{Point, Rect};

// ============================================================================
// PATH COMMAND
// ============================================================================

/// A single command in a path
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    /// Move to a new point (start new contour)
    MoveTo { x: f32, y: f32 },
    /// Draw a line to the given point
    LineTo { x: f32, y: f32 },
    /// Draw a quadratic bezier curve
    QuadTo { cx: f32, cy: f32, x: f32, y: f32 },
    /// Draw a cubic bezier curve
    CubicTo { c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32 },
    /// Close the current contour
    Close,
}

// ============================================================================
// FILL RULE
// ============================================================================

/// Rule for determining which areas are inside a path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillRule {
    /// Non-zero winding rule (most common)
    #[default]
    NonZero,
    /// Even-odd rule
    EvenOdd,
}

// ============================================================================
// PATH
// ============================================================================

/// A vector path for drawing complex shapes
/// 
/// # Example
/// ```
/// use venom_render::Path;
/// 
/// // Draw a triangle
/// let triangle = Path::new()
///     .move_to(50.0, 0.0)
///     .line_to(100.0, 100.0)
///     .line_to(0.0, 100.0)
///     .close();
/// 
/// // Draw a rounded rectangle
/// let rounded = Path::rounded_rect(0.0, 0.0, 100.0, 50.0, 8.0);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Path {
    commands: Vec<PathCommand>,
    fill_rule: FillRule,
}

impl Path {
    /// Create a new empty path
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // PATH BUILDING (BUILDER PATTERN)
    // ========================================================================

    /// Move to a new point (starts a new contour)
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::MoveTo { x, y });
        self
    }

    /// Draw a line to the given point
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::LineTo { x, y });
        self
    }

    /// Draw a quadratic bezier curve
    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::QuadTo { cx, cy, x, y });
        self
    }

    /// Draw a cubic bezier curve
    pub fn cubic_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        self.commands.push(PathCommand::CubicTo { c1x, c1y, c2x, c2y, x, y });
        self
    }

    /// Close the current contour
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    /// Set the fill rule
    pub fn fill_rule(mut self, rule: FillRule) -> Self {
        self.fill_rule = rule;
        self
    }

    // ========================================================================
    // MUTABLE BUILDING
    // ========================================================================

    /// Move to a new point (mutable version)
    pub fn move_to_mut(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::MoveTo { x, y });
    }

    /// Draw a line to the given point (mutable version)
    pub fn line_to_mut(&mut self, x: f32, y: f32) {
        self.commands.push(PathCommand::LineTo { x, y });
    }

    /// Draw a quadratic bezier curve (mutable version)
    pub fn quad_to_mut(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::QuadTo { cx, cy, x, y });
    }

    /// Draw a cubic bezier curve (mutable version)
    pub fn cubic_to_mut(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.commands.push(PathCommand::CubicTo { c1x, c1y, c2x, c2y, x, y });
    }

    /// Close the current contour (mutable version)
    pub fn close_mut(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    // ========================================================================
    // FACTORY METHODS (COMMON SHAPES)
    // ========================================================================

    /// Create a rectangular path
    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new()
            .move_to(x, y)
            .line_to(x + width, y)
            .line_to(x + width, y + height)
            .line_to(x, y + height)
            .close()
    }

    /// Create a rect from a Rect
    pub fn from_rect(rect: Rect) -> Self {
        Self::rect(rect.x, rect.y, rect.width, rect.height)
    }

    /// Create a rounded rectangular path
    pub fn rounded_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        let r = radius.min(width / 2.0).min(height / 2.0);
        
        if r <= 0.0 {
            return Self::rect(x, y, width, height);
        }

        // Control point factor for approximating circular arcs
        // Using the approximation: c = r * (4/3) * tan(π/8) ≈ r * 0.5523
        let c = r * 0.5523;

        Self::new()
            // Start at top-left after the radius
            .move_to(x + r, y)
            // Top edge
            .line_to(x + width - r, y)
            // Top-right corner
            .cubic_to(x + width - r + c, y, x + width, y + r - c, x + width, y + r)
            // Right edge
            .line_to(x + width, y + height - r)
            // Bottom-right corner
            .cubic_to(x + width, y + height - r + c, x + width - r + c, y + height, x + width - r, y + height)
            // Bottom edge
            .line_to(x + r, y + height)
            // Bottom-left corner
            .cubic_to(x + r - c, y + height, x, y + height - r + c, x, y + height - r)
            // Left edge
            .line_to(x, y + r)
            // Top-left corner
            .cubic_to(x, y + r - c, x + r - c, y, x + r, y)
            .close()
    }

    /// Create a circular path
    pub fn circle(cx: f32, cy: f32, radius: f32) -> Self {
        Self::oval(cx - radius, cy - radius, radius * 2.0, radius * 2.0)
    }

    /// Create an oval/ellipse path
    pub fn oval(x: f32, y: f32, width: f32, height: f32) -> Self {
        let rx = width / 2.0;
        let ry = height / 2.0;
        let cx = x + rx;
        let cy = y + ry;

        // Control point factor
        let kx = rx * 0.5523;
        let ky = ry * 0.5523;

        Self::new()
            // Start at top
            .move_to(cx, y)
            // Top-right quadrant
            .cubic_to(cx + kx, y, x + width, cy - ky, x + width, cy)
            // Bottom-right quadrant
            .cubic_to(x + width, cy + ky, cx + kx, y + height, cx, y + height)
            // Bottom-left quadrant
            .cubic_to(cx - kx, y + height, x, cy + ky, x, cy)
            // Top-left quadrant
            .cubic_to(x, cy - ky, cx - kx, y, cx, y)
            .close()
    }

    // ========================================================================
    // ACCESSORS
    // ========================================================================

    /// Get the path commands
    pub fn commands(&self) -> &[PathCommand] {
        &self.commands
    }

    /// Get the fill rule
    pub fn get_fill_rule(&self) -> FillRule {
        self.fill_rule
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Reset the path to empty
    pub fn reset(&mut self) {
        self.commands.clear();
    }

    /// Calculate the bounding box of the path
    pub fn bounds(&self) -> Option<Rect> {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for cmd in &self.commands {
            let (x, y) = match *cmd {
                PathCommand::MoveTo { x, y } => (x, y),
                PathCommand::LineTo { x, y } => (x, y),
                PathCommand::QuadTo { x, y, .. } => (x, y),
                PathCommand::CubicTo { x, y, .. } => (x, y),
                PathCommand::Close => continue,
            };

            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }

        if min_x.is_finite() && min_y.is_finite() && max_x.is_finite() && max_y.is_finite() {
            Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_triangle() {
        let path = Path::new()
            .move_to(50.0, 0.0)
            .line_to(100.0, 100.0)
            .line_to(0.0, 100.0)
            .close();

        assert_eq!(path.commands().len(), 4);
    }

    #[test]
    fn test_path_rect() {
        let path = Path::rect(0.0, 0.0, 100.0, 50.0);
        assert!(!path.is_empty());
        
        let bounds = path.bounds().unwrap();
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 50.0);
    }

    #[test]
    fn test_path_circle() {
        let path = Path::circle(50.0, 50.0, 25.0);
        let bounds = path.bounds().unwrap();
        
        // Should be approximately 50x50
        assert!((bounds.width - 50.0).abs() < 1.0);
        assert!((bounds.height - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_path_rounded_rect() {
        let path = Path::rounded_rect(0.0, 0.0, 100.0, 50.0, 10.0);
        assert!(!path.is_empty());
    }
}
