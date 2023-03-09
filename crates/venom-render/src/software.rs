//! Software renderer backend using tiny-skia
//!
//! This module provides a CPU-based software renderer using the tiny-skia library.
//! It implements the `CanvasBackend` trait for real 2D rendering.
//!
//! # Features
//!
//! - Full 2D rendering (shapes, paths, gradients)
//! - Anti-aliased rendering
//! - Alpha blending
//! - High-quality output
//!
//! # Example
//!
//! ```ignore
//! use venom_render::{SoftwareBackend, Canvas, Paint, Color};
//!
//! // Create a 800x600 pixel buffer
//! let backend = SoftwareBackend::new(800, 600);
//! let mut canvas = Canvas::new(backend);
//!
//! // Draw something
//! canvas.clear(Color::BLACK);
//! canvas.fill_rect(Rect::new(10.0, 10.0, 100.0, 50.0), Color::RED);
//!
//! // Get the pixel data
//! let pixels = canvas.backend().pixels();
//! ```

use venom_core::{Color, Rect, Point, Size, BorderRadius};
use crate::{Paint, Path, Transform, PaintCanvas, CanvasBackend, PaintStyle};

// Re-export tiny-skia types for advanced usage
pub use tiny_skia;

// ============================================================================
// SOFTWARE BACKEND
// ============================================================================

/// Software rendering backend using tiny-skia
/// 
/// This backend renders to an in-memory pixel buffer using the CPU.
/// It's portable and works on any platform, making it ideal for:
/// - Testing and development
/// - Headless rendering
/// - Image generation
/// - Platforms without GPU support
pub struct SoftwareBackend {
    /// The pixel buffer
    pixmap: tiny_skia::Pixmap,
    /// Current transform
    transform: tiny_skia::Transform,
    /// Current clip rect
    clip: Option<Rect>,
    /// Transform stack for save/restore
    transform_stack: Vec<tiny_skia::Transform>,
    /// Clip stack for save/restore
    clip_stack: Vec<Option<Rect>>,
    /// Text rendering pipeline
    #[cfg(feature = "text")]
    text_pipeline: Option<crate::text_pipeline::TextPipeline>,
}

impl SoftwareBackend {
    /// Create a new software backend with the given dimensions
    /// 
    /// # Arguments
    /// 
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// 
    /// # Panics
    /// 
    /// Panics if width or height is 0 or if allocation fails.
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = tiny_skia::Pixmap::new(width, height)
            .expect("Failed to create pixmap");
        
        Self {
            pixmap,
            transform: tiny_skia::Transform::identity(),
            clip: None,
            transform_stack: Vec::new(),
            clip_stack: Vec::new(),
            #[cfg(feature = "text")]
            text_pipeline: Some(crate::text_pipeline::TextPipeline::new()),
        }
    }

    /// Create from an existing pixmap
    pub fn from_pixmap(pixmap: tiny_skia::Pixmap) -> Self {
        Self {
            pixmap,
            transform: tiny_skia::Transform::identity(),
            clip: None,
            transform_stack: Vec::new(),
            clip_stack: Vec::new(),
            #[cfg(feature = "text")]
            text_pipeline: Some(crate::text_pipeline::TextPipeline::new()),
        }
    }

    // ========================================================================
    // PIXEL ACCESS
    // ========================================================================

    /// Get the raw pixel data (RGBA, premultiplied alpha)
    pub fn pixels(&self) -> &[u8] {
        self.pixmap.data()
    }

    /// Get mutable pixel data
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        self.pixmap.data_mut()
    }

    /// Get the width in pixels
    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    /// Get the height in pixels
    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    /// Get the underlying pixmap
    pub fn pixmap(&self) -> &tiny_skia::Pixmap {
        &self.pixmap
    }

    /// Get mutable pixmap
    pub fn pixmap_mut(&mut self) -> &mut tiny_skia::Pixmap {
        &mut self.pixmap
    }

    // ========================================================================
    // SAVE/ENCODE
    // ========================================================================

    /// Encode as PNG bytes
    pub fn encode_png(&self) -> Option<Vec<u8>> {
        self.pixmap.encode_png().ok()
    }

    /// Save to a PNG file
    pub fn save_png(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let data = self.encode_png()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to encode PNG"
            ))?;
        std::fs::write(path, data)
    }

    // ========================================================================
    // INTERNAL HELPERS
    // ========================================================================

    /// Convert our Color to tiny_skia Color
    fn to_skia_color(color: Color) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a)
    }

    /// Convert our Paint to tiny_skia Paint
    fn to_skia_paint<'a>(paint: &Paint) -> tiny_skia::Paint<'a> {
        let mut skia_paint = tiny_skia::Paint::default();
        skia_paint.set_color(Self::to_skia_color(paint.color));
        skia_paint.anti_alias = true;
        skia_paint
    }

    /// Convert our Rect to tiny_skia Rect
    fn to_skia_rect(rect: Rect) -> Option<tiny_skia::Rect> {
        tiny_skia::Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
    }

    /// Create a fill path from a rect
    fn rect_to_path(rect: Rect) -> Option<tiny_skia::Path> {
        let skia_rect = Self::to_skia_rect(rect)?;
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_rect(skia_rect);
        pb.finish()
    }

    /// Create a rounded rect path
    fn rounded_rect_to_path(rect: Rect, radius: BorderRadius) -> Option<tiny_skia::Path> {
        let mut pb = tiny_skia::PathBuilder::new();
        
        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;
        
        // Clamp radii to half of dimensions
        let tl = radius.top_left.min(w / 2.0).min(h / 2.0);
        let tr = radius.top_right.min(w / 2.0).min(h / 2.0);
        let br = radius.bottom_right.min(w / 2.0).min(h / 2.0);
        let bl = radius.bottom_left.min(w / 2.0).min(h / 2.0);
        
        // Start at top-left after the corner
        pb.move_to(x + tl, y);
        
        // Top edge and top-right corner
        pb.line_to(x + w - tr, y);
        if tr > 0.0 {
            pb.quad_to(x + w, y, x + w, y + tr);
        }
        
        // Right edge and bottom-right corner
        pb.line_to(x + w, y + h - br);
        if br > 0.0 {
            pb.quad_to(x + w, y + h, x + w - br, y + h);
        }
        
        // Bottom edge and bottom-left corner
        pb.line_to(x + bl, y + h);
        if bl > 0.0 {
            pb.quad_to(x, y + h, x, y + h - bl);
        }
        
        // Left edge and top-left corner
        pb.line_to(x, y + tl);
        if tl > 0.0 {
            pb.quad_to(x, y, x + tl, y);
        }
        
        pb.close();
        pb.finish()
    }

    /// Create a circle path
    fn circle_to_path(center: Point, radius: f32) -> Option<tiny_skia::Path> {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.push_circle(center.x, center.y, radius);
        pb.finish()
    }

    /// Create an oval path
    fn oval_to_path(rect: Rect) -> Option<tiny_skia::Path> {
        let mut pb = tiny_skia::PathBuilder::new();
        let skia_rect = Self::to_skia_rect(rect)?;
        pb.push_oval(skia_rect);
        pb.finish()
    }

    /// Draw a filled path
    fn fill_path(&mut self, path: &tiny_skia::Path, paint: &Paint) {
        let skia_paint = Self::to_skia_paint(paint);
        self.pixmap.fill_path(
            path,
            &skia_paint,
            tiny_skia::FillRule::Winding,
            self.transform,
            None, // TODO: proper clip mask
        );
    }

    /// Draw a stroked path
    fn stroke_path(&mut self, path: &tiny_skia::Path, paint: &Paint) {
        let mut skia_paint = Self::to_skia_paint(paint);
        
        let stroke = tiny_skia::Stroke {
            width: paint.stroke_width,
            line_cap: match paint.stroke_cap {
                crate::StrokeCap::Butt => tiny_skia::LineCap::Butt,
                crate::StrokeCap::Round => tiny_skia::LineCap::Round,
                crate::StrokeCap::Square => tiny_skia::LineCap::Square,
            },
            line_join: match paint.stroke_join {
                crate::StrokeJoin::Miter => tiny_skia::LineJoin::Miter,
                crate::StrokeJoin::Round => tiny_skia::LineJoin::Round,
                crate::StrokeJoin::Bevel => tiny_skia::LineJoin::Bevel,
            },
            miter_limit: paint.miter_limit,
            dash: None,
        };

        self.pixmap.stroke_path(
            path,
            &skia_paint,
            &stroke,
            self.transform,
            None,
        );
    }
}

// ============================================================================
// PAINT CANVAS IMPLEMENTATION
// ============================================================================

impl PaintCanvas for SoftwareBackend {
    fn clear(&mut self, color: Color) {
        self.pixmap.fill(Self::to_skia_color(color));
    }

    fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        if let Some(path) = Self::rect_to_path(rect) {
            match paint.style {
                PaintStyle::Fill => self.fill_path(&path, paint),
                PaintStyle::Stroke => self.stroke_path(&path, paint),
            }
        }
    }

    fn draw_rounded_rect(&mut self, rect: Rect, radius: BorderRadius, paint: &Paint) {
        if let Some(path) = Self::rounded_rect_to_path(rect, radius) {
            match paint.style {
                PaintStyle::Fill => self.fill_path(&path, paint),
                PaintStyle::Stroke => self.stroke_path(&path, paint),
            }
        }
    }

    fn draw_circle(&mut self, center: Point, radius: f32, paint: &Paint) {
        if let Some(path) = Self::circle_to_path(center, radius) {
            match paint.style {
                PaintStyle::Fill => self.fill_path(&path, paint),
                PaintStyle::Stroke => self.stroke_path(&path, paint),
            }
        }
    }

    fn draw_oval(&mut self, rect: Rect, paint: &Paint) {
        if let Some(path) = Self::oval_to_path(rect) {
            match paint.style {
                PaintStyle::Fill => self.fill_path(&path, paint),
                PaintStyle::Stroke => self.stroke_path(&path, paint),
            }
        }
    }

    fn draw_line(&mut self, p1: Point, p2: Point, paint: &Paint) {
        let mut pb = tiny_skia::PathBuilder::new();
        pb.move_to(p1.x, p1.y);
        pb.line_to(p2.x, p2.y);
        if let Some(path) = pb.finish() {
            self.stroke_path(&path, paint);
        }
    }

    fn draw_path(&mut self, path: &Path, paint: &Paint) {
        // Convert our Path to tiny_skia Path
        let mut pb = tiny_skia::PathBuilder::new();
        
        for cmd in path.commands() {
            match cmd {
                crate::PathCommand::MoveTo { x, y } => pb.move_to(*x, *y),
                crate::PathCommand::LineTo { x, y } => pb.line_to(*x, *y),
                crate::PathCommand::QuadTo { cx, cy, x, y } => pb.quad_to(*cx, *cy, *x, *y),
                crate::PathCommand::CubicTo { c1x, c1y, c2x, c2y, x, y } => {
                    pb.cubic_to(*c1x, *c1y, *c2x, *c2y, *x, *y);
                }
                crate::PathCommand::Close => pb.close(),
            }
        }

        if let Some(skia_path) = pb.finish() {
            match paint.style {
                PaintStyle::Fill => self.fill_path(&skia_path, paint),
                PaintStyle::Stroke => self.stroke_path(&skia_path, paint),
            }
        }
    }

    // Draw text using TextPipeline or placeholder
    fn draw_text(&mut self, text: &str, position: Point, paint: &Paint, size: f32) {
        #[cfg(feature = "text")]
        {
            if let Some(pipeline) = &mut self.text_pipeline {
                // Skia pixmap data is reference counted, but we can safely access mutable data 
                // since we have exclusive access to SoftwareBackend
                let width = self.pixmap.width();
                let height = self.pixmap.height();
                let pixels = self.pixmap.data_mut();
                
                // Convert u8 byte slice to u32 slice for pixel manipulation
                // Safety: Pixmap data is always 4-byte aligned (RGBA8888)
                let pixels_u32 = unsafe {
                    std::slice::from_raw_parts_mut(
                        pixels.as_mut_ptr() as *mut u32,
                        pixels.len() / 4
                    )
                };

                pipeline.draw_text(
                    text,
                    position.x,
                    position.y,
                    size,
                    paint.color,
                    pixels_u32,
                    width,
                    height
                );
                return;
            }
        }
        
        // Fallback or if text feature disabled
        let char_width = size * 0.6;
        let mut x = position.x;
        for _ in text.chars() {
            if let Some(rect) = Self::to_skia_rect(Rect::new(x, position.y, char_width * 0.8, size)) {
                let mut pb = tiny_skia::PathBuilder::new();
                pb.push_rect(rect);
                if let Some(path) = pb.finish() {
                    self.fill_path(&path, paint);
                }
            }
            x += char_width;
        }
    }

    fn set_transform(&mut self, transform: &Transform) {
        self.transform = tiny_skia::Transform::from_row(
            transform.m00, transform.m01,
            transform.m10, transform.m11,
            transform.m02, transform.m12,
        );
    }

    fn set_clip_rect(&mut self, rect: Option<Rect>) {
        self.clip = rect;
        // Note: tiny-skia uses mask-based clipping which is more complex
        // For simple rectangular clips, we'd create a mask pixmap
    }

    fn save(&mut self) {
        self.transform_stack.push(self.transform);
        self.clip_stack.push(self.clip);
    }

    fn restore(&mut self) {
        if let Some(transform) = self.transform_stack.pop() {
            self.transform = transform;
        }
        if let Some(clip) = self.clip_stack.pop() {
            self.clip = clip;
        }
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        self.transform = self.transform.pre_translate(dx, dy);
    }
}

// ============================================================================
// CANVAS BACKEND IMPLEMENTATION
// ============================================================================

impl CanvasBackend for SoftwareBackend {
    fn size(&self) -> Size {
        Size::new(self.pixmap.width() as f32, self.pixmap.height() as f32)
    }

    fn flush(&mut self) {
        // Software backend doesn't need flushing - all operations are immediate
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = SoftwareBackend::new(800, 600);
        assert_eq!(backend.width(), 800);
        assert_eq!(backend.height(), 600);
    }

    #[test]
    fn test_clear() {
        let mut backend = SoftwareBackend::new(100, 100);
        backend.clear(Color::RED);
        
        // Check first pixel (RGBA)
        let pixels = backend.pixels();
        // Note: tiny-skia stores as RGBA premultiplied
        assert_eq!(pixels[0], 255); // R
        assert_eq!(pixels[1], 0);   // G
        assert_eq!(pixels[2], 0);   // B
        assert_eq!(pixels[3], 255); // A
    }

    #[test]
    fn test_draw_rect() {
        let mut backend = SoftwareBackend::new(100, 100);
        backend.clear(Color::BLACK);
        
        let rect = Rect::new(10.0, 10.0, 20.0, 20.0);
        backend.draw_rect(rect, &Paint::fill(Color::WHITE));
        
        // Check that something was drawn
        let pixels = backend.pixels();
        // Pixel at (15, 15) should be white
        let idx = (15 * 100 + 15) * 4;
        assert_eq!(pixels[idx], 255);
    }

    #[test]
    fn test_draw_circle() {
        let mut backend = SoftwareBackend::new(100, 100);
        backend.clear(Color::BLACK);
        
        backend.draw_circle(Point::new(50.0, 50.0), 20.0, &Paint::fill(Color::BLUE));
        
        // Check center pixel
        let pixels = backend.pixels();
        let idx = (50 * 100 + 50) * 4;
        assert_eq!(pixels[idx + 2], 255); // Blue channel
    }

    #[test]
    fn test_save_restore() {
        let mut backend = SoftwareBackend::new(100, 100);
        
        backend.translate(10.0, 20.0);
        backend.save();
        backend.translate(5.0, 5.0);
        backend.restore();
        
        // Transform should be back to (10, 20)
        assert_eq!(backend.transform.tx, 10.0);
        assert_eq!(backend.transform.ty, 20.0);
    }

    #[test]
    fn test_size() {
        let backend = SoftwareBackend::new(1920, 1080);
        let size = backend.size();
        assert_eq!(size.width, 1920.0);
        assert_eq!(size.height, 1080.0);
    }
}
