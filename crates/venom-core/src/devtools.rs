//! DevTools - Logging, Debugging, and State Inspection
//!
//! This module provides tools for debugging and monitoring state management:
//!
//! - [`BlocObserver`] - Observe bloc lifecycle and state changes
//! - [`ConsoleObserver`] - Log state changes to console
//! - [`StateHistory`] - Time-travel debugging with undo/redo
//! - [`DevToolsConfig`] - Global configuration
//!
//! # Example
//!
//! ```
//! use venom_core::{DevTools, ConsoleObserver, CubitCore};
//!
//! // Enable global observer
//! DevTools::set_observer(ConsoleObserver::new());
//!
//! // Now all state changes are logged
//! let counter = CubitCore::named("counter", 0);
//! counter.emit(1); // Logs: [counter] State: 0 -> 1
//! ```

use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// ============================================================================
// DEVTOOLS CONFIG
// ============================================================================

/// Configuration for DevTools
#[derive(Debug, Clone)]
pub struct DevToolsConfig {
    /// Enable/disable all DevTools features
    pub enabled: bool,
    /// Log state changes
    pub log_state_changes: bool,
    /// Log event dispatches
    pub log_events: bool,
    /// Log subscription add/remove
    pub log_subscriptions: bool,
    /// Include timestamps in logs
    pub include_timestamps: bool,
    /// Maximum history entries for time-travel
    pub max_history_entries: usize,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(debug_assertions), // Only enabled in debug builds by default
            log_state_changes: true,
            log_events: true,
            log_subscriptions: false,
            include_timestamps: true,
            max_history_entries: 100,
        }
    }
}

impl DevToolsConfig {
    /// Create a new config with all logging enabled
    pub fn verbose() -> Self {
        Self {
            enabled: true,
            log_state_changes: true,
            log_events: true,
            log_subscriptions: true,
            include_timestamps: true,
            max_history_entries: 100,
        }
    }

    /// Create a disabled config
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

// ============================================================================
// BLOC OBSERVER TRAIT
// ============================================================================

/// Observer trait for monitoring bloc lifecycle and state changes
///
/// Implement this trait to create custom observers for logging,
/// analytics, or debugging purposes.
///
/// # Example
///
/// ```
/// use venom_core::BlocObserver;
///
/// struct MyObserver;
///
/// impl BlocObserver for MyObserver {
///     fn on_create(&self, bloc_name: Option<&str>) {
///         println!("Bloc created: {:?}", bloc_name);
///     }
///
///     fn on_change(&self, bloc_name: Option<&str>, change: &str) {
///         println!("[{:?}] {}", bloc_name, change);
///     }
///
///     fn on_event(&self, bloc_name: Option<&str>, event: &str) {
///         println!("[{:?}] Event: {}", bloc_name, event);
///     }
///
///     fn on_error(&self, bloc_name: Option<&str>, error: &str) {
///         eprintln!("[{:?}] Error: {}", bloc_name, error);
///     }
///
///     fn on_close(&self, bloc_name: Option<&str>) {
///         println!("Bloc closed: {:?}", bloc_name);
///     }
/// }
/// ```
pub trait BlocObserver: Send + Sync {
    /// Called when a new bloc/cubit is created
    fn on_create(&self, bloc_name: Option<&str>);

    /// Called when state changes
    ///
    /// The `change` string contains formatted old -> new state
    fn on_change(&self, bloc_name: Option<&str>, change: &str);

    /// Called when an event is dispatched (Bloc only)
    fn on_event(&self, bloc_name: Option<&str>, event: &str);

    /// Called when an error occurs during event handling
    fn on_error(&self, bloc_name: Option<&str>, error: &str);

    /// Called when a bloc/cubit is closed/dropped
    fn on_close(&self, bloc_name: Option<&str>);
}

// ============================================================================
// CONSOLE OBSERVER
// ============================================================================

/// Default observer that logs to console (stdout/stderr)
///
/// # Example
///
/// ```
/// use venom_core::{ConsoleObserver, DevToolsConfig};
///
/// // With default config
/// let observer = ConsoleObserver::new();
///
/// // With custom config
/// let observer = ConsoleObserver::with_config(DevToolsConfig::verbose());
/// ```
pub struct ConsoleObserver {
    config: DevToolsConfig,
    start_time: Instant,
}

impl ConsoleObserver {
    /// Create a new console observer with default config
    pub fn new() -> Self {
        Self {
            config: DevToolsConfig::default(),
            start_time: Instant::now(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: DevToolsConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
        }
    }

    fn format_timestamp(&self) -> String {
        if self.config.include_timestamps {
            let elapsed = self.start_time.elapsed();
            format!("[{:>6.2}s] ", elapsed.as_secs_f64())
        } else {
            String::new()
        }
    }

    fn format_name(&self, name: Option<&str>) -> String {
        match name {
            Some(n) => format!("[{}] ", n),
            None => String::new(),
        }
    }
}

impl Default for ConsoleObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl BlocObserver for ConsoleObserver {
    fn on_create(&self, bloc_name: Option<&str>) {
        if !self.config.enabled {
            return;
        }
        println!(
            "{}{}📦 Created",
            self.format_timestamp(),
            self.format_name(bloc_name)
        );
    }

    fn on_change(&self, bloc_name: Option<&str>, change: &str) {
        if !self.config.enabled || !self.config.log_state_changes {
            return;
        }
        println!(
            "{}{}🔄 {}",
            self.format_timestamp(),
            self.format_name(bloc_name),
            change
        );
    }

    fn on_event(&self, bloc_name: Option<&str>, event: &str) {
        if !self.config.enabled || !self.config.log_events {
            return;
        }
        println!(
            "{}{}⚡ Event: {}",
            self.format_timestamp(),
            self.format_name(bloc_name),
            event
        );
    }

    fn on_error(&self, bloc_name: Option<&str>, error: &str) {
        if !self.config.enabled {
            return;
        }
        eprintln!(
            "{}{}❌ Error: {}",
            self.format_timestamp(),
            self.format_name(bloc_name),
            error
        );
    }

    fn on_close(&self, bloc_name: Option<&str>) {
        if !self.config.enabled {
            return;
        }
        println!(
            "{}{}🗑️ Closed",
            self.format_timestamp(),
            self.format_name(bloc_name)
        );
    }
}

// ============================================================================
// SILENT OBSERVER
// ============================================================================

/// Observer that does nothing (for production or testing)
pub struct SilentObserver;

impl BlocObserver for SilentObserver {
    fn on_create(&self, _: Option<&str>) {}
    fn on_change(&self, _: Option<&str>, _: &str) {}
    fn on_event(&self, _: Option<&str>, _: &str) {}
    fn on_error(&self, _: Option<&str>, _: &str) {}
    fn on_close(&self, _: Option<&str>) {}
}

// ============================================================================
// GLOBAL DEVTOOLS
// ============================================================================

/// Global DevTools singleton for managing observers
///
/// # Example
///
/// ```
/// use venom_core::{DevTools, ConsoleObserver};
///
/// // Set global observer
/// DevTools::set_observer(ConsoleObserver::new());
///
/// // Or use the silent observer for production
/// DevTools::disable();
/// ```
pub struct DevTools;

// Global observer storage
static GLOBAL_OBSERVER: RwLock<Option<Arc<dyn BlocObserver>>> = RwLock::new(None);

impl DevTools {
    /// Set the global bloc observer
    pub fn set_observer<O: BlocObserver + 'static>(observer: O) {
        let mut guard = GLOBAL_OBSERVER.write().unwrap();
        *guard = Some(Arc::new(observer));
    }

    /// Remove the global observer (disables logging)
    pub fn disable() {
        let mut guard = GLOBAL_OBSERVER.write().unwrap();
        *guard = None;
    }

    /// Get the current global observer (if any)
    pub fn observer() -> Option<Arc<dyn BlocObserver>> {
        GLOBAL_OBSERVER.read().unwrap().clone()
    }

    /// Log a bloc creation
    pub fn log_create(name: Option<&str>) {
        if let Some(obs) = Self::observer() {
            obs.on_create(name);
        }
    }

    /// Log a state change
    pub fn log_change(name: Option<&str>, change: &str) {
        if let Some(obs) = Self::observer() {
            obs.on_change(name, change);
        }
    }

    /// Log an event dispatch
    pub fn log_event(name: Option<&str>, event: &str) {
        if let Some(obs) = Self::observer() {
            obs.on_event(name, event);
        }
    }

    /// Log an error
    pub fn log_error(name: Option<&str>, error: &str) {
        if let Some(obs) = Self::observer() {
            obs.on_error(name, error);
        }
    }

    /// Log a bloc close
    pub fn log_close(name: Option<&str>) {
        if let Some(obs) = Self::observer() {
            obs.on_close(name);
        }
    }
}

// ============================================================================
// STATE HISTORY (Time-Travel Debugging)
// ============================================================================

/// Entry in state history
#[derive(Debug, Clone)]
pub struct HistoryEntry<S> {
    /// The state at this point
    pub state: S,
    /// When this state was recorded
    pub timestamp: Instant,
    /// Optional description of what caused this state
    pub description: Option<String>,
}

impl<S> HistoryEntry<S> {
    /// Create a new history entry
    pub fn new(state: S) -> Self {
        Self {
            state,
            timestamp: Instant::now(),
            description: None,
        }
    }

    /// Create with description
    pub fn with_description(state: S, description: impl Into<String>) -> Self {
        Self {
            state,
            timestamp: Instant::now(),
            description: Some(description.into()),
        }
    }

    /// Get age of this entry
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

/// State history for time-travel debugging
///
/// Keeps track of state changes and allows undo/redo operations.
///
/// # Example
///
/// ```
/// use venom_core::StateHistory;
///
/// let mut history = StateHistory::new(10);
///
/// history.push(0);
/// history.push(1);
/// history.push(2);
///
/// assert_eq!(history.current(), Some(&2));
///
/// // Undo
/// assert_eq!(history.undo(), Some(&1));
/// assert_eq!(history.current(), Some(&1));
///
/// // Redo
/// assert_eq!(history.redo(), Some(&2));
/// assert_eq!(history.current(), Some(&2));
/// ```
pub struct StateHistory<S> {
    entries: VecDeque<HistoryEntry<S>>,
    cursor: usize,
    max_entries: usize,
}

impl<S: Clone> StateHistory<S> {
    /// Create a new state history with maximum entries
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_entries),
            cursor: 0,
            max_entries,
        }
    }

    /// Push a new state to history
    ///
    /// This clears any "future" states (from undos).
    pub fn push(&mut self, state: S) {
        // Remove future states if we're not at the end
        while self.entries.len() > self.cursor {
            self.entries.pop_back();
        }

        // Remove oldest if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }

        self.entries.push_back(HistoryEntry::new(state));
        self.cursor = self.entries.len();
    }

    /// Push with description
    pub fn push_with_description(&mut self, state: S, description: impl Into<String>) {
        while self.entries.len() > self.cursor {
            self.entries.pop_back();
        }

        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }

        self.entries
            .push_back(HistoryEntry::with_description(state, description));
        self.cursor = self.entries.len();
    }

    /// Get the current state
    pub fn current(&self) -> Option<&S> {
        if self.cursor > 0 {
            self.entries.get(self.cursor - 1).map(|e| &e.state)
        } else {
            None
        }
    }

    /// Get current entry with metadata
    pub fn current_entry(&self) -> Option<&HistoryEntry<S>> {
        if self.cursor > 0 {
            self.entries.get(self.cursor - 1)
        } else {
            None
        }
    }

    /// Undo - go back to previous state
    ///
    /// Returns the previous state if available.
    pub fn undo(&mut self) -> Option<&S> {
        if self.cursor > 1 {
            self.cursor -= 1;
            self.current()
        } else {
            None
        }
    }

    /// Redo - go forward to next state
    ///
    /// Returns the next state if available.
    pub fn redo(&mut self) -> Option<&S> {
        if self.cursor < self.entries.len() {
            self.cursor += 1;
            self.current()
        } else {
            None
        }
    }

    /// Check if undo is possible
    pub fn can_undo(&self) -> bool {
        self.cursor > 1
    }

    /// Check if redo is possible
    pub fn can_redo(&self) -> bool {
        self.cursor < self.entries.len()
    }

    /// Get number of entries in history
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = 0;
    }

    /// Get all entries (for inspection)
    pub fn entries(&self) -> &VecDeque<HistoryEntry<S>> {
        &self.entries
    }

    /// Jump to a specific index in history
    pub fn jump_to(&mut self, index: usize) -> Option<&S> {
        if index > 0 && index <= self.entries.len() {
            self.cursor = index;
            self.current()
        } else {
            None
        }
    }
}

impl<S: Clone> Default for StateHistory<S> {
    fn default() -> Self {
        Self::new(100)
    }
}

// ============================================================================
// DEBUGGABLE CUBIT
// ============================================================================

/// A cubit wrapper with DevTools integration
///
/// This wrapper automatically logs all state changes to the global observer.
///
/// # Example
///
/// ```
/// use venom_core::{DebuggableCubit, DevTools, ConsoleObserver};
///
/// // Enable logging
/// DevTools::set_observer(ConsoleObserver::new());
///
/// let counter = DebuggableCubit::new("counter", 0);
/// counter.emit(1); // Automatically logged
/// ```
pub struct DebuggableCubit<S> {
    name: String,
    state: Rc<RefCell<S>>,
    history: Rc<RefCell<StateHistory<S>>>,
    enable_history: bool,
}

impl<S: Clone + PartialEq + Debug + 'static> DebuggableCubit<S> {
    /// Create a new debuggable cubit
    pub fn new(name: impl Into<String>, initial: S) -> Self {
        let name = name.into();
        DevTools::log_create(Some(&name));

        let mut history = StateHistory::new(100);
        history.push(initial.clone());

        Self {
            name,
            state: Rc::new(RefCell::new(initial)),
            history: Rc::new(RefCell::new(history)),
            enable_history: true,
        }
    }

    /// Create without history tracking
    pub fn without_history(name: impl Into<String>, initial: S) -> Self {
        let name = name.into();
        DevTools::log_create(Some(&name));

        Self {
            name,
            state: Rc::new(RefCell::new(initial)),
            history: Rc::new(RefCell::new(StateHistory::new(0))),
            enable_history: false,
        }
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.state.borrow().clone()
    }

    /// Emit a new state
    pub fn emit(&self, new_state: S) {
        let old_state = self.state.borrow().clone();

        if old_state != new_state {
            // Log the change
            let change = format!("{:?} -> {:?}", old_state, new_state);
            DevTools::log_change(Some(&self.name), &change);

            // Update state
            *self.state.borrow_mut() = new_state.clone();

            // Record history
            if self.enable_history {
                self.history.borrow_mut().push(new_state);
            }
        }
    }

    /// Emit with description (for history)
    pub fn emit_with_description(&self, new_state: S, description: impl Into<String>) {
        let old_state = self.state.borrow().clone();

        if old_state != new_state {
            let desc = description.into();
            let change = format!("{:?} -> {:?} ({})", old_state, new_state, desc);
            DevTools::log_change(Some(&self.name), &change);

            *self.state.borrow_mut() = new_state.clone();

            if self.enable_history {
                self.history.borrow_mut().push_with_description(new_state, desc);
            }
        }
    }

    /// Undo to previous state
    pub fn undo(&self) -> bool {
        if let Some(state) = self.history.borrow_mut().undo() {
            let state = state.clone();
            DevTools::log_change(Some(&self.name), &format!("Undo -> {:?}", state));
            *self.state.borrow_mut() = state;
            true
        } else {
            false
        }
    }

    /// Redo to next state
    pub fn redo(&self) -> bool {
        if let Some(state) = self.history.borrow_mut().redo() {
            let state = state.clone();
            DevTools::log_change(Some(&self.name), &format!("Redo -> {:?}", state));
            *self.state.borrow_mut() = state;
            true
        } else {
            false
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.borrow().can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.borrow().can_redo()
    }

    /// Get history
    pub fn history(&self) -> std::cell::Ref<'_, StateHistory<S>> {
        self.history.borrow()
    }

    /// Get the cubit name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<S> Drop for DebuggableCubit<S> {
    fn drop(&mut self) {
        DevTools::log_close(Some(&self.name));
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_history_basic() {
        let mut history = StateHistory::new(10);

        history.push(0);
        history.push(1);
        history.push(2);

        assert_eq!(history.current(), Some(&2));
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_state_history_undo_redo() {
        let mut history = StateHistory::new(10);

        history.push(0);
        history.push(1);
        history.push(2);

        // Undo
        assert_eq!(history.undo(), Some(&1));
        assert_eq!(history.current(), Some(&1));

        assert_eq!(history.undo(), Some(&0));
        assert_eq!(history.current(), Some(&0));

        // Can't undo past the beginning
        assert_eq!(history.undo(), None);

        // Redo
        assert_eq!(history.redo(), Some(&1));
        assert_eq!(history.redo(), Some(&2));

        // Can't redo past the end
        assert_eq!(history.redo(), None);
    }

    #[test]
    fn test_state_history_undo_then_push() {
        let mut history = StateHistory::new(10);

        history.push(0);
        history.push(1);
        history.push(2);

        history.undo(); // at 1
        history.push(3); // clears redo, adds 3

        assert_eq!(history.current(), Some(&3));
        assert!(!history.can_redo());
    }

    #[test]
    fn test_state_history_max_entries() {
        let mut history = StateHistory::new(3);

        for i in 0..5 {
            history.push(i);
        }

        // Only last 3 entries kept
        assert_eq!(history.len(), 3);
        assert_eq!(history.current(), Some(&4));
    }

    #[test]
    fn test_state_history_jump_to() {
        let mut history = StateHistory::new(10);

        history.push(0);
        history.push(1);
        history.push(2);
        history.push(3);

        assert_eq!(history.jump_to(2), Some(&1));
        assert_eq!(history.current(), Some(&1));
    }

    #[test]
    fn test_console_observer_format() {
        let observer = ConsoleObserver::new();
        // Just ensure it doesn't panic
        observer.on_create(Some("test"));
        observer.on_change(Some("test"), "0 -> 1");
        observer.on_event(Some("test"), "Increment");
        observer.on_error(Some("test"), "Something went wrong");
        observer.on_close(Some("test"));
    }

    #[test]
    fn test_debuggable_cubit() {
        let cubit = DebuggableCubit::new("test", 0);

        assert_eq!(cubit.state(), 0);

        cubit.emit(1);
        assert_eq!(cubit.state(), 1);

        cubit.emit(2);
        assert_eq!(cubit.state(), 2);

        // Undo
        assert!(cubit.undo());
        assert_eq!(cubit.state(), 1);

        // Redo
        assert!(cubit.redo());
        assert_eq!(cubit.state(), 2);
    }

    #[test]
    fn test_devtools_config() {
        let config = DevToolsConfig::default();
        assert!(config.log_state_changes);

        let config = DevToolsConfig::disabled();
        assert!(!config.enabled);

        let config = DevToolsConfig::verbose();
        assert!(config.log_subscriptions);
    }
}
