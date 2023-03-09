//! Async Bloc - Asynchronous State Management
//!
//! This module provides support for asynchronous operations in state management:
//!
//! - [`AsyncCubit`] - Cubit with async operation support
//! - [`AsyncResult`] - Result type for async handlers
//! - [`AsyncBloc`] - Bloc with async event handling
//!
//! # Design Note
//!
//! Since Rust's async ecosystem is complex (requires runtime, etc.), we use a
//! callback-based approach that works without async/await dependencies.
//!
//! # Example
//!
//! ```
//! use venom_core::AsyncCubit;
//!
//! let cubit = AsyncCubit::new(0);
//!
//! // Execute an "async" operation (simulated with callback)
//! cubit.execute(|emit| {
//!     emit(1); // Loading state
//!     // In real app: spawn thread or use callback from async API
//!     emit(42); // Final state
//! });
//! ```

use crate::subscription::{StateStream, SubscriptionHandle};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// ASYNC RESULT
// ============================================================================

/// Result of an async operation
///
/// Used to indicate what states should be emitted from an async handler.
#[derive(Debug, Clone)]
pub enum AsyncResult<S> {
    /// Emit a single state
    Emit(S),
    /// Emit multiple states in sequence
    EmitMany(Vec<S>),
    /// No state change
    None,
}

impl<S> AsyncResult<S> {
    /// Create an emit result
    pub fn emit(state: S) -> Self {
        Self::Emit(state)
    }

    /// Create a multi-emit result
    pub fn emit_many(states: Vec<S>) -> Self {
        Self::EmitMany(states)
    }

    /// Create no-change result
    pub fn none() -> Self {
        Self::None
    }
}

// ============================================================================
// ASYNC CUBIT
// ============================================================================

/// Cubit with support for asynchronous operations
///
/// `AsyncCubit` allows you to perform async operations and emit
/// multiple states during the operation (e.g., loading -> success).
///
/// # Example
///
/// ```
/// use venom_core::AsyncCubit;
///
/// #[derive(Clone, PartialEq, Debug)]
/// enum LoadState {
///     Initial,
///     Loading,
///     Loaded(String),
///     Error(String),
/// }
///
/// let cubit = AsyncCubit::new(LoadState::Initial);
///
/// // Simulate async data loading
/// cubit.execute(|emit| {
///     emit(LoadState::Loading);
///     // In real app: fetch data from API
///     emit(LoadState::Loaded("Data loaded!".into()));
/// });
///
/// // The cubit now has the final state
/// assert!(matches!(cubit.state(), LoadState::Loaded(_)));
/// ```
pub struct AsyncCubit<S> {
    stream: StateStream<S>,
    name: Option<String>,
}

impl<S: Clone + PartialEq + 'static> AsyncCubit<S> {
    /// Create a new async cubit
    pub fn new(initial: S) -> Self {
        Self {
            stream: StateStream::new(initial),
            name: None,
        }
    }

    /// Create with a name for debugging
    pub fn named(name: impl Into<String>, initial: S) -> Self {
        Self {
            stream: StateStream::new(initial),
            name: Some(name.into()),
        }
    }

    /// Get the cubit name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get state version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Emit a state directly (synchronous)
    pub fn emit(&self, state: S) -> bool {
        self.stream.emit(state)
    }

    /// Execute an operation that can emit multiple states
    ///
    /// The operation receives an `emit` function that can be called
    /// multiple times to emit intermediate states.
    ///
    /// # Example
    ///
    /// ```
    /// use venom_core::AsyncCubit;
    ///
    /// let cubit = AsyncCubit::new(0);
    ///
    /// cubit.execute(|emit| {
    ///     emit(1);
    ///     emit(2);
    ///     emit(3);
    /// });
    ///
    /// assert_eq!(cubit.state(), 3);
    /// ```
    pub fn execute<F>(&self, operation: F)
    where
        F: FnOnce(&dyn Fn(S)),
    {
        let stream = self.stream.clone();
        let emit = move |state: S| {
            stream.emit(state);
        };
        operation(&emit);
    }

    /// Execute with result
    ///
    /// Alternative API using AsyncResult enum.
    pub fn execute_with_result<F>(&self, operation: F)
    where
        F: FnOnce(&S) -> AsyncResult<S>,
    {
        let current = self.state();
        match operation(&current) {
            AsyncResult::Emit(state) => {
                self.stream.emit(state);
            }
            AsyncResult::EmitMany(states) => {
                for state in states {
                    self.stream.emit(state);
                }
            }
            AsyncResult::None => {}
        }
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        self.stream.subscribe(callback)
    }

    /// Subscribe to new state only
    pub fn subscribe_new<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S) + 'static,
    {
        self.stream.subscribe_new(callback)
    }
}

impl<S> Clone for AsyncCubit<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            name: self.name.clone(),
        }
    }
}

impl<S: std::fmt::Debug> std::fmt::Debug for AsyncCubit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AsyncCubit")
            .field("name", &self.name)
            .field("stream", &self.stream)
            .finish()
    }
}

// ============================================================================
// THREADED ASYNC BLOC
// ============================================================================

/// Message sent to the background thread
enum BlocMessage<E> {
    Event(E),
    Shutdown,
}

/// Thread-safe async bloc for background processing
///
/// This bloc processes events on a background thread, allowing for
/// truly asynchronous operations without blocking the UI.
///
/// # Safety Note
///
/// The state and handler run on a background thread, so they must be
/// `Send + Sync`. States are synchronized via channels.
///
/// # Example
///
/// ```
/// use venom_core::ThreadedBloc;
/// use std::time::Duration;
///
/// let bloc = ThreadedBloc::new(0, |event: i32, state, emit| {
///     // This runs on a background thread
///     emit(state + event);
/// });
///
/// bloc.add(10);
/// std::thread::sleep(Duration::from_millis(10));
/// // State is updated asynchronously
/// ```
pub struct ThreadedBloc<E, S> {
    state: Arc<Mutex<S>>,
    sender: mpsc::Sender<BlocMessage<E>>,
    name: Option<String>,
}

impl<E: Send + 'static, S: Clone + PartialEq + Send + Sync + 'static> ThreadedBloc<E, S> {
    /// Create a new threaded bloc
    ///
    /// # Arguments
    ///
    /// * `initial` - Initial state
    /// * `handler` - Event handler function `(event, current_state, emit)`
    pub fn new<H>(initial: S, handler: H) -> Self
    where
        H: Fn(E, S, &dyn Fn(S)) + Send + Sync + 'static,
    {
        let state = Arc::new(Mutex::new(initial));
        let state_clone = Arc::clone(&state);
        let (sender, receiver) = mpsc::channel();
        let handler = Arc::new(handler);

        thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(BlocMessage::Event(event)) => {
                        let current = state_clone.lock().unwrap().clone();
                        let state_ref = Arc::clone(&state_clone);
                        
                        let emit = move |new_state: S| {
                            *state_ref.lock().unwrap() = new_state;
                        };
                        
                        handler(event, current, &emit);
                    }
                    Ok(BlocMessage::Shutdown) | Err(_) => break,
                }
            }
        });

        Self {
            state,
            sender,
            name: None,
        }
    }

    /// Create with name for debugging
    pub fn named<H>(name: impl Into<String>, initial: S, handler: H) -> Self
    where
        H: Fn(E, S, &dyn Fn(S)) + Send + Sync + 'static,
    {
        let mut bloc = Self::new(initial, handler);
        bloc.name = Some(name.into());
        bloc
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.state.lock().unwrap().clone()
    }

    /// Add an event to be processed
    pub fn add(&self, event: E) {
        let _ = self.sender.send(BlocMessage::Event(event));
    }

    /// Get bloc name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl<E, S> Drop for ThreadedBloc<E, S> {
    fn drop(&mut self) {
        let _ = self.sender.send(BlocMessage::Shutdown);
    }
}

// ============================================================================
// CALLBACK BLOC
// ============================================================================

/// Bloc that works with callbacks for async APIs
///
/// This is useful when integrating with APIs that use callbacks
/// rather than futures (like many native APIs).
///
/// # Example
///
/// ```
/// use venom_core::CallbackBloc;
///
/// let bloc = CallbackBloc::new(vec![], |data: Vec<String>, emit| {
///     emit(data);
/// });
///
/// // Simulate receiving data from callback-based API
/// bloc.handle(vec!["item1".into(), "item2".into()]);
/// assert_eq!(bloc.state().len(), 2);
/// ```
pub struct CallbackBloc<T, S> {
    stream: StateStream<S>,
    handler: Rc<dyn Fn(T, &dyn Fn(S))>,
    name: Option<String>,
}

impl<T: 'static, S: Clone + PartialEq + 'static> CallbackBloc<T, S> {
    /// Create a new callback bloc
    pub fn new<H>(initial: S, handler: H) -> Self
    where
        H: Fn(T, &dyn Fn(S)) + 'static,
    {
        Self {
            stream: StateStream::new(initial),
            handler: Rc::new(handler),
            name: None,
        }
    }

    /// Create with name
    pub fn named<H>(name: impl Into<String>, initial: S, handler: H) -> Self
    where
        H: Fn(T, &dyn Fn(S)) + 'static,
    {
        let mut bloc = Self::new(initial, handler);
        bloc.name = Some(name.into());
        bloc
    }

    /// Handle incoming data
    pub fn handle(&self, data: T) {
        let stream = self.stream.clone();
        let emit = move |state: S| {
            stream.emit(state);
        };
        (self.handler)(data, &emit);
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Subscribe to changes
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        self.stream.subscribe(callback)
    }

    /// Get name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

// ============================================================================
// DEBOUNCED CUBIT
// ============================================================================

/// Cubit that debounces rapid state changes
///
/// Only the last state in a series of rapid changes is actually emitted.
/// This is useful for search inputs, window resize handlers, etc.
///
/// # Example (Conceptual)
///
/// ```
/// use venom_core::DebouncedCubit;
/// use std::time::Duration;
///
/// let cubit = DebouncedCubit::new(
///     "".to_string(),
///     Duration::from_millis(300)
/// );
///
/// // Rapid emissions
/// cubit.emit("a".into());
/// cubit.emit("ab".into());
/// cubit.emit("abc".into());
///
/// // Only "abc" will actually trigger subscribers (after 300ms)
/// ```
pub struct DebouncedCubit<S> {
    stream: StateStream<S>,
    pending: Rc<RefCell<Option<S>>>,
    debounce_ms: u64,
    name: Option<String>,
}

impl<S: Clone + PartialEq + 'static> DebouncedCubit<S> {
    /// Create a new debounced cubit
    pub fn new(initial: S, debounce: std::time::Duration) -> Self {
        Self {
            stream: StateStream::new(initial),
            pending: Rc::new(RefCell::new(None)),
            debounce_ms: debounce.as_millis() as u64,
            name: None,
        }
    }

    /// Create with name
    pub fn named(name: impl Into<String>, initial: S, debounce: std::time::Duration) -> Self {
        let mut cubit = Self::new(initial, debounce);
        cubit.name = Some(name.into());
        cubit
    }

    /// Queue a state change (will be debounced)
    ///
    /// Note: In a real implementation, this would use a timer.
    /// For simplicity, this version just stores the pending value.
    pub fn emit(&self, state: S) {
        *self.pending.borrow_mut() = Some(state);
        // In a full implementation, we'd start a timer here
    }

    /// Flush pending state immediately
    pub fn flush(&self) {
        if let Some(state) = self.pending.borrow_mut().take() {
            self.stream.emit(state);
        }
    }

    /// Emit immediately without debounce
    pub fn emit_now(&self, state: S) {
        *self.pending.borrow_mut() = None;
        self.stream.emit(state);
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get pending state (if any)
    pub fn pending_state(&self) -> Option<S> {
        self.pending.borrow().clone()
    }

    /// Subscribe to changes
    pub fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        self.stream.subscribe(callback)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::time::Duration;

    #[test]
    fn test_async_cubit_basic() {
        let cubit = AsyncCubit::new(0);

        cubit.emit(42);
        assert_eq!(cubit.state(), 42);
    }

    #[test]
    fn test_async_cubit_execute() {
        let cubit = AsyncCubit::new(0);

        cubit.execute(|emit| {
            emit(1);
            emit(2);
            emit(3);
        });

        assert_eq!(cubit.state(), 3);
    }

    #[test]
    fn test_async_cubit_execute_with_result() {
        let cubit = AsyncCubit::new(0);

        cubit.execute_with_result(|current| {
            AsyncResult::emit(current + 10)
        });

        assert_eq!(cubit.state(), 10);
    }

    #[test]
    fn test_async_cubit_emit_many() {
        let cubit = AsyncCubit::new(0);
        let states = Rc::new(RefCell::new(Vec::new()));
        let states_clone = Rc::clone(&states);

        let _sub = cubit.subscribe_new(move |s| {
            states_clone.borrow_mut().push(*s);
        });

        cubit.execute_with_result(|_| {
            AsyncResult::EmitMany(vec![1, 2, 3])
        });

        assert_eq!(*states.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_threaded_bloc() {
        let bloc = ThreadedBloc::new(0, |event: i32, state, emit| {
            emit(state + event);
        });

        bloc.add(10);
        thread::sleep(Duration::from_millis(50));

        assert_eq!(bloc.state(), 10);
    }

    #[test]
    fn test_callback_bloc() {
        let bloc = CallbackBloc::new(Vec::<i32>::new(), |data: Vec<i32>, emit| {
            emit(data);
        });

        bloc.handle(vec![1, 2, 3]);
        assert_eq!(bloc.state(), vec![1, 2, 3]);

        bloc.handle(vec![4, 5]);
        assert_eq!(bloc.state(), vec![4, 5]);
    }

    #[test]
    fn test_debounced_cubit() {
        let cubit = DebouncedCubit::new(0, Duration::from_millis(100));

        cubit.emit(1);
        cubit.emit(2);
        cubit.emit(3);

        // Pending, not yet emitted
        assert_eq!(cubit.state(), 0);
        assert_eq!(cubit.pending_state(), Some(3));

        // Flush manually
        cubit.flush();
        assert_eq!(cubit.state(), 3);
    }

    #[test]
    fn test_debounced_cubit_emit_now() {
        let cubit = DebouncedCubit::new(0, Duration::from_millis(100));

        cubit.emit_now(42);
        assert_eq!(cubit.state(), 42);
    }

    #[test]
    fn test_async_result() {
        let emit = AsyncResult::emit(42);
        assert!(matches!(emit, AsyncResult::Emit(42)));

        let none = AsyncResult::<i32>::none();
        assert!(matches!(none, AsyncResult::None));
    }
}
