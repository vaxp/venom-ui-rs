//! Text rendering pipeline using cosmic-text
//!
//! This module provides text shaping, layout, and rendering using cosmic-text.
//!
//! # Architecture
//! 
//! - `FontSystem`: Manages font loading and lookup
//! - `Buffer`: Lays out text into lines and glyphs
//! - `GlyphCache`: Caches rendered glyph bitmaps
//! - `TextPipeline`: Main entry point for text rendering

#[cfg(feature = "text")]
use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::OnceLock;
use venom_core::Color;

// ============================================================================
// FONT SYSTEM SINGLETON
// ============================================================================

/// Global font system (expensive to create, so we share it)
static FONT_SYSTEM: OnceLock<std::sync::Mutex<FontSystem>> = OnceLock::new();

/// Get or initialize the global font system
#[cfg(feature = "text")]
pub fn font_system() -> &'static std::sync::Mutex<FontSystem> {
    FONT_SYSTEM.get_or_init(|| {
        let mut font_system = FontSystem::new();
        // Load system fonts
        font_system.db_mut().load_system_fonts();
        std::sync::Mutex::new(font_system)
    })
}

// ============================================================================
// GLYPH CACHE
// ============================================================================

/// Cache for rendered glyphs to avoid re-rasterizing
#[derive(Debug, Default)]
pub struct GlyphCache {
    /// Map from (cache_key, color_rgb) to (pixel buffer, placement)
    entries: FxHashMap<(cosmic_text::CacheKey, [u8; 3]), GlyphEntry>,
    /// Recently used keys for LRU eviction
    recently_used: FxHashSet<(cosmic_text::CacheKey, [u8; 3])>,
    /// Trim counter
    trim_count: usize,
}

/// A cached glyph entry
#[derive(Debug, Clone)]
pub struct GlyphEntry {
    /// RGBA pixel buffer
    pub buffer: Vec<u32>,
    /// Glyph placement info
    pub width: u32,
    /// Glyph height
    pub height: u32,
    /// X offset
    pub left: i32,
    /// Y offset
    pub top: i32,
}

#[cfg(feature = "text")]
impl GlyphCache {
    const TRIM_INTERVAL: usize = 300;
    const CAPACITY_LIMIT: usize = 8 * 1024;

    /// Create a new glyph cache
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or allocate a glyph in the cache
    pub fn get_or_insert(
        &mut self,
        cache_key: cosmic_text::CacheKey,
        color: Color,
        font_system: &mut FontSystem,
        swash: &mut SwashCache,
    ) -> Option<&GlyphEntry> {
        let [r, g, b, _] = [color.r, color.g, color.b, color.a];
        let key = (cache_key, [r, g, b]);

        // Check if we need to rasterize
        if !self.entries.contains_key(&key) {
            // Rasterize the glyph
            let image = swash.get_image_uncached(font_system, cache_key)?;
            
            let glyph_w = image.placement.width as usize;
            let glyph_h = image.placement.height as usize;
            let glyph_size = glyph_w * glyph_h;

            if glyph_size == 0 {
                return None;
            }

            let mut buffer = vec![0u32; glyph_size];

            match image.content {
                cosmic_text::SwashContent::Mask => {
                    // Grayscale mask - apply color
                    for (i, alpha) in image.data.iter().enumerate() {
                        if i < buffer.len() {
                            // ARGB format for tiny-skia
                            buffer[i] = u32::from_be_bytes([*alpha, r, g, b]);
                        }
                    }
                }
                cosmic_text::SwashContent::Color => {
                    // Color glyph (emoji)
                    for i in 0..glyph_size {
                        let offset = i * 4;
                        if offset + 3 < image.data.len() {
                            let pr = image.data[offset];
                            let pg = image.data[offset + 1];
                            let pb = image.data[offset + 2];
                            let pa = image.data[offset + 3];
                            buffer[i] = u32::from_be_bytes([pa, pr, pg, pb]);
                        }
                    }
                }
                cosmic_text::SwashContent::SubpixelMask => {
                    // Subpixel - treat as grayscale for now
                    for i in 0..glyph_size {
                        let offset = i * 3;
                        if offset + 2 < image.data.len() {
                            let avg = ((image.data[offset] as u16 
                                + image.data[offset + 1] as u16 
                                + image.data[offset + 2] as u16) / 3) as u8;
                            buffer[i] = u32::from_be_bytes([avg, r, g, b]);
                        }
                    }
                }
            }

            let entry = GlyphEntry {
                buffer,
                width: image.placement.width,
                height: image.placement.height,
                left: image.placement.left,
                top: image.placement.top,
            };

            self.entries.insert(key, entry);
        }

        self.recently_used.insert(key);
        self.entries.get(&key)
    }

    /// Trim the cache to prevent unbounded growth
    pub fn trim(&mut self) {
        self.trim_count += 1;
        
        if self.trim_count > Self::TRIM_INTERVAL || self.entries.len() > Self::CAPACITY_LIMIT {
            // Keep only recently used glyphs
            self.entries.retain(|key, _| self.recently_used.contains(key));
            self.recently_used.clear();
            self.entries.shrink_to(Self::CAPACITY_LIMIT);
            self.trim_count = 0;
        }
    }
}

// ============================================================================
// TEXT PIPELINE
// ============================================================================

/// Main text rendering pipeline
#[cfg(feature = "text")]
pub struct TextPipeline {
    /// Glyph cache
    glyph_cache: GlyphCache,
    /// Swash cache for rasterization
    swash_cache: SwashCache,
}

#[cfg(feature = "text")]
impl TextPipeline {
    /// Create a new text pipeline
    pub fn new() -> Self {
        Self {
            glyph_cache: GlyphCache::new(),
            swash_cache: SwashCache::new(),
        }
    }

    /// Render text to a pixel buffer
    /// 
    /// # Arguments
    /// * `text` - The text to render
    /// * `x` - X position
    /// * `y` - Y position  
    /// * `size` - Font size in pixels
    /// * `color` - Text color
    /// * `pixels` - Output pixel buffer (RGBA)
    /// * `width` - Buffer width
    /// * `height` - Buffer height
    pub fn draw_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: Color,
        pixels: &mut [u32],
        width: u32,
        height: u32,
    ) {
        if text.is_empty() {
            return;
        }

        let mut font_system = font_system().lock().expect("Lock font system");

        // Create text buffer with metrics
        let metrics = Metrics::new(size, size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        
        // Set text with default attributes
        let attrs = Attrs::new().family(Family::SansSerif);
        buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);
        
        // Shape the text
        buffer.shape_until_scroll(&mut font_system, false);

        // Render each glyph
        let x_offset = x as i32;
        let y_offset = y as i32;

        for run in buffer.layout_runs() {
            let line_y = run.line_y as i32;
            
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0.0, 0.0), 1.0);
                
                if let Some(entry) = self.glyph_cache.get_or_insert(
                    physical.cache_key,
                    color,
                    &mut font_system,
                    &mut self.swash_cache,
                ) {
                    // Calculate glyph position
                    let glyph_x = x_offset + physical.x + entry.left;
                    let glyph_y = y_offset + line_y + physical.y - entry.top;
                    
                    // Blit glyph to output buffer
                    Self::blit_glyph(
                        entry,
                        glyph_x,
                        glyph_y,
                        color.a,
                        pixels,
                        width,
                        height,
                    );
                }
            }
        }

        // Trim cache periodically
        drop(font_system);
        self.glyph_cache.trim();
    }

    /// Blit a glyph to the output buffer with alpha blending
    fn blit_glyph(
        entry: &GlyphEntry,
        x: i32,
        y: i32,
        opacity: u8,
        pixels: &mut [u32],
        buf_width: u32,
        buf_height: u32,
    ) {
        for gy in 0..entry.height as i32 {
            for gx in 0..entry.width as i32 {
                let px = x + gx;
                let py = y + gy;

                // Bounds check
                if px < 0 || py < 0 || px >= buf_width as i32 || py >= buf_height as i32 {
                    continue;
                }

                let glyph_idx = (gy as u32 * entry.width + gx as u32) as usize;
                let buf_idx = (py as u32 * buf_width + px as u32) as usize;

                if glyph_idx >= entry.buffer.len() || buf_idx >= pixels.len() {
                    continue;
                }

                let src = entry.buffer[glyph_idx];
                let [src_a, src_r, src_g, src_b] = src.to_be_bytes();
                
                // Apply opacity
                let alpha = (src_a as u16 * opacity as u16 / 255) as u8;
                
                if alpha == 0 {
                    continue;
                }

                if alpha == 255 {
                    // Opaque - direct copy
                    pixels[buf_idx] = u32::from_be_bytes([255, src_r, src_g, src_b]);
                } else {
                    // Alpha blend
                    let dst = pixels[buf_idx];
                    let [dst_a, dst_r, dst_g, dst_b] = dst.to_be_bytes();
                    
                    let inv_alpha = 255 - alpha as u16;
                    let out_r = ((src_r as u16 * alpha as u16 + dst_r as u16 * inv_alpha) / 255) as u8;
                    let out_g = ((src_g as u16 * alpha as u16 + dst_g as u16 * inv_alpha) / 255) as u8;
                    let out_b = ((src_b as u16 * alpha as u16 + dst_b as u16 * inv_alpha) / 255) as u8;
                    let out_a = alpha.max(dst_a);
                    
                    pixels[buf_idx] = u32::from_be_bytes([out_a, out_r, out_g, out_b]);
                }
            }
        }
    }

    /// Measure text dimensions
    pub fn measure_text(&mut self, text: &str, size: f32) -> (f32, f32) {
        if text.is_empty() {
            return (0.0, size);
        }

        let mut font_system = font_system().lock().expect("Lock font system");
        
        let metrics = Metrics::new(size, size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        
        let attrs = Attrs::new().family(Family::SansSerif);
        buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for run in buffer.layout_runs() {
            let run_width: f32 = run.glyphs.iter().map(|g| g.w).sum();
            width = width.max(run_width);
            height = height.max(run.line_y + size);
        }

        (width, height)
    }
}

#[cfg(feature = "text")]
impl Default for TextPipeline {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
#[cfg(feature = "text")]
mod tests {
    use super::*;

    #[test]
    fn test_text_pipeline_creation() {
        let pipeline = TextPipeline::new();
        assert!(pipeline.glyph_cache.entries.is_empty());
    }

    #[test]
    fn test_measure_text() {
        let mut pipeline = TextPipeline::new();
        let (w, h) = pipeline.measure_text("Hello", 16.0);
        assert!(w > 0.0);
        assert!(h > 0.0);
    }
}
