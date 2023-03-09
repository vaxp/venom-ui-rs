//! VenomUI App - Window and application management
//!
//! This crate provides window creation and event loop management for VenomUI
//! applications. It uses `winit` for cross-platform window management and
//! `softbuffer` for presenting rendered frames.
//!
//! # Example
//!
//! ```ignore
//! use venom_app::{App, AppConfig};
//! use venom_widgets::{Container, Text, Color};
//!
//! fn main() {
//!     let app = App::new(AppConfig {
//!         title: "My VenomUI App".into(),
//!         width: 800,
//!         height: 600,
//!         ..Default::default()
//!     });
//!
//!     app.run(|ctx| {
//!         Container::new()
//!             .color(Color::hex("#1a1a2e"))
//!             .child(Text::new("Hello, VenomUI!"))
//!     });
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "winit-app")]
mod window;

#[cfg(feature = "winit-app")]
pub use window::*;

// Re-export dependencies for convenience
pub use venom_core;
pub use venom_render;
pub use venom_widgets;
