//! Transform - 2D affine transformations
//!
//! Provides matrix-based transformations for translate, rotate, scale, skew.

use venom_core::{Point, Offset};

// ============================================================================
// TRANSFORM (2D Affine Matrix)
// ============================================================================

/// 2D affine transformation matrix
/// 
/// Stored as a 3x3 matrix in row-major order:
/// ```text
/// | m00  m01  m02 |   | scaleX  skewX   translateX |
/// | m10  m11  m12 | = | skewY   scaleY  translateY |
/// | 0    0    1   |   | 0       0       1          |
/// ```
/// 
/// # Example
/// ```
/// use venom_render::Transform;
/// 
/// let transform = Transform::identity()
///     .translate(100.0, 50.0)
///     .rotate(45.0)
///     .scale(2.0, 2.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Scale X and rotation component
    pub m00: f32,
    /// Skew X component
    pub m01: f32,
    /// Translate X
    pub m02: f32,
    /// Skew Y component
    pub m10: f32,
    /// Scale Y and rotation component
    pub m11: f32,
    /// Translate Y
    pub m12: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// Create a new transform with given matrix values
    pub const fn new(m00: f32, m01: f32, m02: f32, m10: f32, m11: f32, m12: f32) -> Self {
        Self { m00, m01, m02, m10, m11, m12 }
    }

    /// Create an identity transform (no transformation)
    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0)
    }

    // ========================================================================
    // FACTORY METHODS
    // ========================================================================

    /// Create a translation transform
    pub const fn translation(dx: f32, dy: f32) -> Self {
        Self::new(1.0, 0.0, dx, 0.0, 1.0, dy)
    }

    /// Create a scaling transform
    pub const fn scaling(sx: f32, sy: f32) -> Self {
        Self::new(sx, 0.0, 0.0, 0.0, sy, 0.0)
    }

    /// Create a uniform scaling transform
    pub const fn uniform_scale(s: f32) -> Self {
        Self::scaling(s, s)
    }

    /// Create a rotation transform (degrees)
    pub fn rotation(degrees: f32) -> Self {
        let radians = degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        Self::new(cos, -sin, 0.0, sin, cos, 0.0)
    }

    /// Create a rotation transform around a point (degrees)
    pub fn rotation_around(degrees: f32, cx: f32, cy: f32) -> Self {
        Self::translation(cx, cy)
            .then(&Self::rotation(degrees))
            .then(&Self::translation(-cx, -cy))
    }

    /// Create a skew transform
    pub fn skew(sx: f32, sy: f32) -> Self {
        Self::new(1.0, sx.tan(), 0.0, sy.tan(), 1.0, 0.0)
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Apply a translation
    pub fn translate(self, dx: f32, dy: f32) -> Self {
        self.then(&Self::translation(dx, dy))
    }

    /// Apply a scale
    pub fn scale(self, sx: f32, sy: f32) -> Self {
        self.then(&Self::scaling(sx, sy))
    }

    /// Apply a uniform scale
    pub fn scale_uniform(self, s: f32) -> Self {
        self.then(&Self::uniform_scale(s))
    }

    /// Apply a rotation (degrees)
    pub fn rotate(self, degrees: f32) -> Self {
        self.then(&Self::rotation(degrees))
    }

    /// Apply a rotation around a point (degrees)
    pub fn rotate_around(self, degrees: f32, cx: f32, cy: f32) -> Self {
        self.then(&Self::rotation_around(degrees, cx, cy))
    }

    // ========================================================================
    // MATRIX OPERATIONS
    // ========================================================================

    /// Concatenate with another transform (this × other)
    pub fn then(&self, other: &Transform) -> Self {
        Self {
            m00: self.m00 * other.m00 + self.m01 * other.m10,
            m01: self.m00 * other.m01 + self.m01 * other.m11,
            m02: self.m00 * other.m02 + self.m01 * other.m12 + self.m02,
            m10: self.m10 * other.m00 + self.m11 * other.m10,
            m11: self.m10 * other.m01 + self.m11 * other.m11,
            m12: self.m10 * other.m02 + self.m11 * other.m12 + self.m12,
        }
    }

    /// Calculate the determinant
    pub fn determinant(&self) -> f32 {
        self.m00 * self.m11 - self.m01 * self.m10
    }

    /// Check if the transform is invertible
    pub fn is_invertible(&self) -> bool {
        self.determinant().abs() > f32::EPSILON
    }

    /// Calculate the inverse transform
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() <= f32::EPSILON {
            return None;
        }

        let inv_det = 1.0 / det;
        Some(Self {
            m00: self.m11 * inv_det,
            m01: -self.m01 * inv_det,
            m02: (self.m01 * self.m12 - self.m02 * self.m11) * inv_det,
            m10: -self.m10 * inv_det,
            m11: self.m00 * inv_det,
            m12: (self.m02 * self.m10 - self.m00 * self.m12) * inv_det,
        })
    }

    // ========================================================================
    // POINT TRANSFORMATION
    // ========================================================================

    /// Transform a point
    pub fn transform_point(&self, point: Point) -> Point {
        Point::new(
            self.m00 * point.x + self.m01 * point.y + self.m02,
            self.m10 * point.x + self.m11 * point.y + self.m12,
        )
    }

    /// Transform an offset (ignores translation)
    pub fn transform_offset(&self, offset: Offset) -> Offset {
        Offset::new(
            self.m00 * offset.dx + self.m01 * offset.dy,
            self.m10 * offset.dx + self.m11 * offset.dy,
        )
    }

    /// Transform multiple points
    pub fn transform_points(&self, points: &mut [Point]) {
        for point in points {
            *point = self.transform_point(*point);
        }
    }

    // ========================================================================
    // QUERY METHODS
    // ========================================================================

    /// Check if this is the identity transform
    pub fn is_identity(&self) -> bool {
        (self.m00 - 1.0).abs() < f32::EPSILON
            && self.m01.abs() < f32::EPSILON
            && self.m02.abs() < f32::EPSILON
            && self.m10.abs() < f32::EPSILON
            && (self.m11 - 1.0).abs() < f32::EPSILON
            && self.m12.abs() < f32::EPSILON
    }

    /// Check if this transform only contains translation
    pub fn is_translation_only(&self) -> bool {
        (self.m00 - 1.0).abs() < f32::EPSILON
            && self.m01.abs() < f32::EPSILON
            && self.m10.abs() < f32::EPSILON
            && (self.m11 - 1.0).abs() < f32::EPSILON
    }

    /// Get the translation component
    pub fn translation_component(&self) -> Offset {
        Offset::new(self.m02, self.m12)
    }
}

// ============================================================================
// TRANSFORM STACK (for Canvas)
// ============================================================================

/// A stack of transforms for save/restore operations
#[derive(Debug, Clone, Default)]
pub struct TransformStack {
    stack: Vec<Transform>,
    current: Transform,
}

impl TransformStack {
    /// Create a new transform stack
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Transform::identity(),
        }
    }

    /// Get the current transform
    pub fn current(&self) -> &Transform {
        &self.current
    }

    /// Push the current transform onto the stack (save)
    pub fn save(&mut self) {
        self.stack.push(self.current);
    }

    /// Pop the last saved transform (restore)
    pub fn restore(&mut self) {
        if let Some(transform) = self.stack.pop() {
            self.current = transform;
        }
    }

    /// Apply a transform to the current
    pub fn concat(&mut self, transform: &Transform) {
        self.current = self.current.then(transform);
    }

    /// Translate
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.concat(&Transform::translation(dx, dy));
    }

    /// Scale
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.concat(&Transform::scaling(sx, sy));
    }

    /// Rotate (degrees)
    pub fn rotate(&mut self, degrees: f32) {
        self.concat(&Transform::rotation(degrees));
    }

    /// Reset to identity
    pub fn reset(&mut self) {
        self.stack.clear();
        self.current = Transform::identity();
    }

    /// Get the stack depth
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        assert!(t.is_identity());
        
        let point = Point::new(10.0, 20.0);
        let transformed = t.transform_point(point);
        assert_eq!(transformed, point);
    }

    #[test]
    fn test_translation() {
        let t = Transform::translation(10.0, 20.0);
        let point = Point::new(5.0, 5.0);
        let transformed = t.transform_point(point);
        assert_eq!(transformed, Point::new(15.0, 25.0));
    }

    #[test]
    fn test_scale() {
        let t = Transform::scaling(2.0, 3.0);
        let point = Point::new(10.0, 10.0);
        let transformed = t.transform_point(point);
        assert_eq!(transformed, Point::new(20.0, 30.0));
    }

    #[test]
    fn test_rotation() {
        let t = Transform::rotation(90.0);
        let point = Point::new(1.0, 0.0);
        let transformed = t.transform_point(point);
        // Should be approximately (0, 1)
        assert!((transformed.x - 0.0).abs() < 0.0001);
        assert!((transformed.y - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_inverse() {
        let t = Transform::identity()
            .translate(10.0, 20.0)
            .scale(2.0, 2.0);
        
        let inv = t.inverse().unwrap();
        let result = t.then(&inv);
        assert!(result.is_identity());
    }

    #[test]
    fn test_transform_stack() {
        let mut stack = TransformStack::new();
        
        stack.translate(10.0, 0.0);
        stack.save();
        stack.translate(5.0, 0.0);
        
        let point = stack.current().transform_point(Point::ZERO);
        assert_eq!(point.x, 15.0);
        
        stack.restore();
        let point = stack.current().transform_point(Point::ZERO);
        assert_eq!(point.x, 10.0);
    }
}
