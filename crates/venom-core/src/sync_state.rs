//! Thread-Safe State Management
//!
//! This module provides thread-safe (Send + Sync) versions of state management
//! utilities for use in multi-threaded contexts and Widget integration.
//!
//! # Components
//!
//! - [`SyncStateStream`] - Thread-safe observable state container
//! - [`SyncCubitCore`] - Thread-safe cubit core
//! - [`SyncBloc`] - Thread-safe bloc
//!
//! # When to Use
//!
//! Use these types when you need:
//! - Integration with venom-widgets (Widget trait requires Send + Sync)
//! - Multi-threaded state access
//! - State sharing across threads
//!
//! For single-threaded use, prefer the regular `CubitCore` and `StateStream`
//! for better performance.
//!
//! # Example
//!
//! ```
//! use venom_core::{SyncCubitCore, SyncCubit};
//!
//! // Create a thread-safe cubit
//! let counter = SyncCubitCore::new(0);
//!
//! // Clone and share across threads
//! let counter_clone = counter.clone();
//! std::thread::spawn(move || {
//!     counter_clone.emit(42);
//! });
//! ```

// Allow unsafe code for Send+Sync implementations.
// SAFETY: All types use Arc<RwLock<T>> which provides synchronized access.
// The manual Send/Sync impls are needed because Rust can't auto-derive them
// when there are trait object callbacks involved.
#![allow(unsafe_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};

// ============================================================================
// SYNC SUBSCRIPTION ID
// ============================================================================

/// Thread-safe subscription identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncSubscriptionId(u64);

impl SyncSubscriptionId {
    /// Create a new unique subscription ID
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Get the raw ID value
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for SyncSubscriptionId {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SYNC SUBSCRIPTION HANDLE
// ============================================================================

/// Thread-safe RAII handle for managing subscriptions
///
/// When dropped, automatically unsubscribes from the source.
pub struct SyncSubscriptionHandle {
    id: SyncSubscriptionId,
    unsubscribe: Option<Arc<dyn Fn(SyncSubscriptionId) + Send + Sync>>,
}

impl SyncSubscriptionHandle {
    /// Create a new subscription handle
    pub fn new<F>(id: SyncSubscriptionId, unsubscribe: F) -> Self
    where
        F: Fn(SyncSubscriptionId) + Send + Sync + 'static,
    {
        Self {
            id,
            unsubscribe: Some(Arc::new(unsubscribe)),
        }
    }

    /// Create a detached handle (won't auto-unsubscribe)
    pub fn detached(id: SyncSubscriptionId) -> Self {
        Self {
            id,
            unsubscribe: None,
        }
    }

    /// Get the subscription ID
    pub fn id(&self) -> SyncSubscriptionId {
        self.id
    }

    /// Manually unsubscribe
    pub fn unsubscribe(&mut self) {
        if let Some(unsub) = self.unsubscribe.take() {
            unsub(self.id);
        }
    }

    /// Detach the handle (prevent auto-unsubscribe on drop)
    pub fn detach(&mut self) {
        self.unsubscribe = None;
    }
}

impl Drop for SyncSubscriptionHandle {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

// ============================================================================
// SYNC STATE STREAM
// ============================================================================

type SyncCallback<S> = Arc<dyn Fn(&S, &S) + Send + Sync>;

struct SyncStreamInner<S> {
    state: S,
    version: u64,
    subscribers: HashMap<SyncSubscriptionId, SyncCallback<S>>,
}

/// Thread-safe observable state container
///
/// Uses `Arc<RwLock>` for interior mutability across threads.
///
/// # Example
///
/// ```
/// use venom_core::SyncStateStream;
/// use std::sync::Arc;
///
/// let stream = SyncStateStream::new(0);
/// let stream_clone = stream.clone();
///
/// // Safe to use across threads
/// std::thread::spawn(move || {
///     stream_clone.emit(42);
/// });
/// ```
pub struct SyncStateStream<S> {
    inner: Arc<RwLock<SyncStreamInner<S>>>,
}

impl<S> SyncStateStream<S> {
    /// Create a new sync state stream
    pub fn new(initial: S) -> Self {
        Self {
            inner: Arc::new(RwLock::new(SyncStreamInner {
                state: initial,
                version: 0,
                subscribers: HashMap::new(),
            })),
        }
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.inner.read().unwrap().version
    }

    /// Get the subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.inner.read().unwrap().subscribers.len()
    }
}

impl<S: Clone> SyncStateStream<S> {
    /// Get a clone of the current state
    pub fn state(&self) -> S {
        self.inner.read().unwrap().state.clone()
    }

    /// Execute a function with a reference to the current state
    pub fn with_state<R, F: FnOnce(&S) -> R>(&self, f: F) -> R {
        let guard = self.inner.read().unwrap();
        f(&guard.state)
    }
}

impl<S: Clone + PartialEq> SyncStateStream<S> {
    /// Emit a new state
    ///
    /// Returns true if the state actually changed.
    pub fn emit(&self, new_state: S) -> bool {
        let mut guard = self.inner.write().unwrap();
        
        if guard.state == new_state {
            return false;
        }

        let old_state = guard.state.clone();
        guard.state = new_state.clone();
        guard.version += 1;

        // Collect subscribers to avoid holding lock during callbacks
        let subscribers: Vec<_> = guard.subscribers.values().cloned().collect();
        drop(guard);

        // Notify subscribers outside the lock
        for callback in subscribers {
            callback(&old_state, &new_state);
        }

        true
    }

    /// Update state with a function
    pub fn update<F: FnOnce(&S) -> S>(&self, f: F) -> bool {
        let new_state = {
            let guard = self.inner.read().unwrap();
            f(&guard.state)
        };
        self.emit(new_state)
    }
}

impl<S: Send + Sync + 'static> SyncStateStream<S> {
    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S, &S) + Send + Sync + 'static,
    {
        let id = SyncSubscriptionId::new();
        
        {
            let mut guard = self.inner.write().unwrap();
            guard.subscribers.insert(id, Arc::new(callback));
        }

        let inner = Arc::downgrade(&self.inner);
        SyncSubscriptionHandle::new(id, move |sub_id| {
            if let Some(inner) = inner.upgrade() {
                let mut guard = inner.write().unwrap();
                guard.subscribers.remove(&sub_id);
            }
        })
    }

    /// Subscribe to new state only
    pub fn subscribe_new<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        self.subscribe(move |_, new| callback(new))
    }
}

impl<S> Clone for SyncStateStream<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for SyncStateStream<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.inner.read().unwrap();
        f.debug_struct("SyncStateStream")
            .field("state", &guard.state)
            .field("version", &guard.version)
            .field("subscribers", &guard.subscribers.len())
            .finish()
    }
}

// Implement Send + Sync for SyncStateStream
unsafe impl<S: Send> Send for SyncStateStream<S> {}
unsafe impl<S: Send + Sync> Sync for SyncStateStream<S> {}

// ============================================================================
// SYNC CUBIT CORE
// ============================================================================

/// Thread-safe cubit core
///
/// Drop-in replacement for `CubitCore` that implements `Send + Sync`.
///
/// # Example
///
/// ```
/// use venom_core::SyncCubitCore;
///
/// let cubit = SyncCubitCore::new(0);
/// cubit.emit(42);
/// assert_eq!(cubit.state(), 42);
///
/// // Safe to clone and share across threads
/// let cubit_clone = cubit.clone();
/// ```
pub struct SyncCubitCore<S> {
    stream: SyncStateStream<S>,
    name: Option<String>,
}

impl<S> SyncCubitCore<S> {
    /// Create a new sync cubit
    pub fn new(initial: S) -> Self {
        Self {
            stream: SyncStateStream::new(initial),
            name: None,
        }
    }

    /// Create with a name for debugging
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            stream: SyncStateStream::new(initial),
            name: Some(name.into()),
        }
    }

    /// Get the cubit's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the current version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.stream.subscriber_count()
    }
}

impl<S: Clone> SyncCubitCore<S> {
    /// Get a clone of the current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Execute a function with a reference to the current state
    pub fn with_state<R, F: FnOnce(&S) -> R>(&self, f: F) -> R {
        self.stream.with_state(f)
    }
}

impl<S: Clone + PartialEq> SyncCubitCore<S> {
    /// Emit a new state
    pub fn emit(&self, state: S) -> bool {
        self.stream.emit(state)
    }

    /// Update state with a function
    pub fn update<F: FnOnce(&S) -> S>(&self, f: F) -> bool {
        self.stream.update(f)
    }

    /// Reset state
    pub fn reset(&self, state: S) -> bool {
        self.emit(state)
    }
}

impl<S: Clone + PartialEq + Default> SyncCubitCore<S> {
    /// Reset to default state
    pub fn reset_to_default(&self) -> bool {
        self.emit(S::default())
    }
}

impl<S: Send + Sync + 'static> SyncCubitCore<S> {
    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S, &S) + Send + Sync + 'static,
    {
        self.stream.subscribe(callback)
    }

    /// Subscribe to new state only
    pub fn subscribe_new<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        self.stream.subscribe_new(callback)
    }
}

impl<S> Clone for SyncCubitCore<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            name: self.name.clone(),
        }
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for SyncCubitCore<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncCubitCore")
            .field("name", &self.name)
            .field("stream", &self.stream)
            .finish()
    }
}

impl<S: Default> Default for SyncCubitCore<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// Implement Send + Sync
unsafe impl<S: Send> Send for SyncCubitCore<S> {}
unsafe impl<S: Send + Sync> Sync for SyncCubitCore<S> {}

// ============================================================================
// SYNC CUBIT TRAIT
// ============================================================================

/// Trait for thread-safe cubits
pub trait SyncCubit: Send + Sync {
    /// The state type
    type State: Clone + PartialEq + Send + Sync + 'static;

    /// Get a reference to the cubit core
    fn core(&self) -> &SyncCubitCore<Self::State>;

    /// Get the current state
    fn state(&self) -> Self::State {
        self.core().state()
    }

    /// Get version
    fn version(&self) -> u64 {
        self.core().version()
    }

    /// Emit a new state
    fn emit(&self, state: Self::State) -> bool {
        self.core().emit(state)
    }

    /// Update state
    fn update<F: FnOnce(&Self::State) -> Self::State>(&self, f: F) -> bool {
        self.core().update(f)
    }

    /// Subscribe to changes
    fn subscribe<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&Self::State, &Self::State) + Send + Sync + 'static,
    {
        self.core().subscribe(callback)
    }
}

// ============================================================================
// SYNC SIMPLE CUBIT
// ============================================================================

/// Thread-safe simple cubit
///
/// # Example
///
/// ```
/// use venom_core::{SyncSimpleCubit, SyncCubit};
///
/// let cubit = SyncSimpleCubit::new(0);
/// cubit.emit(42);
/// assert_eq!(cubit.state(), 42);
/// ```
#[derive(Clone)]
pub struct SyncSimpleCubit<S> {
    core: SyncCubitCore<S>,
}

impl<S> SyncSimpleCubit<S> {
    /// Create a new simple cubit
    pub fn new(initial: S) -> Self {
        Self {
            core: SyncCubitCore::new(initial),
        }
    }

    /// Create with name
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            core: SyncCubitCore::named(name, initial),
        }
    }
}

impl<S: Clone + PartialEq + Send + Sync + 'static> SyncCubit for SyncSimpleCubit<S> {
    type State = S;

    fn core(&self) -> &SyncCubitCore<Self::State> {
        &self.core
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for SyncSimpleCubit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncSimpleCubit")
            .field("core", &self.core)
            .finish()
    }
}

impl<S: Default> Default for SyncSimpleCubit<S> {
    fn default() -> Self {
        Self::new(S::default())
    }
}

// Send + Sync are automatically derived from SyncCubitCore

// ============================================================================
// SYNC BLOC
// ============================================================================

/// Thread-safe bloc for event-driven state management
///
/// # Example
///
/// ```
/// use venom_core::SyncBloc;
///
/// #[derive(Clone)]
/// enum Event { Increment, Decrement }
///
/// let bloc = SyncBloc::new(0, |event: Event, state, emit| {
///     match event {
///         Event::Increment => emit(state + 1),
///         Event::Decrement => emit(state - 1),
///     }
/// });
///
/// bloc.add(Event::Increment);
/// assert_eq!(bloc.state(), 1);
/// ```
pub struct SyncBloc<E, S> {
    stream: SyncStateStream<S>,
    handler: Arc<dyn Fn(E, S, &dyn Fn(S)) + Send + Sync>,
    name: Option<String>,
}

impl<E: Send + 'static, S: Clone + PartialEq + Send + Sync + 'static> SyncBloc<E, S> {
    /// Create a new sync bloc
    pub fn new<H>(initial: S, handler: H) -> Self
    where
        H: Fn(E, S, &dyn Fn(S)) + Send + Sync + 'static,
    {
        Self {
            stream: SyncStateStream::new(initial),
            handler: Arc::new(handler),
            name: None,
        }
    }

    /// Create with name
    pub fn named<H>(name: impl Into<String>, initial: S, handler: H) -> Self
    where
        H: Fn(E, S, &dyn Fn(S)) + Send + Sync + 'static,
    {
        let mut bloc = Self::new(initial, handler);
        bloc.name = Some(name.into());
        bloc
    }

    /// Get name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Add an event
    pub fn add(&self, event: E) {
        let current = self.stream.state();
        let stream = self.stream.clone();
        let emit = move |state: S| {
            stream.emit(state);
        };
        (self.handler)(event, current, &emit);
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S, &S) + Send + Sync + 'static,
    {
        self.stream.subscribe(callback)
    }

    /// Subscribe to new state only
    pub fn subscribe_new<F>(&self, callback: F) -> SyncSubscriptionHandle
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        self.stream.subscribe_new(callback)
    }
}

impl<E, S: std::fmt::Debug> std::fmt::Debug for SyncBloc<E, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncBloc")
            .field("name", &self.name)
            .field("stream", &self.stream)
            .finish()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering as AtomicOrdering};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_sync_state_stream_basic() {
        let stream = SyncStateStream::new(0);
        assert_eq!(stream.state(), 0);

        stream.emit(42);
        assert_eq!(stream.state(), 42);
    }

    #[test]
    fn test_sync_state_stream_subscribe() {
        let stream = SyncStateStream::new(0);
        let call_count = Arc::new(AtomicI32::new(0));
        let count_clone = Arc::clone(&call_count);

        let _handle = stream.subscribe(move |_, _| {
            count_clone.fetch_add(1, AtomicOrdering::SeqCst);
        });

        stream.emit(1);
        stream.emit(2);
        stream.emit(3);

        assert_eq!(call_count.load(AtomicOrdering::SeqCst), 3);
    }

    #[test]
    fn test_sync_state_stream_multithread() {
        let stream = SyncStateStream::new(0);
        let stream_clone = stream.clone();

        let handle = thread::spawn(move || {
            for i in 1..=10 {
                stream_clone.emit(i);
                thread::sleep(Duration::from_millis(1));
            }
        });

        handle.join().unwrap();
        assert_eq!(stream.state(), 10);
    }

    #[test]
    fn test_sync_cubit_core_basic() {
        let cubit = SyncCubitCore::new(0);
        assert_eq!(cubit.state(), 0);

        cubit.emit(42);
        assert_eq!(cubit.state(), 42);
    }

    #[test]
    fn test_sync_cubit_core_update() {
        let cubit = SyncCubitCore::new(10);
        cubit.update(|n| n * 2);
        assert_eq!(cubit.state(), 20);
    }

    #[test]
    fn test_sync_cubit_core_multithread() {
        let cubit = SyncCubitCore::new(0);
        let cubit_clone = cubit.clone();

        let handle = thread::spawn(move || {
            cubit_clone.emit(100);
        });

        handle.join().unwrap();
        assert_eq!(cubit.state(), 100);
    }

    #[test]
    fn test_sync_simple_cubit() {
        let cubit = SyncSimpleCubit::new(0);
        cubit.emit(42);
        assert_eq!(cubit.state(), 42);
    }

    #[test]
    fn test_sync_bloc() {
        #[derive(Clone)]
        enum CounterEvent {
            Increment,
            Decrement,
            Add(i32),
        }

        let bloc = SyncBloc::new(0, |event, state, emit| {
            match event {
                CounterEvent::Increment => emit(state + 1),
                CounterEvent::Decrement => emit(state - 1),
                CounterEvent::Add(n) => emit(state + n),
            }
        });

        bloc.add(CounterEvent::Increment);
        assert_eq!(bloc.state(), 1);

        bloc.add(CounterEvent::Add(10));
        assert_eq!(bloc.state(), 11);

        bloc.add(CounterEvent::Decrement);
        assert_eq!(bloc.state(), 10);
    }

    #[test]
    fn test_sync_unsubscribe_on_drop() {
        let stream = SyncStateStream::new(0);
        
        {
            let _handle = stream.subscribe(|_, _| {});
            assert_eq!(stream.subscriber_count(), 1);
        } // handle dropped here

        assert_eq!(stream.subscriber_count(), 0);
    }

    #[test]
    fn test_sync_subscription_id_unique() {
        let id1 = SyncSubscriptionId::new();
        let id2 = SyncSubscriptionId::new();
        assert_ne!(id1, id2);
    }
}
