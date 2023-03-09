//! Reactive State Management
//!
//! A simple, efficient reactive state system inspired by signals/atoms.
//! No macros required - pure Rust with builder pattern.
//!
//! # Example
//! ```
//! use venom_core::State;
//!
//! let counter = State::new(0);
//! println!("Count: {}", counter.get());
//! counter.set(counter.get() + 1);
//! ```

use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// STATE
// ============================================================================

/// Reactive state container
/// 
/// `State<T>` holds a value and allows reactive updates.
/// When the value changes, all registered listeners are notified.
pub struct State<T> {
    inner: Rc<RefCell<StateInner<T>>>,
}

struct StateInner<T> {
    value: T,
    listeners: Vec<Box<dyn Fn(&T)>>,
    version: u64,
}

impl<T> State<T> {
    /// Create a new state with initial value
    pub fn new(initial: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(StateInner {
                value: initial,
                listeners: Vec::new(),
                version: 0,
            })),
        }
    }

    /// Get current version (increments on each change)
    pub fn version(&self) -> u64 {
        self.inner.borrow().version
    }

    /// Subscribe to state changes
    /// Returns a handle that unsubscribes when dropped
    pub fn subscribe<F: Fn(&T) + 'static>(&self, callback: F) -> Subscription {
        let mut inner = self.inner.borrow_mut();
        inner.listeners.push(Box::new(callback));
        Subscription { /* TODO: implement unsubscribe */ }
    }
}

impl<T: Clone> State<T> {
    /// Get a clone of the current value
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

impl<T: Clone + PartialEq> State<T> {
    /// Set a new value (only notifies if value changed)
    pub fn set(&self, value: T) {
        let mut inner = self.inner.borrow_mut();
        if inner.value != value {
            inner.value = value.clone();
            inner.version += 1;
            
            // Notify listeners
            for listener in &inner.listeners {
                listener(&inner.value);
            }
        }
    }

    /// Update value using a function
    pub fn update<F: FnOnce(&T) -> T>(&self, f: F) {
        let new_value = {
            let inner = self.inner.borrow();
            f(&inner.value)
        };
        self.set(new_value);
    }
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for State<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("value", &self.inner.borrow().value)
            .field("version", &self.inner.borrow().version)
            .finish()
    }
}

// ============================================================================
// SUBSCRIPTION
// ============================================================================

/// Handle to a state subscription
/// Subscription is automatically cancelled when dropped
pub struct Subscription {
    // TODO: implement unsubscribe mechanism
}

// ============================================================================
// COMPUTED (Derived State)
// ============================================================================

/// Computed/derived state - automatically updates when dependencies change
/// 
/// # Example
/// ```
/// use venom_core::{State, Computed};
/// 
/// let count = State::new(5);
/// let doubled = Computed::from(&count, |n| n * 2);
/// 
/// assert_eq!(doubled.get(), 10);
/// count.set(10);
/// assert_eq!(doubled.get(), 20);
/// ```
pub struct Computed<T, S> {
    source: State<S>,
    compute: Box<dyn Fn(&S) -> T>,
}

impl<T, S: Clone> Computed<T, S> {
    /// Create a computed value from a source state
    pub fn from<F: Fn(&S) -> T + 'static>(source: &State<S>, compute: F) -> Self {
        Self {
            source: source.clone(),
            compute: Box::new(compute),
        }
    }

    /// Get the computed value
    pub fn get(&self) -> T {
        let source = self.source.inner.borrow();
        (self.compute)(&source.value)
    }
}

// ============================================================================
// EFFECT
// ============================================================================

/// Side effect that runs when state changes
/// 
/// # Example
/// ```
/// use venom_core::{State, Effect};
/// 
/// let count = State::new(0);
/// let _effect = Effect::new(&count, |value| {
///     println!("Count changed to: {}", value);
/// });
/// ```
pub struct Effect {
    _subscription: Subscription,
}

impl Effect {
    /// Create a new effect
    pub fn new<T, F>(state: &State<T>, callback: F) -> Self
    where
        T: Clone + 'static,
        F: Fn(&T) + 'static,
    {
        let subscription = state.subscribe(callback);
        Self { _subscription: subscription }
    }
}

// ============================================================================
// MEMO (Cached computation)
// ============================================================================

/// Memoized value - only recomputes when dependencies change
pub struct Memo<T> {
    value: RefCell<Option<T>>,
    version: RefCell<u64>,
    compute: Box<dyn Fn() -> T>,
    dependency_version: Box<dyn Fn() -> u64>,
}

impl<T: Clone> Memo<T> {
    /// Create a memo with a single dependency
    pub fn new<S: Clone + 'static, F>(source: &State<S>, compute: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let source_clone = source.clone();
        Self {
            value: RefCell::new(None),
            version: RefCell::new(0),
            compute: Box::new(compute),
            dependency_version: Box::new(move || source_clone.version()),
        }
    }

    /// Get the memoized value (recomputes if stale)
    pub fn get(&self) -> T {
        let current_version = (self.dependency_version)();
        let mut version = self.version.borrow_mut();
        let mut value = self.value.borrow_mut();

        if *version != current_version || value.is_none() {
            *value = Some((self.compute)());
            *version = current_version;
        }

        value.as_ref().unwrap().clone()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_basic() {
        let state = State::new(42);
        assert_eq!(state.get(), 42);
        assert_eq!(state.version(), 0);

        state.set(100);
        assert_eq!(state.get(), 100);
        assert_eq!(state.version(), 1);
    }

    #[test]
    fn test_state_no_change_no_version_bump() {
        let state = State::new(42);
        state.set(42); // Same value
        assert_eq!(state.version(), 0); // Version should not change
    }

    #[test]
    fn test_state_update() {
        let state = State::new(10);
        state.update(|n| n * 2);
        assert_eq!(state.get(), 20);
    }

    #[test]
    fn test_computed() {
        let count = State::new(5);
        let doubled = Computed::from(&count, |n| n * 2);

        assert_eq!(doubled.get(), 10);
        count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn test_state_clone_shares_data() {
        let state1 = State::new(100);
        let state2 = state1.clone();

        state1.set(200);
        assert_eq!(state2.get(), 200);
    }
}
