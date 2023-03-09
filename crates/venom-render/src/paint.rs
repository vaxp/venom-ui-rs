//! Paint - Describes how to draw shapes
//!
//! Paint defines the visual style: color, stroke width, fill mode, etc.
//! Inspired by Flutter's Paint and your C framework's VenomPaint.

use venom_core::Color;

// ============================================================================
// PAINT STYLE
// ============================================================================

/// How to apply paint to a shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PaintStyle {
    /// Fill the interior of the shape
    #[default]
    Fill,
    /// Draw the outline of the shape
    Stroke,
}

// ============================================================================
// STROKE CAP
// ============================================================================

/// How to draw the ends of lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrokeCap {
    /// Flat end, stops exactly at the endpoint
    #[default]
    Butt,
    /// Rounded end, extends past endpoint by half stroke width
    Round,
    /// Square end, extends past endpoint by half stroke width
    Square,
}

// ============================================================================
// STROKE JOIN
// ============================================================================

/// How to draw the junction of two line segments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrokeJoin {
    /// Sharp corner
    #[default]
    Miter,
    /// Rounded corner
    Round,
    /// Flattened corner
    Bevel,
}

// ============================================================================
// BLEND MODE
// ============================================================================

/// How to blend source color with destination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BlendMode {
    /// Draw source over destination (normal)
    #[default]
    SrcOver,
    /// Replace destination with source
    Src,
    /// Keep destination, ignore source
    Dst,
    /// Draw destination over source
    DstOver,
    /// Clear both source and destination
    Clear,
    /// Multiply colors
    Multiply,
    /// Screen blend
    Screen,
    /// Overlay blend
    Overlay,
}

// ============================================================================
// PAINT
// ============================================================================

/// Defines how to visually render shapes
/// 
/// # Example
/// ```
/// use venom_render::Paint;
/// use venom_core::Color;
/// 
/// // Simple fill
/// let fill = Paint::fill(Color::RED);
/// 
/// // Stroke with custom width
/// let stroke = Paint::stroke(Color::BLUE, 2.0);
/// 
/// // Builder pattern for complex paint
/// let custom = Paint::new()
///     .color(Color::hex("#6366f1"))
///     .stroke_width(3.0)
///     .anti_alias(true);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Paint {
    /// The color to use
    pub color: Color,
    /// Fill or stroke
    pub style: PaintStyle,
    /// Width of stroke (only matters for Stroke style)
    pub stroke_width: f32,
    /// How to cap line ends
    pub stroke_cap: StrokeCap,
    /// How to join line segments
    pub stroke_join: StrokeJoin,
    /// Miter limit for StrokeJoin::Miter
    pub miter_limit: f32,
    /// How to blend with background
    pub blend_mode: BlendMode,
    /// Whether to apply anti-aliasing
    pub anti_alias: bool,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            style: PaintStyle::Fill,
            stroke_width: 1.0,
            stroke_cap: StrokeCap::Butt,
            stroke_join: StrokeJoin::Miter,
            miter_limit: 4.0,
            blend_mode: BlendMode::SrcOver,
            anti_alias: true,
        }
    }
}

impl Paint {
    /// Create a new default paint
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // FACTORY METHODS
    // ========================================================================

    /// Create a fill paint with the given color
    pub fn fill(color: Color) -> Self {
        Self {
            color,
            style: PaintStyle::Fill,
            ..Self::default()
        }
    }

    /// Create a stroke paint with the given color and width
    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            color,
            style: PaintStyle::Stroke,
            stroke_width: width,
            ..Self::default()
        }
    }

    // ========================================================================
    // BUILDER METHODS
    // ========================================================================

    /// Set the color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the paint style
    pub fn style(mut self, style: PaintStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the stroke width
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width.max(0.0);
        self
    }

    /// Set the stroke cap style
    pub fn stroke_cap(mut self, cap: StrokeCap) -> Self {
        self.stroke_cap = cap;
        self
    }

    /// Set the stroke join style
    pub fn stroke_join(mut self, join: StrokeJoin) -> Self {
        self.stroke_join = join;
        self
    }

    /// Set the miter limit
    pub fn miter_limit(mut self, limit: f32) -> Self {
        self.miter_limit = limit.max(1.0);
        self
    }

    /// Set the blend mode
    pub fn blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    /// Enable or disable anti-aliasing
    pub fn anti_alias(mut self, enabled: bool) -> Self {
        self.anti_alias = enabled;
        self
    }

    /// Create a copy with modified alpha
    pub fn with_alpha(mut self, alpha: u8) -> Self {
        self.color = self.color.with_alpha(alpha);
        self
    }
}

// ============================================================================
// GRADIENT
// ============================================================================

/// Gradient color stop
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    /// Position along the gradient (0.0 to 1.0)
    pub offset: f32,
    /// Color at this position
    pub color: Color,
}

impl GradientStop {
    /// Create a new gradient stop
    pub const fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

/// Linear gradient definition
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    /// Start point X
    pub start_x: f32,
    /// Start point Y
    pub start_y: f32,
    /// End point X
    pub end_x: f32,
    /// End point Y
    pub end_y: f32,
    /// Color stops
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Create a horizontal gradient
    pub fn horizontal(width: f32, colors: &[Color]) -> Self {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, &color)| {
                let offset = if colors.len() > 1 {
                    i as f32 / (colors.len() - 1) as f32
                } else {
                    0.0
                };
                GradientStop::new(offset, color)
            })
            .collect();

        Self {
            start_x: 0.0,
            start_y: 0.0,
            end_x: width,
            end_y: 0.0,
            stops,
        }
    }

    /// Create a vertical gradient
    pub fn vertical(height: f32, colors: &[Color]) -> Self {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, &color)| {
                let offset = if colors.len() > 1 {
                    i as f32 / (colors.len() - 1) as f32
                } else {
                    0.0
                };
                GradientStop::new(offset, color)
            })
            .collect();

        Self {
            start_x: 0.0,
            start_y: 0.0,
            end_x: 0.0,
            end_y: height,
            stops,
        }
    }
}

/// Radial gradient definition
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    /// Center X
    pub center_x: f32,
    /// Center Y
    pub center_y: f32,
    /// Radius
    pub radius: f32,
    /// Color stops
    pub stops: Vec<GradientStop>,
}

impl RadialGradient {
    /// Create a radial gradient
    pub fn new(center_x: f32, center_y: f32, radius: f32, colors: &[Color]) -> Self {
        let stops = colors
            .iter()
            .enumerate()
            .map(|(i, &color)| {
                let offset = if colors.len() > 1 {
                    i as f32 / (colors.len() - 1) as f32
                } else {
                    0.0
                };
                GradientStop::new(offset, color)
            })
            .collect();

        Self {
            center_x,
            center_y,
            radius,
            stops,
        }
    }
}

// ============================================================================
// SHADER (for advanced fills)
// ============================================================================

/// Fill shader for complex fills
#[derive(Debug, Clone, PartialEq)]
pub enum Shader {
    /// Solid color
    Solid(Color),
    /// Linear gradient
    Linear(LinearGradient),
    /// Radial gradient
    Radial(RadialGradient),
}

impl From<Color> for Shader {
    fn from(color: Color) -> Self {
        Shader::Solid(color)
    }
}

impl From<LinearGradient> for Shader {
    fn from(gradient: LinearGradient) -> Self {
        Shader::Linear(gradient)
    }
}

impl From<RadialGradient> for Shader {
    fn from(gradient: RadialGradient) -> Self {
        Shader::Radial(gradient)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paint_fill() {
        let paint = Paint::fill(Color::RED);
        assert_eq!(paint.color, Color::RED);
        assert_eq!(paint.style, PaintStyle::Fill);
    }

    #[test]
    fn test_paint_stroke() {
        let paint = Paint::stroke(Color::BLUE, 2.5);
        assert_eq!(paint.color, Color::BLUE);
        assert_eq!(paint.style, PaintStyle::Stroke);
        assert_eq!(paint.stroke_width, 2.5);
    }

    #[test]
    fn test_paint_builder() {
        let paint = Paint::new()
            .color(Color::GREEN)
            .stroke_width(3.0)
            .stroke_cap(StrokeCap::Round)
            .anti_alias(false);

        assert_eq!(paint.color, Color::GREEN);
        assert_eq!(paint.stroke_width, 3.0);
        assert_eq!(paint.stroke_cap, StrokeCap::Round);
        assert!(!paint.anti_alias);
    }

    #[test]
    fn test_linear_gradient() {
        let gradient = LinearGradient::horizontal(100.0, &[Color::RED, Color::BLUE]);
        assert_eq!(gradient.stops.len(), 2);
        assert_eq!(gradient.stops[0].offset, 0.0);
        assert_eq!(gradient.stops[1].offset, 1.0);
    }
}
