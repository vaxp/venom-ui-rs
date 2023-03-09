//! VenomUI Render - Backend-agnostic rendering abstraction
//!
//! This crate provides a Canvas API that can be implemented by different backends:
//! - Software renderer (CPU) via `tiny-skia`
//! - OpenGL/Vulkan (future)
//! - GPU via wgpu (future)
//!
//! The design is inspired by Flutter's rendering layer.
//!
//! # Features
//!
//! - `software` (default) - CPU-based software renderer using tiny-skia
//! - `text` - Text rendering using cosmic-text

#![warn(missing_docs)]

mod canvas;
mod paint;
mod path;
mod transform;

#[cfg(feature = "software")]
mod software;

#[cfg(feature = "text")]
mod text_pipeline;

pub use canvas::*;
pub use paint::*;
pub use path::*;
pub use transform::*;

#[cfg(feature = "software")]
pub use software::*;

#[cfg(feature = "text")]
pub use text_pipeline::*;
