//! Animation System - Tween, AnimationController, Animated Widgets
//!
//! This module provides animation primitives and animated widgets:
//!
//! - [`Tween`] - Interpolation between values
//! - [`AnimationController`] - Controls animation playback
//! - [`AnimatedContainer`] - Animates container properties
//! - [`AnimatedOpacity`] - Animates opacity
//! - [`AnimatedScale`] - Animates scale
//! - [`AnimatedSlide`] - Animates position
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{AnimatedContainer, Curve};
//!
//! let container = AnimatedContainer::new()
//!     .duration_ms(300)
//!     .curve(Curve::EaseInOut)
//!     .color(new_color);
//! ```

use std::any::Any;
use venom_core::{BoxConstraints, Color, Insets, Offset, Rect, Size, BorderRadius};
use venom_render::{PaintCanvas, Paint};
use crate::{BoxedWidget, Widget};

// ============================================================================
// ANIMATION CURVES
// ============================================================================

/// Animation easing curves
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Curve {
    /// Linear interpolation
    #[default]
    Linear,
    /// Ease in (slow start)
    EaseIn,
    /// Ease out (slow end)
    EaseOut,
    /// Ease in and out
    EaseInOut,
    /// Bounce effect
    Bounce,
    /// Elastic effect
    Elastic,
}

impl Curve {
    /// Transform a linear t (0.0 to 1.0) using this curve
    pub fn transform(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curve::Linear => t,
            Curve::EaseIn => t * t,
            Curve::EaseOut => t * (2.0 - t),
            Curve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Curve::Bounce => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 - 8.0 * t * t * t * t
                }
            }
            Curve::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    (2.0_f32.powf(-10.0 * t) * ((t - s) * std::f32::consts::TAU / p).sin() + 1.0)
                }
            }
        }
    }
}

// ============================================================================
// TWEEN
// ============================================================================

/// Interpolation between two values
#[derive(Debug, Clone, Copy)]
pub struct Tween<T> {
    /// Start value
    pub begin: T,
    /// End value
    pub end: T,
}

impl<T: Copy> Tween<T> {
    /// Create a new tween
    pub fn new(begin: T, end: T) -> Self {
        Self { begin, end }
    }
}

impl Tween<f32> {
    /// Interpolate at position t (0.0 to 1.0)
    pub fn lerp(&self, t: f32) -> f32 {
        self.begin + (self.end - self.begin) * t
    }
}

impl Tween<Color> {
    /// Interpolate colors at position t
    pub fn lerp(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgba(
            (self.begin.r as f32 + (self.end.r as f32 - self.begin.r as f32) * t) as u8,
            (self.begin.g as f32 + (self.end.g as f32 - self.begin.g as f32) * t) as u8,
            (self.begin.b as f32 + (self.end.b as f32 - self.begin.b as f32) * t) as u8,
            (self.begin.a as f32 + (self.end.a as f32 - self.begin.a as f32) * t) as u8,
        )
    }
}

impl Tween<Offset> {
    /// Interpolate offsets at position t
    pub fn lerp(&self, t: f32) -> Offset {
        Offset::new(
            self.begin.dx + (self.end.dx - self.begin.dx) * t,
            self.begin.dy + (self.end.dy - self.begin.dy) * t,
        )
    }
}

impl Tween<Size> {
    /// Interpolate sizes at position t
    pub fn lerp(&self, t: f32) -> Size {
        Size::new(
            self.begin.width + (self.end.width - self.begin.width) * t,
            self.begin.height + (self.end.height - self.begin.height) * t,
        )
    }
}

// ============================================================================
// ANIMATION STATE
// ============================================================================

/// Animation playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationStatus {
    /// Animation has not started
    #[default]
    Idle,
    /// Animation is playing forward
    Forward,
    /// Animation is playing in reverse
    Reverse,
    /// Animation has completed
    Completed,
}

/// Animation value holder
#[derive(Debug, Clone)]
pub struct AnimationValue {
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Animation curve
    pub curve: Curve,
    /// Animation status
    pub status: AnimationStatus,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Start timestamp (for external tracking)
    pub start_time_ms: u64,
}

impl Default for AnimationValue {
    fn default() -> Self {
        Self {
            progress: 0.0,
            curve: Curve::default(),
            duration_ms: 300,
            status: AnimationStatus::Idle,
            start_time_ms: 0,
        }
    }
}

impl AnimationValue {
    /// Create a new animation value
    pub fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms,
            ..Default::default()
        }
    }

    /// Set the curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Get the curved value
    pub fn value(&self) -> f32 {
        self.curve.transform(self.progress)
    }

    /// Check if animation is complete
    pub fn is_completed(&self) -> bool {
        self.status == AnimationStatus::Completed
    }

    /// Check if animation is running
    pub fn is_animating(&self) -> bool {
        matches!(self.status, AnimationStatus::Forward | AnimationStatus::Reverse)
    }
}

// ============================================================================
// OPACITY WIDGET
// ============================================================================

/// Widget that applies opacity to its child
pub struct Opacity {
    /// Child widget
    child: BoxedWidget,
    /// Opacity value (0.0 to 1.0)
    opacity: f32,
}

impl Opacity {
    /// Create a new opacity widget
    pub fn new<W: Widget + 'static>(child: W, opacity: f32) -> Self {
        Self {
            child: Box::new(child),
            opacity: opacity.clamp(0.0, 1.0),
        }
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl Widget for Opacity {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Note: True opacity would require compositing. For now we just paint the child.
        // A proper implementation would use canvas layers.
        if self.opacity > 0.0 {
            self.child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ANIMATED CONTAINER
// ============================================================================

/// Container that animates property changes
pub struct AnimatedContainer {
    /// Child widget
    child: Option<BoxedWidget>,
    /// Current width
    width: Option<f32>,
    /// Current height
    height: Option<f32>,
    /// Current color
    color: Color,
    /// Current padding
    padding: Insets,
    /// Current border radius
    border_radius: f32,
    /// Animation duration
    duration_ms: u32,
    /// Animation curve
    curve: Curve,
}

impl Default for AnimatedContainer {
    fn default() -> Self {
        Self {
            child: None,
            width: None,
            height: None,
            color: Color::TRANSPARENT,
            padding: Insets::ZERO,
            border_radius: 0.0,
            duration_ms: 300,
            curve: Curve::EaseInOut,
        }
    }
}

impl AnimatedContainer {
    /// Create a new animated container
    pub fn new() -> Self {
        Self::default()
    }

    /// Set child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

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

    /// Set color
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }

    /// Set animation duration
    pub fn duration_ms(mut self, duration: u32) -> Self {
        self.duration_ms = duration;
        self
    }

    /// Set animation curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl Widget for AnimatedContainer {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let width = self.width.unwrap_or(constraints.max_width);
        let height = self.height.unwrap_or(constraints.max_height);
        Size::new(
            width.min(constraints.max_width).max(constraints.min_width),
            height.min(constraints.max_height).max(constraints.min_height),
        )
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.layout(BoxConstraints::new());
        let rect = Rect::new(offset.dx, offset.dy, size.width, size.height);

        // Draw background
        if self.border_radius > 0.0 {
            canvas.draw_rounded_rect(
                rect,
                BorderRadius::all(self.border_radius),
                &Paint::fill(self.color),
            );
        } else {
            canvas.draw_rect(rect, &Paint::fill(self.color));
        }

        // Paint child
        if let Some(child) = &self.child {
            let child_offset = Offset::new(
                offset.dx + self.padding.left,
                offset.dy + self.padding.top,
            );
            child.paint(canvas, child_offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ANIMATED OPACITY
// ============================================================================

/// Widget that animates opacity changes
pub struct AnimatedOpacity {
    /// Child widget
    child: BoxedWidget,
    /// Current opacity
    opacity: f32,
    /// Animation duration
    duration_ms: u32,
    /// Animation curve
    curve: Curve,
}

impl AnimatedOpacity {
    /// Create a new animated opacity widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            opacity: 1.0,
            duration_ms: 300,
            curve: Curve::EaseInOut,
        }
    }

    /// Set opacity
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// Set animation duration
    pub fn duration_ms(mut self, duration: u32) -> Self {
        self.duration_ms = duration;
        self
    }

    /// Set animation curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl Widget for AnimatedOpacity {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        if self.opacity > 0.0 {
            self.child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ANIMATED SCALE
// ============================================================================

/// Widget that animates scale changes
pub struct AnimatedScale {
    /// Child widget
    child: BoxedWidget,
    /// Current scale
    scale: f32,
    /// Animation duration
    duration_ms: u32,
    /// Animation curve
    curve: Curve,
}

impl AnimatedScale {
    /// Create a new animated scale widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            scale: 1.0,
            duration_ms: 300,
            curve: Curve::EaseInOut,
        }
    }

    /// Set scale
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.0);
        self
    }

    /// Set animation duration
    pub fn duration_ms(mut self, duration: u32) -> Self {
        self.duration_ms = duration;
        self
    }

    /// Set animation curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl Widget for AnimatedScale {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        let child_size = self.child.layout(constraints);
        Size::new(child_size.width * self.scale, child_size.height * self.scale)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Scale transform would be applied here in a real implementation
        if self.scale > 0.0 {
            self.child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// ANIMATED SLIDE
// ============================================================================

/// Widget that animates position changes
pub struct AnimatedSlide {
    /// Child widget
    child: BoxedWidget,
    /// Current offset (as fraction of child size)
    offset: Offset,
    /// Animation duration
    duration_ms: u32,
    /// Animation curve
    curve: Curve,
}

impl AnimatedSlide {
    /// Create a new animated slide widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            offset: Offset::ZERO,
            duration_ms: 300,
            curve: Curve::EaseInOut,
        }
    }

    /// Set offset (as fraction of child size)
    pub fn offset(mut self, dx: f32, dy: f32) -> Self {
        self.offset = Offset::new(dx, dy);
        self
    }

    /// Set animation duration
    pub fn duration_ms(mut self, duration: u32) -> Self {
        self.duration_ms = duration;
        self
    }

    /// Set animation curve
    pub fn curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }
}

impl Widget for AnimatedSlide {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let child_size = self.child.layout(BoxConstraints::new());
        let slide_offset = Offset::new(
            offset.dx + child_size.width * self.offset.dx,
            offset.dy + child_size.height * self.offset.dy,
        );
        self.child.paint(canvas, slide_offset);
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// TRANSFORM
// ============================================================================

/// Transform widget for rotation, scale, and translation
pub struct Transform {
    /// Child widget
    child: BoxedWidget,
    /// Rotation in degrees
    rotation: f32,
    /// Scale factor
    scale: f32,
    /// Translation offset
    translation: Offset,
    /// Transform origin (as fraction, 0.5 = center)
    origin: (f32, f32),
}

impl Transform {
    /// Create a new transform widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            rotation: 0.0,
            scale: 1.0,
            translation: Offset::ZERO,
            origin: (0.5, 0.5),
        }
    }

    /// Set rotation in degrees
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Set scale factor
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Set translation
    pub fn translate(mut self, dx: f32, dy: f32) -> Self {
        self.translation = Offset::new(dx, dy);
        self
    }

    /// Set transform origin (as fraction, 0.5 = center)
    pub fn origin(mut self, x: f32, y: f32) -> Self {
        self.origin = (x, y);
        self
    }
}

impl Widget for Transform {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        // Apply translation
        let translated_offset = Offset::new(
            offset.dx + self.translation.dx,
            offset.dy + self.translation.dy,
        );
        
        // Note: Full rotation/scale would require canvas transform support
        self.child.paint(canvas, translated_offset);
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ============================================================================
// CLIP WIDGETS
// ============================================================================

/// Clips child to rounded rectangle
pub struct ClipRRect {
    /// Child widget
    child: BoxedWidget,
    /// Border radius
    radius: f32,
}

impl ClipRRect {
    /// Create a new clip widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            radius: 0.0,
        }
    }

    /// Set border radius
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl Widget for ClipRRect {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.child.layout(BoxConstraints::new());
        let clip_rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
        
        canvas.save();
        canvas.set_clip_rect(Some(clip_rect));
        self.child.paint(canvas, offset);
        canvas.restore();
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Clips child to oval/circle
pub struct ClipOval {
    /// Child widget
    child: BoxedWidget,
}

impl ClipOval {
    /// Create a new clip oval widget
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
        }
    }
}

impl Widget for ClipOval {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let size = self.child.layout(BoxConstraints::new());
        let clip_rect = Rect::new(offset.dx, offset.dy, size.width, size.height);
        
        canvas.save();
        canvas.set_clip_rect(Some(clip_rect));
        self.child.paint(canvas, offset);
        canvas.restore();
    }

    fn children(&self) -> &[BoxedWidget] {
        std::slice::from_ref(&self.child)
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
    use crate::SizedBox;

    #[test]
    fn test_curve_linear() {
        assert_eq!(Curve::Linear.transform(0.0), 0.0);
        assert_eq!(Curve::Linear.transform(0.5), 0.5);
        assert_eq!(Curve::Linear.transform(1.0), 1.0);
    }

    #[test]
    fn test_curve_ease_in() {
        assert_eq!(Curve::EaseIn.transform(0.0), 0.0);
        assert!(Curve::EaseIn.transform(0.5) < 0.5); // Slow start
        assert_eq!(Curve::EaseIn.transform(1.0), 1.0);
    }

    #[test]
    fn test_tween_f32() {
        let tween = Tween::new(0.0, 100.0);
        assert_eq!(tween.lerp(0.0), 0.0);
        assert_eq!(tween.lerp(0.5), 50.0);
        assert_eq!(tween.lerp(1.0), 100.0);
    }

    #[test]
    fn test_tween_color() {
        let tween = Tween::new(Color::BLACK, Color::WHITE);
        let mid = tween.lerp(0.5);
        assert_eq!(mid.r, 127);
        assert_eq!(mid.g, 127);
        assert_eq!(mid.b, 127);
    }

    #[test]
    fn test_animated_container() {
        let container = AnimatedContainer::new()
            .width(100.0)
            .height(50.0)
            .color(Color::RED)
            .duration_ms(500)
            .curve(Curve::EaseOut);
        
        assert_eq!(container.duration_ms, 500);
    }

    #[test]
    fn test_opacity() {
        let widget = Opacity::new(SizedBox::square(50.0), 0.5);
        assert_eq!(widget.opacity, 0.5);
    }

    #[test]
    fn test_transform() {
        let widget = Transform::new(SizedBox::square(50.0))
            .rotate(45.0)
            .scale(1.5)
            .translate(10.0, 20.0);
        
        assert_eq!(widget.rotation, 45.0);
        assert_eq!(widget.scale, 1.5);
    }
}
