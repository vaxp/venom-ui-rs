//! VenomUI Core - Fundamental types and primitives
//!
//! This crate provides the foundation for VenomUI:
//! - Primitive types (Color, Size, Rect, Point)
//! - Constraint system (BoxConstraints)
//! - Widget trait definitions
//! - Reactive state management (Cubit, Bloc, Selectors)
//!
//! # State Management
//!
//! VenomUI provides a powerful state management system inspired by Flutter's BLoC pattern:
//!
//! - [`StateStream`] - Observable state container with auto-unsubscribe
//! - [`CubitCore`] / [`SimpleCubit`] - Simple state management for basic use cases
//! - [`StatefulBloc`] / [`BlocCore`] - Event-driven state for complex scenarios
//! - [`Selector`] - Derived state with memoization
//! - Provider utilities for dependency injection
//!
//! ## Quick Example
//!
//! ```
//! use venom_core::{CubitCore, Cubit};
//!
//! // Simple counter cubit
//! struct CounterCubit {
//!     core: CubitCore<i32>,
//! }
//!
//! impl CounterCubit {
//!     fn new() -> Self {
//!         Self { core: CubitCore::new(0) }
//!     }
//!
//!     fn increment(&self) {
//!         self.core.emit(self.core.state() + 1);
//!     }
//!
//!     fn state(&self) -> i32 {
//!         self.core.state()
//!     }
//! }
//!
//! let counter = CounterCubit::new();
//! counter.increment();
//! assert_eq!(counter.state(), 1);
//! ```

// Note: We use deny instead of forbid to allow the sync_state module
// to implement Send+Sync for thread-safe state management types.
// The unsafe code is limited and well-audited.
#![deny(unsafe_code)]
#![warn(missing_docs)]

mod types;
mod constraints;
mod state;

// State management modules
mod subscription;
mod cubit;
mod bloc;
mod selector;
mod provider;

// Advanced state management
mod devtools;
mod async_bloc;
mod hydration;
mod sync_state;

// Core types
pub use types::*;
pub use constraints::*;
pub use state::*;

// State management exports
pub use subscription::*;
pub use cubit::*;
pub use bloc::*;
pub use selector::*;
pub use provider::*;

// Advanced state management exports
pub use devtools::*;
pub use async_bloc::*;
pub use hydration::*;
pub use sync_state::*;
