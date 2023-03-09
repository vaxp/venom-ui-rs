//! Subscription System for State Management
//!
//! This module provides a robust subscription system that allows components to
//! listen to state changes and automatically unsubscribe when dropped.
//!
//! # Key Types
//!
//! - [`SubscriptionId`] - Unique identifier for each subscription
//! - [`SubscriptionHandle`] - RAII handle that unsubscribes on drop
//! - [`StateStream`] - Observable state container with subscription support
//!
//! # Example
//!
//! ```
//! use venom_core::{StateStream, SubscriptionHandle};
//!
//! let stream: StateStream<i32> = StateStream::new(0);
//!
//! // Subscribe to changes
//! let handle = stream.subscribe(|old, new| {
//!     println!("Changed from {} to {}", old, new);
//! });
//!
//! stream.emit(42); // Prints: "Changed from 0 to 42"
//!
//! // Subscription automatically cancelled when handle is dropped
//! drop(handle);
//! stream.emit(100); // No output - subscription was cancelled
//! ```

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

// ============================================================================
// SUBSCRIPTION ID
// ============================================================================

/// Global counter for generating unique subscription IDs
static NEXT_SUBSCRIPTION_ID: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a subscription
///
/// Each subscription in the application has a unique ID that can be used
/// to identify and manage it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

impl SubscriptionId {
    /// Create a new unique subscription ID
    pub fn new() -> Self {
        Self(NEXT_SUBSCRIPTION_ID.fetch_add(1, Ordering::SeqCst))
    }

    /// Get the raw ID value (useful for debugging)
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for SubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SUBSCRIPTION HANDLE
// ============================================================================

/// Handle to an active subscription
///
/// When this handle is dropped, the subscription is automatically cancelled.
/// This provides RAII-style subscription management.
///
/// # Example
///
/// ```
/// use venom_core::StateStream;
///
/// let stream = StateStream::new(0);
///
/// {
///     let _handle = stream.subscribe(|_, new| println!("Value: {}", new));
///     stream.emit(1); // Listener is called
/// } // _handle dropped here, subscription cancelled
///
/// stream.emit(2); // Listener is NOT called
/// ```
pub struct SubscriptionHandle {
    id: SubscriptionId,
    unsubscribe_fn: Option<Box<dyn FnOnce()>>,
}

impl SubscriptionHandle {
    /// Create a new subscription handle
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription's unique identifier
    /// * `unsubscribe_fn` - Function to call when unsubscribing
    pub fn new<F: FnOnce() + 'static>(id: SubscriptionId, unsubscribe_fn: F) -> Self {
        Self {
            id,
            unsubscribe_fn: Some(Box::new(unsubscribe_fn)),
        }
    }

    /// Create a no-op handle (for testing or placeholder)
    pub fn noop() -> Self {
        Self {
            id: SubscriptionId::new(),
            unsubscribe_fn: None,
        }
    }

    /// Get the subscription ID
    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    /// Manually unsubscribe (consumes the handle)
    pub fn unsubscribe(mut self) {
        if let Some(unsub) = self.unsubscribe_fn.take() {
            unsub();
        }
    }

    /// Cancel automatic unsubscription (leak the subscription)
    ///
    /// This is useful when you want the subscription to live forever
    /// without keeping the handle around.
    pub fn detach(mut self) {
        self.unsubscribe_fn = None;
    }
}

impl Drop for SubscriptionHandle {
    fn drop(&mut self) {
        if let Some(unsub) = self.unsubscribe_fn.take() {
            unsub();
        }
    }
}

impl std::fmt::Debug for SubscriptionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionHandle")
            .field("id", &self.id)
            .field("active", &self.unsubscribe_fn.is_some())
            .finish()
    }
}

// ============================================================================
// STATE STREAM
// ============================================================================

/// Observable state container with subscription support
///
/// `StateStream` is the foundation of the state management system.
/// It holds a value and notifies all subscribers when it changes.
///
/// # Features
///
/// - Efficient change detection (only notifies if value actually changed)
/// - Version tracking for advanced use cases
/// - RAII subscription management (auto-unsubscribe on handle drop)
/// - Multiple concurrent subscribers
///
/// # Example
///
/// ```
/// use venom_core::StateStream;
///
/// let stream = StateStream::new("Hello".to_string());
///
/// let handle = stream.subscribe(|old, new| {
///     println!("Changed: '{}' -> '{}'", old, new);
/// });
///
/// stream.emit("World".to_string());
/// // Output: Changed: 'Hello' -> 'World'
/// ```
pub struct StateStream<S> {
    current: Rc<RefCell<S>>,
    listeners: Rc<RefCell<HashMap<SubscriptionId, Box<dyn Fn(&S, &S)>>>>,
    version: Rc<Cell<u64>>,
}

impl<S> StateStream<S> {
    /// Create a new state stream with an initial value
    pub fn new(initial: S) -> Self {
        Self {
            current: Rc::new(RefCell::new(initial)),
            listeners: Rc::new(RefCell::new(HashMap::new())),
            version: Rc::new(Cell::new(0)),
        }
    }

    /// Get the current version number
    ///
    /// The version increments each time the state changes.
    /// Useful for caching and memoization.
    pub fn version(&self) -> u64 {
        self.version.get()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.listeners.borrow().len()
    }

    /// Check if there are any active subscribers
    pub fn has_subscribers(&self) -> bool {
        !self.listeners.borrow().is_empty()
    }
}

impl<S: Clone> StateStream<S> {
    /// Get a clone of the current state
    pub fn state(&self) -> S {
        self.current.borrow().clone()
    }

    /// Get a reference to the current state
    ///
    /// Note: This borrows the state, so avoid holding the reference
    /// while emitting new states.
    pub fn with_state<R, F: FnOnce(&S) -> R>(&self, f: F) -> R {
        f(&self.current.borrow())
    }
}

impl<S: Clone + PartialEq> StateStream<S> {
    /// Emit a new state
    ///
    /// If the new state is different from the current state,
    /// all subscribers are notified with both the old and new values.
    ///
    /// Returns `true` if the state actually changed.
    pub fn emit(&self, new_state: S) -> bool {
        let old_state = self.current.borrow().clone();
        
        if old_state == new_state {
            return false;
        }

        // Update state
        *self.current.borrow_mut() = new_state.clone();
        self.version.set(self.version.get() + 1);

        // Notify listeners
        let listeners = self.listeners.borrow();
        for callback in listeners.values() {
            callback(&old_state, &new_state);
        }

        true
    }

    /// Update the state using a function
    ///
    /// The function receives the current state and returns the new state.
    /// This is useful for atomic updates based on the current value.
    pub fn update<F: FnOnce(&S) -> S>(&self, f: F) -> bool {
        let new_state = {
            let current = self.current.borrow();
            f(&current)
        };
        self.emit(new_state)
    }
}

impl<S: 'static> StateStream<S> {
    /// Subscribe to state changes
    ///
    /// The callback receives both the old and new state values.
    /// Returns a handle that cancels the subscription when dropped.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function called with `(old_state, new_state)` on each change
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        let id = SubscriptionId::new();
        self.listeners.borrow_mut().insert(id, Box::new(callback));

        let listeners = Rc::clone(&self.listeners);
        SubscriptionHandle::new(id, move || {
            listeners.borrow_mut().remove(&id);
        })
    }

    /// Subscribe only to the new state value
    ///
    /// A convenience method when you don't need the old state.
    pub fn subscribe_new<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S) + 'static,
    {
        self.subscribe(move |_, new| callback(new))
    }
}

impl<S> Clone for StateStream<S> {
    fn clone(&self) -> Self {
        Self {
            current: Rc::clone(&self.current),
            listeners: Rc::clone(&self.listeners),
            version: Rc::clone(&self.version),
        }
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for StateStream<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateStream")
            .field("current", &self.current.borrow())
            .field("version", &self.version.get())
            .field("subscribers", &self.listeners.borrow().len())
            .finish()
    }
}

impl<S: Default> Default for StateStream<S> {
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
    fn test_subscription_id_unique() {
        let id1 = SubscriptionId::new();
        let id2 = SubscriptionId::new();
        let id3 = SubscriptionId::new();

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_state_stream_basic() {
        let stream = StateStream::new(0);
        assert_eq!(stream.state(), 0);
        assert_eq!(stream.version(), 0);

        stream.emit(42);
        assert_eq!(stream.state(), 42);
        assert_eq!(stream.version(), 1);
    }

    #[test]
    fn test_state_stream_no_change() {
        let stream = StateStream::new(42);
        let changed = stream.emit(42);
        
        assert!(!changed);
        assert_eq!(stream.version(), 0);
    }

    #[test]
    fn test_subscription_receives_changes() {
        let stream = StateStream::new(0);
        let call_count = Rc::new(Cell::new(0));
        let last_values = Rc::new(RefCell::new((0, 0)));

        let call_count_clone = Rc::clone(&call_count);
        let last_values_clone = Rc::clone(&last_values);

        let _handle = stream.subscribe(move |old, new| {
            call_count_clone.set(call_count_clone.get() + 1);
            *last_values_clone.borrow_mut() = (*old, *new);
        });

        stream.emit(10);
        assert_eq!(call_count.get(), 1);
        assert_eq!(*last_values.borrow(), (0, 10));

        stream.emit(20);
        assert_eq!(call_count.get(), 2);
        assert_eq!(*last_values.borrow(), (10, 20));
    }

    #[test]
    fn test_subscription_auto_unsubscribe_on_drop() {
        let stream = StateStream::new(0);
        let call_count = Rc::new(Cell::new(0));

        {
            let call_count_clone = Rc::clone(&call_count);
            let _handle = stream.subscribe(move |_, _| {
                call_count_clone.set(call_count_clone.get() + 1);
            });

            stream.emit(1);
            assert_eq!(call_count.get(), 1);
        } // handle dropped here

        stream.emit(2);
        assert_eq!(call_count.get(), 1); // Still 1, not called
    }

    #[test]
    fn test_subscription_manual_unsubscribe() {
        let stream = StateStream::new(0);
        let call_count = Rc::new(Cell::new(0));

        let call_count_clone = Rc::clone(&call_count);
        let handle = stream.subscribe(move |_, _| {
            call_count_clone.set(call_count_clone.get() + 1);
        });

        stream.emit(1);
        assert_eq!(call_count.get(), 1);

        handle.unsubscribe();

        stream.emit(2);
        assert_eq!(call_count.get(), 1);
    }

    #[test]
    fn test_multiple_subscribers() {
        let stream = StateStream::new(0);
        let count1 = Rc::new(Cell::new(0));
        let count2 = Rc::new(Cell::new(0));

        let count1_clone = Rc::clone(&count1);
        let count2_clone = Rc::clone(&count2);

        let _h1 = stream.subscribe(move |_, _| count1_clone.set(count1_clone.get() + 1));
        let _h2 = stream.subscribe(move |_, _| count2_clone.set(count2_clone.get() + 1));

        assert_eq!(stream.subscriber_count(), 2);

        stream.emit(42);
        assert_eq!(count1.get(), 1);
        assert_eq!(count2.get(), 1);
    }

    #[test]
    fn test_update_function() {
        let stream = StateStream::new(10);
        
        stream.update(|n| n * 2);
        assert_eq!(stream.state(), 20);

        stream.update(|n| n + 5);
        assert_eq!(stream.state(), 25);
    }

    #[test]
    fn test_detach_subscription() {
        let stream = StateStream::new(0);
        let call_count = Rc::new(Cell::new(0));

        let call_count_clone = Rc::clone(&call_count);
        let handle = stream.subscribe(move |_, _| {
            call_count_clone.set(call_count_clone.get() + 1);
        });

        handle.detach(); // Don't unsubscribe on drop

        stream.emit(1);
        assert_eq!(call_count.get(), 1);

        stream.emit(2);
        assert_eq!(call_count.get(), 2); // Still subscribed!
    }
}
