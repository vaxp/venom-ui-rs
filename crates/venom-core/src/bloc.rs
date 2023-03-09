//! Bloc - Event-driven State Management
//!
//! Bloc (Business Logic Component) is an advanced state management solution
//! for complex event-driven scenarios.
//!
//! # When to use Bloc vs Cubit
//!
//! | Feature | Cubit | Bloc |
//! |---------|-------|------|
//! | Simple state updates | ✅ Best choice | Overkill |
//! | Complex event handling | Limited | ✅ Best choice |
//! | Event history/logging | No | ✅ Yes |
//! | Event transformation | No | ✅ Yes |
//! | Learning curve | Low | Medium |
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │                         Bloc                             │
//! │                                                          │
//! │   ┌─────────┐     ┌───────────┐     ┌────────────────┐  │
//! │   │  Event  │────▶│ on_event  │────▶│ emit(state)    │  │
//! │   └─────────┘     └───────────┘     └───────┬────────┘  │
//! │        ▲                                    │            │
//! │        │                                    ▼            │
//! │   ┌─────────┐                       ┌────────────────┐  │
//! │   │  add()  │                       │  StateStream   │  │
//! │   └─────────┘                       └───────┬────────┘  │
//! │                                             │            │
//! └─────────────────────────────────────────────┼────────────┘
//!                                               │
//!                                               ▼
//!                                      ┌─────────────────┐
//!                                      │   Subscribers   │
//!                                      └─────────────────┘
//! ```
//!
//! # Example: Authentication Bloc
//!
//! ```
//! use venom_core::{BlocCore, SubscriptionHandle};
//!
//! // 1. Define Events
//! #[derive(Debug, Clone)]
//! enum AuthEvent {
//!     Login { email: String, password: String },
//!     Logout,
//!     CheckSession,
//! }
//!
//! // 2. Define States
//! #[derive(Debug, Clone, PartialEq, Default)]
//! enum AuthState {
//!     #[default]
//!     Initial,
//!     Loading,
//!     Authenticated { user_id: String },
//!     Unauthenticated,
//!     Error { message: String },
//! }
//!
//! // 3. Create Bloc
//! struct AuthBloc {
//!     core: BlocCore<AuthEvent, AuthState>,
//! }
//!
//! impl AuthBloc {
//!     fn new() -> Self {
//!         let core = BlocCore::new(AuthState::Initial, |event, emit| {
//!             match event {
//!                 AuthEvent::Login { email, password } => {
//!                     emit(AuthState::Loading);
//!                     // Simulate async login
//!                     if email == "user@test.com" && password == "password" {
//!                         emit(AuthState::Authenticated { user_id: "123".into() });
//!                     } else {
//!                         emit(AuthState::Error { message: "Invalid credentials".into() });
//!                     }
//!                 }
//!                 AuthEvent::Logout => {
//!                     emit(AuthState::Unauthenticated);
//!                 }
//!                 AuthEvent::CheckSession => {
//!                     emit(AuthState::Unauthenticated);
//!                 }
//!             }
//!         });
//!         Self { core }
//!     }
//!
//!     fn add(&self, event: AuthEvent) {
//!         self.core.add(event);
//!     }
//!
//!     fn state(&self) -> AuthState {
//!         self.core.state()
//!     }
//! }
//!
//! let auth = AuthBloc::new();
//! assert_eq!(auth.state(), AuthState::Initial);
//!
//! auth.add(AuthEvent::Login {
//!     email: "user@test.com".into(),
//!     password: "password".into(),
//! });
//!
//! assert!(matches!(auth.state(), AuthState::Authenticated { .. }));
//! ```

use crate::subscription::{StateStream, SubscriptionHandle};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

// ============================================================================
// BLOC CORE
// ============================================================================

/// Core implementation for Bloc pattern
///
/// `BlocCore` provides event-driven state management with:
/// - Event dispatching via `add()`
/// - Multiple state emissions per event
/// - Event queue for sequential processing
/// - Full subscription support
///
/// # Type Parameters
///
/// * `E` - The event type
/// * `S` - The state type, must implement `Clone + PartialEq`
pub struct BlocCore<E, S> {
    stream: StateStream<S>,
    handler: Rc<dyn Fn(E, &dyn Fn(S))>,
    event_queue: Rc<RefCell<VecDeque<E>>>,
    processing: Rc<RefCell<bool>>,
    /// Optional name for debugging
    name: Option<String>,
}

impl<E: 'static, S: Clone + PartialEq + 'static> BlocCore<E, S> {
    /// Create a new bloc with initial state and event handler
    ///
    /// # Arguments
    ///
    /// * `initial` - The initial state
    /// * `handler` - Function that maps events to state emissions
    ///
    /// # Example
    ///
    /// ```
    /// use venom_core::BlocCore;
    ///
    /// enum CounterEvent { Increment, Decrement }
    ///
    /// let bloc = BlocCore::new(0, |event, emit| {
    ///     // Note: We need access to current state somehow
    ///     match event {
    ///         CounterEvent::Increment => emit(1), // Simplified
    ///         CounterEvent::Decrement => emit(-1),
    ///     }
    /// });
    /// ```
    pub fn new<H>(initial: S, handler: H) -> Self
    where
        H: Fn(E, &dyn Fn(S)) + 'static,
    {
        Self {
            stream: StateStream::new(initial),
            handler: Rc::new(handler),
            event_queue: Rc::new(RefCell::new(VecDeque::new())),
            processing: Rc::new(RefCell::new(false)),
            name: None,
        }
    }

    /// Create a bloc with a name for debugging
    pub fn named<H>(name: impl Into<String>, initial: S, handler: H) -> Self
    where
        H: Fn(E, &dyn Fn(S)) + 'static,
    {
        let mut bloc = Self::new(initial, handler);
        bloc.name = Some(name.into());
        bloc
    }

    /// Get the bloc's name (if set)
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get the current state version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Add an event to be processed
    ///
    /// Events are processed sequentially in the order they are added.
    /// Each event can emit multiple states.
    pub fn add(&self, event: E) {
        self.event_queue.borrow_mut().push_back(event);
        self.process_events();
    }

    /// Process queued events
    fn process_events(&self) {
        // Prevent re-entrant processing
        if *self.processing.borrow() {
            return;
        }
        *self.processing.borrow_mut() = true;

        while let Some(event) = self.event_queue.borrow_mut().pop_front() {
            let stream = self.stream.clone();
            let emit = move |state: S| {
                stream.emit(state);
            };
            (self.handler)(event, &emit);
        }

        *self.processing.borrow_mut() = false;
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

    /// Get the number of subscribers
    pub fn subscriber_count(&self) -> usize {
        self.stream.subscriber_count()
    }

    /// Get the number of pending events
    pub fn pending_events(&self) -> usize {
        self.event_queue.borrow().len()
    }
}

impl<E, S: std::fmt::Debug> std::fmt::Debug for BlocCore<E, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocCore")
            .field("name", &self.name)
            .field("state", &self.stream)
            .field("pending_events", &self.event_queue.borrow().len())
            .finish()
    }
}

// ============================================================================
// BLOC WITH STATE ACCESS
// ============================================================================

/// Bloc with access to current state in event handler
///
/// This is a more powerful version of `BlocCore` that provides
/// the current state to the event handler, enabling state-based logic.
///
/// # Example
///
/// ```
/// use venom_core::StatefulBloc;
///
/// #[derive(Debug, Clone)]
/// enum CounterEvent { Increment, Decrement, Reset }
///
/// let bloc = StatefulBloc::new(0, |event, state, emit| {
///     match event {
///         CounterEvent::Increment => emit(*state + 1),
///         CounterEvent::Decrement => emit(*state - 1),
///         CounterEvent::Reset => emit(0),
///     }
/// });
///
/// bloc.add(CounterEvent::Increment);
/// bloc.add(CounterEvent::Increment);
/// assert_eq!(bloc.state(), 2);
///
/// bloc.add(CounterEvent::Decrement);
/// assert_eq!(bloc.state(), 1);
/// ```
pub struct StatefulBloc<E, S> {
    stream: StateStream<S>,
    handler: Rc<dyn Fn(E, &S, &dyn Fn(S))>,
    event_queue: Rc<RefCell<VecDeque<E>>>,
    processing: Rc<RefCell<bool>>,
    name: Option<String>,
}

impl<E: 'static, S: Clone + PartialEq + 'static> StatefulBloc<E, S> {
    /// Create a new stateful bloc
    ///
    /// # Arguments
    ///
    /// * `initial` - The initial state
    /// * `handler` - Function `(event, current_state, emit)` that handles events
    pub fn new<H>(initial: S, handler: H) -> Self
    where
        H: Fn(E, &S, &dyn Fn(S)) + 'static,
    {
        Self {
            stream: StateStream::new(initial),
            handler: Rc::new(handler),
            event_queue: Rc::new(RefCell::new(VecDeque::new())),
            processing: Rc::new(RefCell::new(false)),
            name: None,
        }
    }

    /// Create with a name for debugging
    pub fn named<H>(name: impl Into<String>, initial: S, handler: H) -> Self
    where
        H: Fn(E, &S, &dyn Fn(S)) + 'static,
    {
        let mut bloc = Self::new(initial, handler);
        bloc.name = Some(name.into());
        bloc
    }

    /// Get the bloc's name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Get the current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get the state version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Add an event
    pub fn add(&self, event: E) {
        self.event_queue.borrow_mut().push_back(event);
        self.process_events();
    }

    fn process_events(&self) {
        if *self.processing.borrow() {
            return;
        }
        *self.processing.borrow_mut() = true;

        while let Some(event) = self.event_queue.borrow_mut().pop_front() {
            let current_state = self.stream.state();
            let stream = self.stream.clone();
            let emit = move |state: S| {
                stream.emit(state);
            };
            (self.handler)(event, &current_state, &emit);
        }

        *self.processing.borrow_mut() = false;
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

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.stream.subscriber_count()
    }
}

impl<E, S: std::fmt::Debug> std::fmt::Debug for StatefulBloc<E, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatefulBloc")
            .field("name", &self.name)
            .field("state", &self.stream)
            .finish()
    }
}

// ============================================================================
// BLOC TRAIT
// ============================================================================

/// Trait for implementing typed blocs
///
/// Provides a structured interface for bloc implementations.
pub trait Bloc {
    /// The event type
    type Event;
    /// The state type
    type State: Clone + PartialEq;

    /// Get the current state
    fn state(&self) -> Self::State;

    /// Add an event to be processed
    fn add(&self, event: Self::Event);

    /// Subscribe to state changes
    fn subscribe<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&Self::State, &Self::State) + 'static;
}

// ============================================================================
// EVENT TRANSFORMER
// ============================================================================

/// Transform events before they reach the bloc
///
/// Useful for:
/// - Debouncing rapid events
/// - Throttling
/// - Mapping events
///
/// # Example
///
/// ```
/// use venom_core::EventTransformer;
///
/// // Debounce search events
/// let transformer: EventTransformer<String> = EventTransformer::new(|events: Vec<String>| {
///     // Return only the last event (simplified debounce)
///     events.last().cloned()
/// });
/// ```
pub struct EventTransformer<E> {
    transform: Box<dyn Fn(Vec<E>) -> Option<E>>,
}

impl<E> EventTransformer<E> {
    /// Create a new event transformer
    pub fn new<F>(transform: F) -> Self
    where
        F: Fn(Vec<E>) -> Option<E> + 'static,
    {
        Self {
            transform: Box::new(transform),
        }
    }

    /// Identity transformer (no transformation)
    pub fn identity() -> Self
    where
        E: 'static,
    {
        Self::new(|events| events.into_iter().last())
    }

    /// Take only the first event, drop the rest
    pub fn take_first() -> Self
    where
        E: 'static,
    {
        Self::new(|events| events.into_iter().next())
    }

    /// Take only the last event (simple debounce)
    pub fn take_last() -> Self
    where
        E: 'static,
    {
        Self::new(|events| events.into_iter().last())
    }

    /// Apply the transformation
    pub fn apply(&self, events: Vec<E>) -> Option<E> {
        (self.transform)(events)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[derive(Debug, Clone)]
    enum CounterEvent {
        Increment,
        Decrement,
        Add(i32),
        Reset,
    }

    #[test]
    fn test_stateful_bloc_basic() {
        let bloc = StatefulBloc::new(0, |event, state, emit| {
            match event {
                CounterEvent::Increment => emit(state + 1),
                CounterEvent::Decrement => emit(state - 1),
                CounterEvent::Add(n) => emit(state + n),
                CounterEvent::Reset => emit(0),
            }
        });

        assert_eq!(bloc.state(), 0);

        bloc.add(CounterEvent::Increment);
        assert_eq!(bloc.state(), 1);

        bloc.add(CounterEvent::Add(10));
        assert_eq!(bloc.state(), 11);

        bloc.add(CounterEvent::Decrement);
        assert_eq!(bloc.state(), 10);

        bloc.add(CounterEvent::Reset);
        assert_eq!(bloc.state(), 0);
    }

    #[test]
    fn test_bloc_subscription() {
        let bloc = StatefulBloc::new(0, |event: CounterEvent, state, emit| {
            match event {
                CounterEvent::Increment => emit(state + 1),
                _ => {}
            }
        });

        let call_count = Rc::new(Cell::new(0));
        let count_clone = Rc::clone(&call_count);

        let _handle = bloc.subscribe(move |_, _| {
            count_clone.set(count_clone.get() + 1);
        });

        bloc.add(CounterEvent::Increment);
        bloc.add(CounterEvent::Increment);
        bloc.add(CounterEvent::Increment);

        assert_eq!(call_count.get(), 3);
    }

    #[test]
    fn test_bloc_multiple_emissions() {
        let states = Rc::new(RefCell::new(Vec::new()));
        let states_clone = Rc::clone(&states);

        let bloc = StatefulBloc::new(0, |event: i32, _, emit| {
            // Emit multiple states per event
            for i in 1..=event {
                emit(i);
            }
        });

        let _handle = bloc.subscribe_new(move |state| {
            states_clone.borrow_mut().push(*state);
        });

        bloc.add(3); // Should emit 1, 2, 3

        assert_eq!(*states.borrow(), vec![1, 2, 3]);
    }

    #[test]
    fn test_bloc_named() {
        let bloc = StatefulBloc::named("auth", "initial".to_string(), |_: (), _, _| {});
        assert_eq!(bloc.name(), Some("auth"));
    }

    #[derive(Debug, Clone, PartialEq)]
    enum AuthState {
        Initial,
        Loading,
        Authenticated { user: String },
        Error { message: String },
    }

    #[derive(Debug, Clone)]
    enum AuthEvent {
        Login { email: String, password: String },
        Logout,
    }

    #[test]
    fn test_auth_bloc_example() {
        let bloc = StatefulBloc::new(AuthState::Initial, |event, _, emit| {
            match event {
                AuthEvent::Login { email, password } => {
                    emit(AuthState::Loading);
                    if email == "test@test.com" && password == "password" {
                        emit(AuthState::Authenticated { user: "Test User".into() });
                    } else {
                        emit(AuthState::Error { message: "Invalid credentials".into() });
                    }
                }
                AuthEvent::Logout => {
                    emit(AuthState::Initial);
                }
            }
        });

        assert_eq!(bloc.state(), AuthState::Initial);

        bloc.add(AuthEvent::Login {
            email: "test@test.com".into(),
            password: "password".into(),
        });

        assert_eq!(
            bloc.state(),
            AuthState::Authenticated { user: "Test User".into() }
        );

        bloc.add(AuthEvent::Logout);
        assert_eq!(bloc.state(), AuthState::Initial);
    }

    #[test]
    fn test_event_transformer() {
        let transformer = EventTransformer::<i32>::take_last();
        
        let result = transformer.apply(vec![1, 2, 3]);
        assert_eq!(result, Some(3));

        let transformer = EventTransformer::<i32>::take_first();
        let result = transformer.apply(vec![1, 2, 3]);
        assert_eq!(result, Some(1));
    }
}
