//! Canvas - Abstract rendering surface
//!
//! Canvas provides a backend-agnostic drawing API.
//! Backends implement the `CanvasBackend` trait.
//!
//! Inspired by Flutter's Canvas and your C framework's VenomCanvas.

use venom_core::{Color, Rect, Point, Size, BorderRadius};
use crate::{Paint, Path, Transform, TransformStack};

// ============================================================================
// TEXT ALIGNMENT
// ============================================================================

/// Horizontal text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlign {
    /// Align to the left
    #[default]
    Left,
    /// Align to the center
    Center,
    /// Align to the right
    Right,
}

/// Backend trait for rendering implementations
/// 
/// This is implemented by software renderer, OpenGL, Vulkan, etc.
pub trait CanvasBackend: PaintCanvas {
    /// Get the canvas size
    fn size(&self) -> Size;

    /// Flush pending operations
    fn flush(&mut self);
}

/// Dyn-compatible canvas operations for painting
/// 
/// This trait contains only the drawing methods, making it safe to use
/// with `dyn` for type-erased widget painting.
pub trait PaintCanvas {
    /// Clear the canvas with a color
    fn clear(&mut self, color: Color);

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: Rect, paint: &Paint);

    /// Draw a rounded rectangle
    fn draw_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, paint: &Paint);

    /// Draw a circle
    fn draw_circle(&mut self, center: Point, radius: f32, paint: &Paint);

    /// Draw an oval/ellipse
    fn draw_oval(&mut self, rect: Rect, paint: &Paint);

    /// Draw a line
    fn draw_line(&mut self, p1: Point, p2: Point, paint: &Paint);

    /// Draw a path
    fn draw_path(&mut self, path: &Path, paint: &Paint);

    /// Draw text
    fn draw_text(&mut self, text: &str, position: Point, paint: &Paint, size: f32);

    /// Set the current transform
    fn set_transform(&mut self, transform: &Transform);

    /// Set the clip rectangle
    fn set_clip_rect(&mut self, rect: Option<Rect>);

    /// Save the current state
    fn save(&mut self);

    /// Restore the previous state
    fn restore(&mut self);

    /// Translate
    fn translate(&mut self, dx: f32, dy: f32);
}

// ============================================================================
// CANVAS
// ============================================================================

/// High-level drawing surface
/// 
/// Canvas provides a stateful drawing API with:
/// - Transform stack (save/restore)
/// - Clipping
/// - Convenient drawing methods
/// 
/// # Example
/// ```ignore
/// canvas.save();
/// canvas.translate(100.0, 100.0);
/// canvas.draw_rect(Rect::new(0.0, 0.0, 50.0, 50.0), &Paint::fill(Color::RED));
/// canvas.restore();
/// ```
pub struct Canvas<B: CanvasBackend> {
    backend: B,
    transform_stack: TransformStack,
    clip_stack: Vec<Option<Rect>>,
}

impl<B: CanvasBackend> Canvas<B> {
    /// Create a new canvas with the given backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            transform_stack: TransformStack::new(),
            clip_stack: Vec::new(),
        }
    }

    /// Get a reference to the backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Get a mutable reference to the backend
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Get the canvas size
    pub fn size(&self) -> Size {
        self.backend.size()
    }

    // ========================================================================
    // STATE MANAGEMENT
    // ========================================================================

    /// Save the current transform and clip state
    pub fn save(&mut self) {
        self.transform_stack.save();
        self.clip_stack.push(None); // TODO: proper clip saving
    }

    /// Restore the previous transform and clip state
    pub fn restore(&mut self) {
        self.transform_stack.restore();
        self.clip_stack.pop();
        self.backend.set_transform(self.transform_stack.current());
    }

    /// Get the current transform
    pub fn current_transform(&self) -> &Transform {
        self.transform_stack.current()
    }

    // ========================================================================
    // TRANSFORMS
    // ========================================================================

    /// Translate the canvas
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.transform_stack.translate(dx, dy);
        self.backend.set_transform(self.transform_stack.current());
    }

    /// Scale the canvas
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.transform_stack.scale(sx, sy);
        self.backend.set_transform(self.transform_stack.current());
    }

    /// Rotate the canvas (degrees)
    pub fn rotate(&mut self, degrees: f32) {
        self.transform_stack.rotate(degrees);
        self.backend.set_transform(self.transform_stack.current());
    }

    /// Apply a custom transform
    pub fn concat(&mut self, transform: &Transform) {
        self.transform_stack.concat(transform);
        self.backend.set_transform(self.transform_stack.current());
    }

    // ========================================================================
    // CLIPPING
    // ========================================================================

    /// Set a clip rectangle
    pub fn clip_rect(&mut self, rect: Rect) {
        // Transform the clip rect
        let transformed = Rect::from_points(
            self.transform_stack.current().transform_point(rect.top_left()),
            self.transform_stack.current().transform_point(rect.bottom_right()),
        );
        self.backend.set_clip_rect(Some(transformed));
    }

    /// Clear the clip
    pub fn clear_clip(&mut self) {
        self.backend.set_clip_rect(None);
    }

    // ========================================================================
    // DRAWING METHODS
    // ========================================================================

    /// Clear the canvas with a solid color
    pub fn clear(&mut self, color: Color) {
        self.backend.clear(color);
    }

    /// Draw a rectangle
    pub fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        self.backend.draw_rect(rect, paint);
    }

    /// Draw a rounded rectangle
    pub fn draw_rounded_rect(&mut self, rect: Rect, radius: f32, paint: &Paint) {
        self.backend.draw_rounded_rect(rect, BorderRadius::all(radius), paint);
    }

    /// Draw a rounded rectangle with custom radii
    pub fn draw_rounded_rect_with_radii(&mut self, rect: Rect, radii: BorderRadius, paint: &Paint) {
        self.backend.draw_rounded_rect(rect, radii, paint);
    }

    /// Draw a circle
    pub fn draw_circle(&mut self, center: Point, radius: f32, paint: &Paint) {
        self.backend.draw_circle(center, radius, paint);
    }

    /// Draw an oval/ellipse
    pub fn draw_oval(&mut self, rect: Rect, paint: &Paint) {
        self.backend.draw_oval(rect, paint);
    }

    /// Draw a line
    pub fn draw_line(&mut self, p1: Point, p2: Point, paint: &Paint) {
        self.backend.draw_line(p1, p2, paint);
    }

    /// Draw a path
    pub fn draw_path(&mut self, path: &Path, paint: &Paint) {
        self.backend.draw_path(path, paint);
    }

    /// Draw text
    pub fn draw_text(&mut self, text: &str, position: Point, paint: &Paint, size: f32) {
        self.backend.draw_text(text, position, paint, size);
    }

    // ========================================================================
    // CONVENIENCE METHODS
    // ========================================================================

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.draw_rect(rect, &Paint::fill(color));
    }

    /// Draw a stroked rectangle
    pub fn stroke_rect(&mut self, rect: Rect, color: Color, width: f32) {
        self.draw_rect(rect, &Paint::stroke(color, width));
    }

    /// Draw a filled circle
    pub fn fill_circle(&mut self, center: Point, radius: f32, color: Color) {
        self.draw_circle(center, radius, &Paint::fill(color));
    }

    /// Draw a stroked circle
    pub fn stroke_circle(&mut self, center: Point, radius: f32, color: Color, width: f32) {
        self.draw_circle(center, radius, &Paint::stroke(color, width));
    }

    /// Flush pending operations
    pub fn flush(&mut self) {
        self.backend.flush();
    }

    /// Get as a dyn PaintCanvas for widget painting
    pub fn as_paint_canvas(&mut self) -> &mut dyn PaintCanvas {
        self
    }
}

// Implement PaintCanvas for Canvas
impl<B: CanvasBackend> PaintCanvas for Canvas<B> {
    fn clear(&mut self, color: Color) {
        self.backend.clear(color);
    }

    fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        self.backend.draw_rect(rect, paint);
    }

    fn draw_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, paint: &Paint) {
        self.backend.draw_rounded_rect(rect, radius, paint);
    }

    fn draw_circle(&mut self, center: Point, radius: f32, paint: &Paint) {
        self.backend.draw_circle(center, radius, paint);
    }

    fn draw_oval(&mut self, rect: Rect, paint: &Paint) {
        self.backend.draw_oval(rect, paint);
    }

    fn draw_line(&mut self, p1: Point, p2: Point, paint: &Paint) {
        self.backend.draw_line(p1, p2, paint);
    }

    fn draw_path(&mut self, path: &Path, paint: &Paint) {
        self.backend.draw_path(path, paint);
    }

    fn draw_text(&mut self, text: &str, position: Point, paint: &Paint, size: f32) {
        self.backend.draw_text(text, position, paint, size);
    }

    fn set_transform(&mut self, transform: &Transform) {
        self.backend.set_transform(transform);
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) {
        self.backend.set_clip_rect(rect);
    }

    fn save(&mut self) {
        self.transform_stack.save();
        self.clip_stack.push(None);
    }

    fn restore(&mut self) {
        self.transform_stack.restore();
        self.clip_stack.pop();
        self.backend.set_transform(self.transform_stack.current());
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        self.transform_stack.translate(dx, dy);
        self.backend.set_transform(self.transform_stack.current());
    }
}

// ============================================================================
// NULL BACKEND (for testing)
// ============================================================================

/// A no-op backend for testing
#[derive(Debug, Clone, Default)]
pub struct NullBackend {
    size: Size,
    draw_count: usize,
}

impl NullBackend {
    /// Create a new null backend with the given size
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            size: Size::new(width, height),
            draw_count: 0,
        }
    }

    /// Get the number of draw calls
    pub fn draw_count(&self) -> usize {
        self.draw_count
    }
}

impl PaintCanvas for NullBackend {
    fn clear(&mut self, _color: Color) {
        self.draw_count += 1;
    }

    fn draw_rect(&mut self, _rect: Rect, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_rounded_rect(&mut self, _rect: Rect, _radius: BorderRadius, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_circle(&mut self, _center: Point, _radius: f32, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_oval(&mut self, _rect: Rect, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_line(&mut self, _p1: Point, _p2: Point, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_path(&mut self, _path: &Path, _paint: &Paint) {
        self.draw_count += 1;
    }

    fn draw_text(&mut self, _text: &str, _position: Point, _paint: &Paint, _size: f32) {
        self.draw_count += 1;
    }

    fn set_transform(&mut self, _transform: &Transform) {}

    fn set_clip_rect(&mut self, _rect: Option<Rect>) {}

    fn save(&mut self) {}

    fn restore(&mut self) {}

    fn translate(&mut self, _dx: f32, _dy: f32) {}
}

impl CanvasBackend for NullBackend {
    fn size(&self) -> Size {
        self.size
    }

    fn flush(&mut self) {}
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_backend() {
        let backend = NullBackend::new(800.0, 600.0);
        let mut canvas = Canvas::new(backend);

        canvas.clear(Color::BLACK);
        canvas.fill_rect(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
        canvas.fill_circle(Point::new(50.0, 50.0), 25.0, Color::BLUE);

        assert_eq!(canvas.backend().draw_count(), 3);
    }

    #[test]
    fn test_canvas_save_restore() {
        let backend = NullBackend::new(800.0, 600.0);
        let mut canvas = Canvas::new(backend);

        canvas.translate(100.0, 100.0);
        let t1 = *canvas.current_transform();

        canvas.save();
        canvas.translate(50.0, 50.0);
        let t2 = *canvas.current_transform();
        assert_ne!(t1, t2);

        canvas.restore();
        let t3 = *canvas.current_transform();
        assert_eq!(t1, t3);
    }

    #[test]
    fn test_canvas_size() {
        let backend = NullBackend::new(1920.0, 1080.0);
        let canvas = Canvas::new(backend);
        
        let size = canvas.size();
        assert_eq!(size.width, 1920.0);
        assert_eq!(size.height, 1080.0);
    }
}
