//! Cubit - Simple State Management
//!
//! Cubit is a lightweight state management solution perfect for:
//! - Simple state updates without complex event handling
//! - Quick prototyping
//! - Small to medium-sized features
//!
//! For more complex event-driven state management, see [`Bloc`](crate::Bloc).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │                 Cubit                   │
//! │                                         │
//! │   ┌─────────┐     ┌───────────────┐    │
//! │   │ Methods │────▶│ emit(state)   │    │
//! │   └─────────┘     └───────┬───────┘    │
//! │                           │            │
//! │                           ▼            │
//! │                   ┌───────────────┐    │
//! │                   │  StateStream  │    │
//! │                   └───────┬───────┘    │
//! │                           │            │
//! └───────────────────────────┼────────────┘
//!                             │
//!                             ▼
//!                    ┌─────────────────┐
//!                    │   Subscribers   │
//!                    └─────────────────┘
//! ```
//!
//! # Example
//!
//! ```
//! use venom_core::{CubitCore, SubscriptionHandle};
//!
//! // 1. Define your cubit
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
//!     fn decrement(&self) {
//!         self.core.emit(self.core.state() - 1);
//!     }
//!
//!     fn state(&self) -> i32 {
//!         self.core.state()
//!     }
//!
//!     fn subscribe<F: Fn(&i32, &i32) + 'static>(&self, f: F) -> SubscriptionHandle {
//!         self.core.subscribe(f)
//!     }
//! }
//!
//! // 2. Use it
//! let counter = CounterCubit::new();
//! assert_eq!(counter.state(), 0);
//!
//! counter.increment();
//! assert_eq!(counter.state(), 1);
//!
//! counter.increment();
//! counter.increment();
//! assert_eq!(counter.state(), 3);
//!
//! counter.decrement();
//! assert_eq!(counter.state(), 2);
//! ```

use crate::subscription::{StateStream, SubscriptionHandle};
use std::rc::Rc;

// ============================================================================
// CUBIT CORE
// ============================================================================

/// Core implementation for Cubit pattern
///
/// `CubitCore` provides the state management infrastructure for cubits.
/// It wraps a [`StateStream`] and provides a clean API for state updates.
///
/// # Type Parameters
///
/// * `S` - The state type, must implement `Clone + PartialEq`
///
/// # Example
///
/// ```
/// use venom_core::CubitCore;
///
/// // Simple counter
/// let cubit = CubitCore::new(0);
/// cubit.emit(cubit.state() + 1);
/// assert_eq!(cubit.state(), 1);
///
/// // With subscription
/// let _sub = cubit.subscribe(|old, new| {
///     println!("Changed: {} -> {}", old, new);
/// });
///
/// cubit.emit(42); // Prints: "Changed: 1 -> 42"
/// ```
pub struct CubitCore<S> {
    stream: StateStream<S>,
    /// Optional name for debugging
    name: Option<String>,
}

impl<S> CubitCore<S> {
    /// Create a new cubit with an initial state
    pub fn new(initial: S) -> Self {
        Self {
            stream: StateStream::new(initial),
            name: None,
        }
    }

    /// Create a new cubit with a name (useful for debugging)
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            stream: StateStream::new(initial),
            name: Some(name.into()),
        }
    }

    /// Get the cubit's name (if set)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the current state version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }
}

impl<S: Clone> CubitCore<S> {
    /// Get a clone of the current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Execute a function with a reference to the current state
    pub fn with_state<R, F: FnOnce(&S) -> R>(&self, f: F) -> R {
        self.stream.with_state(f)
    }
}

impl<S: Clone + PartialEq> CubitCore<S> {
    /// Emit a new state
    ///
    /// If the new state equals the current state, no action is taken
    /// and subscribers are not notified.
    ///
    /// Returns `true` if the state actually changed.
    pub fn emit(&self, state: S) -> bool {
        self.stream.emit(state)
    }

    /// Update the state using a function
    ///
    /// This is useful for making updates based on the current state.
    ///
    /// # Example
    ///
    /// ```
    /// use venom_core::CubitCore;
    ///
    /// let cubit = CubitCore::new(vec![1, 2, 3]);
    /// cubit.update(|items| {
    ///     let mut new_items = items.clone();
    ///     new_items.push(4);
    ///     new_items
    /// });
    /// assert_eq!(cubit.state(), vec![1, 2, 3, 4]);
    /// ```
    pub fn update<F: FnOnce(&S) -> S>(&self, f: F) -> bool {
        self.stream.update(f)
    }

    /// Reset the state to a new value
    ///
    /// This is semantic sugar for `emit` but signals intent to reset.
    pub fn reset(&self, state: S) -> bool {
        self.emit(state)
    }
}

impl<S: Clone + PartialEq + Default> CubitCore<S> {
    /// Reset the state to default
    pub fn reset_to_default(&self) -> bool {
        self.emit(S::default())
    }
}

impl<S: 'static> CubitCore<S> {
    /// Subscribe to state changes
    ///
    /// The callback receives both old and new state values.
    /// Returns a handle that unsubscribes when dropped.
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        self.stream.subscribe(callback)
    }

    /// Subscribe to state changes (receives only new state)
    pub fn subscribe_new<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S) + 'static,
    {
        self.stream.subscribe_new(callback)
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.stream.subscriber_count()
    }
}

impl<S> Clone for CubitCore<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            name: self.name.clone(),
        }
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for CubitCore<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CubitCore")
            .field("name", &self.name)
            .field("stream", &self.stream)
            .finish()
    }
}

impl<S: Default> Default for CubitCore<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// ============================================================================
// CUBIT TRAIT
// ============================================================================

/// Trait for implementing typed cubits
///
/// This trait provides a structured way to define cubits with
/// a specific state type and custom methods.
///
/// # Example
///
/// ```ignore
/// use venom_core::{Cubit, CubitCore, SubscriptionHandle};
///
/// #[derive(Clone, PartialEq, Default)]
/// struct TimerState {
///     seconds: u32,
///     running: bool,
/// }
///
/// struct TimerCubit {
///     core: CubitCore<TimerState>,
/// }
///
/// impl Cubit for TimerCubit {
///     type State = TimerState;
///
///     fn core(&self) -> &CubitCore<Self::State> {
///         &self.core
///     }
/// }
///
/// impl TimerCubit {
///     fn new() -> Self {
///         Self { core: CubitCore::new(TimerState::default()) }
///     }
///
///     fn start(&self) {
///         self.update(|s| TimerState { running: true, ..s.clone() });
///     }
///
///     fn stop(&self) {
///         self.update(|s| TimerState { running: false, ..s.clone() });
///     }
///
///     fn tick(&self) {
///         self.update(|s| TimerState {
///             seconds: s.seconds + 1,
///             ..s.clone()
///         });
///     }
/// }
/// ```
pub trait Cubit {
    /// The state type managed by this cubit
    type State: Clone + PartialEq + 'static;

    /// Get a reference to the cubit core
    fn core(&self) -> &CubitCore<Self::State>;

    /// Get the current state
    fn state(&self) -> Self::State {
        self.core().state()
    }

    /// Get the current version
    fn version(&self) -> u64 {
        self.core().version()
    }

    /// Emit a new state
    fn emit(&self, state: Self::State) -> bool {
        self.core().emit(state)
    }

    /// Update state using a function
    fn update<F: FnOnce(&Self::State) -> Self::State>(&self, f: F) -> bool {
        self.core().update(f)
    }

    /// Subscribe to state changes
    fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&Self::State, &Self::State) + 'static,
    {
        self.core().subscribe(callback)
    }
}

// ============================================================================
// SIMPLE CUBIT
// ============================================================================

/// A pre-built simple cubit for quick usage
///
/// Use this when you don't need custom methods and just want
/// basic state management.
///
/// # Example
///
/// ```
/// use venom_core::{SimpleCubit, Cubit};
///
/// // Counter
/// let counter = SimpleCubit::new(0);
/// counter.emit(counter.state() + 1);
///
/// // Settings
/// #[derive(Clone, PartialEq, Default)]
/// struct Settings {
///     dark_mode: bool,
///     font_size: u32,
/// }
///
/// let settings = SimpleCubit::new(Settings::default());
/// settings.update(|s: &Settings| Settings { dark_mode: true, ..s.clone() });
/// ```
#[derive(Clone)]
pub struct SimpleCubit<S> {
    core: CubitCore<S>,
}

impl<S> SimpleCubit<S> {
    /// Create a new simple cubit
    pub fn new(initial: S) -> Self {
        Self {
            core: CubitCore::new(initial),
        }
    }

    /// Create with a name for debugging
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            core: CubitCore::named(name, initial),
        }
    }
}

impl<S: Clone + PartialEq + 'static> Cubit for SimpleCubit<S> {
    type State = S;

    fn core(&self) -> &CubitCore<Self::State> {
        &self.core
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for SimpleCubit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleCubit")
            .field("core", &self.core)
            .finish()
    }
}

impl<S: Default> Default for SimpleCubit<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// ============================================================================
// SHARED CUBIT
// ============================================================================

/// A reference-counted cubit for sharing across components
///
/// This is useful when multiple widgets need access to the same cubit.
///
/// # Example
///
/// ```
/// use venom_core::{SharedCubit, Cubit};
///
/// let cubit = SharedCubit::new(0);
///
/// // Clone for different components
/// let cubit_for_button = cubit.clone();
/// let cubit_for_display = cubit.clone();
///
/// // All clones share the same state
/// cubit_for_button.emit(42);
/// assert_eq!(cubit_for_display.state(), 42);
/// ```
#[derive(Clone)]
pub struct SharedCubit<S> {
    core: Rc<CubitCore<S>>,
}

impl<S> SharedCubit<S> {
    /// Create a new shared cubit
    pub fn new(initial: S) -> Self {
        Self {
            core: Rc::new(CubitCore::new(initial)),
        }
    }

    /// Create with a name for debugging
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            core: Rc::new(CubitCore::named(name, initial)),
        }
    }

    /// Get the number of references to this cubit
    pub fn ref_count(&self) -> usize {
        Rc::strong_count(&self.core)
    }
}

impl<S: Clone + PartialEq + 'static> Cubit for SharedCubit<S> {
    type State = S;

    fn core(&self) -> &CubitCore<Self::State> {
        &self.core
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for SharedCubit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedCubit")
            .field("core", &self.core)
            .field("ref_count", &Rc::strong_count(&self.core))
            .finish()
    }
}

impl<S: Default> Default for SharedCubit<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn test_cubit_core_basic() {
        let cubit = CubitCore::new(0);
        assert_eq!(cubit.state(), 0);

        cubit.emit(42);
        assert_eq!(cubit.state(), 42);
    }

    #[test]
    fn test_cubit_core_update() {
        let cubit = CubitCore::new(10);
        cubit.update(|n| n * 2);
        assert_eq!(cubit.state(), 20);
    }

    #[test]
    fn test_cubit_core_no_change() {
        let cubit = CubitCore::new(42);
        let changed = cubit.emit(42);
        assert!(!changed);
    }

    #[test]
    fn test_cubit_core_subscribe() {
        let cubit = CubitCore::new(0);
        let call_count = Rc::new(Cell::new(0));
        let count_clone = Rc::clone(&call_count);

        let _handle = cubit.subscribe(move |_, _| {
            count_clone.set(count_clone.get() + 1);
        });

        cubit.emit(1);
        cubit.emit(2);
        cubit.emit(3);

        assert_eq!(call_count.get(), 3);
    }

    #[test]
    fn test_cubit_core_named() {
        let cubit = CubitCore::named("counter", 0);
        assert_eq!(cubit.name(), Some("counter"));
    }

    #[test]
    fn test_simple_cubit() {
        let cubit = SimpleCubit::new(0);
        cubit.emit(10);
        assert_eq!(cubit.state(), 10);
    }

    #[test]
    fn test_shared_cubit() {
        let cubit1 = SharedCubit::new(0);
        let cubit2 = cubit1.clone();

        cubit1.emit(42);
        assert_eq!(cubit2.state(), 42);
        assert_eq!(cubit1.ref_count(), 2);
    }

    #[test]
    fn test_cubit_reset_to_default() {
        let cubit = CubitCore::new(42);
        cubit.reset_to_default();
        assert_eq!(cubit.state(), 0);
    }

    #[test]
    fn test_counter_cubit_example() {
        struct CounterCubit {
            core: CubitCore<i32>,
        }

        impl CounterCubit {
            fn new() -> Self {
                Self { core: CubitCore::new(0) }
            }

            fn increment(&self) {
                self.core.emit(self.core.state() + 1);
            }

            fn decrement(&self) {
                self.core.emit(self.core.state() - 1);
            }

            fn state(&self) -> i32 {
                self.core.state()
            }
        }

        let counter = CounterCubit::new();
        assert_eq!(counter.state(), 0);

        counter.increment();
        counter.increment();
        assert_eq!(counter.state(), 2);

        counter.decrement();
        assert_eq!(counter.state(), 1);
    }
}
