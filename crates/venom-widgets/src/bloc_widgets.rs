//! Bloc Widgets - Thread-Safe State-aware UI Components
//!
//! This module provides widgets that integrate with the thread-safe state management system:
//!
//! - [`SyncBlocBuilder`] - Rebuilds when state changes
//! - [`SyncBlocListener`] - Executes side effects on state changes
//! - [`SyncBlocConsumer`] - Combines builder and listener
//! - [`SyncBlocSelector`] - Only rebuilds when selected portion of state changes
//!
//! # Example
//!
//! ```ignore
//! use venom_widgets::{SyncBlocBuilder, Container, Text};
//! use venom_core::SyncCubitCore;
//!
//! let counter = SyncCubitCore::new(0);
//!
//! // Build UI based on state
//! let widget = SyncBlocBuilder::new(counter, |state| {
//!     Container::new()
//!         .child(Text::new(format!("Count: {}", state)))
//! });
//! ```

// Allow unsafe code for Send+Sync implementations.
// SAFETY: All widget types properly manage thread-safe Arc/RwLock state.
#![allow(unsafe_code)]


use std::any::Any;
use std::sync::{Arc, RwLock};

use venom_core::{BoxConstraints, SyncCubitCore, Offset, Size};
use venom_render::PaintCanvas;

use crate::widget::{BoxedWidget, Widget};
use crate::basic::Container;

// ============================================================================
// SYNC BLOC BUILDER
// ============================================================================

/// Thread-safe widget that rebuilds when bloc state changes
///
/// `SyncBlocBuilder` subscribes to a sync cubit and rebuilds its child
/// whenever the state changes.
///
/// # Type Parameters
///
/// * `S` - The state type (must be Send + Sync)
///
/// # Example
///
/// ```ignore
/// use venom_widgets::{SyncBlocBuilder, Text};
/// use venom_core::SyncCubitCore;
///
/// let counter = SyncCubitCore::new(0);
///
/// let widget = SyncBlocBuilder::new(counter.clone(), |state| {
///     Box::new(Text::new(format!("Count: {}", state)))
/// });
/// ```
pub struct SyncBlocBuilder<S: Clone + PartialEq + Send + Sync + 'static> {
    cubit: SyncCubitCore<S>,
    builder: Arc<dyn Fn(&S) -> BoxedWidget + Send + Sync>,
    build_when: Option<Arc<dyn Fn(&S, &S) -> bool + Send + Sync>>,
    // Cached child widget
    cached: Arc<RwLock<Option<(u64, BoxedWidget)>>>,
}

impl<S: Clone + PartialEq + Send + Sync + 'static> SyncBlocBuilder<S> {
    /// Create a new SyncBlocBuilder
    ///
    /// # Arguments
    ///
    /// * `cubit` - The sync cubit to observe
    /// * `builder` - Function that builds a widget from current state
    pub fn new<F>(cubit: SyncCubitCore<S>, builder: F) -> Self
    where
        F: Fn(&S) -> BoxedWidget + Send + Sync + 'static,
    {
        Self {
            cubit,
            builder: Arc::new(builder),
            build_when: None,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    /// Add a condition for when to rebuild
    ///
    /// If the condition returns false, the cached widget is used.
    pub fn build_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + Send + Sync + 'static,
    {
        self.build_when = Some(Arc::new(condition));
        self
    }

    /// Get or build the child widget, returns the size from layout
    fn ensure_child(&self, constraints: BoxConstraints) -> Size {
        let current_version = self.cubit.version();
        
        {
            let cached = self.cached.read().unwrap();
            if let Some((version, child)) = cached.as_ref() {
                if *version == current_version {
                    return child.layout(constraints);
                }
            }
        }

        // Need to rebuild
        let state = self.cubit.state();
        let child = (self.builder)(&state);
        let size = child.layout(constraints);
        
        *self.cached.write().unwrap() = Some((current_version, child));
        size
    }
}

impl<S: Clone + PartialEq + Send + Sync + 'static> Widget for SyncBlocBuilder<S> {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.ensure_child(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let cached = self.cached.read().unwrap();
        if let Some((_, child)) = cached.as_ref() {
            child.paint(canvas, offset);
        }
    }

    fn children(&self) -> &[BoxedWidget] {
        &[]
    }

    fn type_name(&self) -> &'static str {
        "SyncBlocBuilder"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Send + Sync implementations
unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Send for SyncBlocBuilder<S> {}
unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Sync for SyncBlocBuilder<S> {}

// ============================================================================
// SYNC BLOC LISTENER
// ============================================================================

/// Thread-safe widget that executes side effects on state changes
///
/// Unlike `SyncBlocBuilder`, `SyncBlocListener` doesn't rebuild its child.
/// It's used for one-time reactions like navigation, showing dialogs, etc.
pub struct SyncBlocListener<S: Clone + PartialEq + Send + Sync + 'static> {
    cubit: SyncCubitCore<S>,
    listener: Arc<dyn Fn(&S, &S) + Send + Sync>,
    child: BoxedWidget,
    listen_when: Option<Arc<dyn Fn(&S, &S) -> bool + Send + Sync>>,
    last_state: Arc<RwLock<Option<S>>>,
}

impl<S: Clone + PartialEq + Send + Sync + 'static> SyncBlocListener<S> {
    /// Create a new SyncBlocListener
    pub fn new<W: Widget>(cubit: SyncCubitCore<S>, child: W) -> Self {
        Self {
            cubit,
            listener: Arc::new(|_, _| {}),
            child: Box::new(child),
            listen_when: None,
            last_state: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the listener callback
    pub fn listener<F>(mut self, callback: F) -> Self
    where
        F: Fn(&S, &S) + Send + Sync + 'static,
    {
        self.listener = Arc::new(callback);
        self
    }

    /// Add a condition for when to listen
    pub fn listen_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + Send + Sync + 'static,
    {
        self.listen_when = Some(Arc::new(condition));
        self
    }

    /// Check for state changes and call listener if needed
    fn check_state(&self) {
        let current = self.cubit.state();
        let mut last = self.last_state.write().unwrap();

        if let Some(old) = last.as_ref() {
            if old != &current {
                let should_listen = self
                    .listen_when
                    .as_ref()
                    .map(|cond| cond(old, &current))
                    .unwrap_or(true);

                if should_listen {
                    (self.listener)(old, &current);
                }
            }
        }

        *last = Some(current);
    }
}

impl<S: Clone + PartialEq + Send + Sync + 'static> Widget for SyncBlocListener<S> {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.check_state();
        self.child.layout(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        self.check_state();
        self.child.paint(canvas, offset);
    }

    fn type_name(&self) -> &'static str {
        "SyncBlocListener"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Send for SyncBlocListener<S> {}
unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Sync for SyncBlocListener<S> {}

// ============================================================================
// SYNC BLOC CONSUMER
// ============================================================================

/// Thread-safe widget that combines BlocBuilder and BlocListener
pub struct SyncBlocConsumer<S: Clone + PartialEq + Send + Sync + 'static> {
    cubit: SyncCubitCore<S>,
    builder: Arc<dyn Fn(&S) -> BoxedWidget + Send + Sync>,
    listener: Arc<dyn Fn(&S, &S) + Send + Sync>,
    build_when: Option<Arc<dyn Fn(&S, &S) -> bool + Send + Sync>>,
    listen_when: Option<Arc<dyn Fn(&S, &S) -> bool + Send + Sync>>,
    cached: Arc<RwLock<Option<BoxedWidget>>>,
    last_state: Arc<RwLock<Option<S>>>,
}

impl<S: Clone + PartialEq + Send + Sync + 'static> SyncBlocConsumer<S> {
    /// Create a new SyncBlocConsumer
    pub fn new(cubit: SyncCubitCore<S>) -> Self {
        Self {
            cubit,
            builder: Arc::new(|_| Box::new(Container::new())),
            listener: Arc::new(|_, _| {}),
            build_when: None,
            listen_when: None,
            cached: Arc::new(RwLock::new(None)),
            last_state: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the builder function
    pub fn builder<F>(mut self, builder: F) -> Self
    where
        F: Fn(&S) -> BoxedWidget + Send + Sync + 'static,
    {
        self.builder = Arc::new(builder);
        self
    }

    /// Set the listener callback
    pub fn listener<F>(mut self, listener: F) -> Self
    where
        F: Fn(&S, &S) + Send + Sync + 'static,
    {
        self.listener = Arc::new(listener);
        self
    }

    /// Add rebuild condition
    pub fn build_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + Send + Sync + 'static,
    {
        self.build_when = Some(Arc::new(condition));
        self
    }

    /// Add listen condition
    pub fn listen_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + Send + Sync + 'static,
    {
        self.listen_when = Some(Arc::new(condition));
        self
    }

    fn process_state(&self) {
        let current = self.cubit.state();
        let mut last = self.last_state.write().unwrap();

        if let Some(old) = last.as_ref() {
            if old != &current {
                // Check listener
                let should_listen = self
                    .listen_when
                    .as_ref()
                    .map(|cond| cond(old, &current))
                    .unwrap_or(true);

                if should_listen {
                    (self.listener)(old, &current);
                }

                // Check rebuild
                let should_build = self
                    .build_when
                    .as_ref()
                    .map(|cond| cond(old, &current))
                    .unwrap_or(true);

                if should_build {
                    *self.cached.write().unwrap() = Some((self.builder)(&current));
                }
            }
        } else {
            *self.cached.write().unwrap() = Some((self.builder)(&current));
        }

        *last = Some(current);
    }

    fn ensure_child(&self, constraints: BoxConstraints) -> Size {
        self.process_state();
        
        {
            let cached = self.cached.read().unwrap();
            if let Some(child) = cached.as_ref() {
                return child.layout(constraints);
            }
        }

        // Build if not exists
        let state = self.cubit.state();
        let child = (self.builder)(&state);
        let size = child.layout(constraints);
        *self.cached.write().unwrap() = Some(child);
        size
    }
}

impl<S: Clone + PartialEq + Send + Sync + 'static> Widget for SyncBlocConsumer<S> {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.ensure_child(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let cached = self.cached.read().unwrap();
        if let Some(child) = cached.as_ref() {
            child.paint(canvas, offset);
        }
    }

    fn type_name(&self) -> &'static str {
        "SyncBlocConsumer"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Send for SyncBlocConsumer<S> {}
unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Sync for SyncBlocConsumer<S> {}

// ============================================================================
// SYNC BLOC SELECTOR
// ============================================================================

/// Thread-safe widget that only rebuilds when a selected part of state changes
pub struct SyncBlocSelector<S, T>
where
    S: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    cubit: SyncCubitCore<S>,
    selector: Arc<dyn Fn(&S) -> T + Send + Sync>,
    builder: Arc<dyn Fn(&T) -> BoxedWidget + Send + Sync>,
    cached: Arc<RwLock<Option<(T, BoxedWidget)>>>,
}

impl<S, T> SyncBlocSelector<S, T>
where
    S: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    /// Create a new SyncBlocSelector
    pub fn new<FS, FB>(cubit: SyncCubitCore<S>, selector: FS, builder: FB) -> Self
    where
        FS: Fn(&S) -> T + Send + Sync + 'static,
        FB: Fn(&T) -> BoxedWidget + Send + Sync + 'static,
    {
        Self {
            cubit,
            selector: Arc::new(selector),
            builder: Arc::new(builder),
            cached: Arc::new(RwLock::new(None)),
        }
    }

    fn ensure_child(&self, constraints: BoxConstraints) -> Size {
        let state = self.cubit.state();
        let selection = (self.selector)(&state);

        {
            let cached = self.cached.read().unwrap();
            if let Some((cached_selection, child)) = cached.as_ref() {
                if cached_selection == &selection {
                    return child.layout(constraints);
                }
            }
        }

        // Need to rebuild
        let child = (self.builder)(&selection);
        let size = child.layout(constraints);
        *self.cached.write().unwrap() = Some((selection, child));
        size
    }
}

impl<S, T> Widget for SyncBlocSelector<S, T>
where
    S: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.ensure_child(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let cached = self.cached.read().unwrap();
        if let Some((_, child)) = cached.as_ref() {
            child.paint(canvas, offset);
        }
    }

    fn type_name(&self) -> &'static str {
        "SyncBlocSelector"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

unsafe impl<S, T> Send for SyncBlocSelector<S, T>
where
    S: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
}

unsafe impl<S, T> Sync for SyncBlocSelector<S, T>
where
    S: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
}

// ============================================================================
// SYNC STATEFUL WIDGET
// ============================================================================

/// Thread-safe simple stateful widget wrapper
pub struct SyncStatefulWidget<S: Clone + PartialEq + Send + Sync + 'static> {
    cubit: SyncCubitCore<S>,
    builder: Arc<dyn Fn(&S) -> BoxedWidget + Send + Sync>,
    cached: Arc<RwLock<Option<(u64, BoxedWidget)>>>,
}

impl<S: Clone + PartialEq + Send + Sync + 'static> SyncStatefulWidget<S> {
    /// Create a new sync stateful widget
    pub fn new<F>(cubit: SyncCubitCore<S>, builder: F) -> Self
    where
        F: Fn(&S) -> BoxedWidget + Send + Sync + 'static,
    {
        Self {
            cubit,
            builder: Arc::new(builder),
            cached: Arc::new(RwLock::new(None)),
        }
    }

    fn ensure_child(&self, constraints: BoxConstraints) -> Size {
        let version = self.cubit.version();

        {
            let cached = self.cached.read().unwrap();
            if let Some((v, child)) = cached.as_ref() {
                if *v == version {
                    return child.layout(constraints);
                }
            }
        }

        let state = self.cubit.state();
        let child = (self.builder)(&state);
        let size = child.layout(constraints);
        *self.cached.write().unwrap() = Some((version, child));
        size
    }
}

impl<S: Clone + PartialEq + Send + Sync + 'static> Widget for SyncStatefulWidget<S> {
    fn layout(&self, constraints: BoxConstraints) -> Size {
        self.ensure_child(constraints)
    }

    fn paint(&self, canvas: &mut dyn PaintCanvas, offset: Offset) {
        let cached = self.cached.read().unwrap();
        if let Some((_, child)) = cached.as_ref() {
            child.paint(canvas, offset);
        }
    }

    fn type_name(&self) -> &'static str {
        "SyncStatefulWidget"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Send for SyncStatefulWidget<S> {}
unsafe impl<S: Clone + PartialEq + Send + Sync + 'static> Sync for SyncStatefulWidget<S> {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_sync_bloc_builder_caching() {
        let cubit = SyncCubitCore::new(0);
        let build_count = Arc::new(AtomicI32::new(0));
        let build_count_clone = Arc::clone(&build_count);

        let builder = SyncBlocBuilder::new(cubit.clone(), move |_state| {
            build_count_clone.fetch_add(1, Ordering::SeqCst);
            Box::new(Container::new())
        });

        // First access builds
        let _ = builder.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 1);

        // Same version, should use cache
        let _ = builder.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 1);

        // Change state, should rebuild
        cubit.emit(1);
        let _ = builder.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_sync_bloc_consumer_listener() {
        let cubit = SyncCubitCore::new(0);
        let listener_called = Arc::new(AtomicI32::new(0));
        let listener_called_clone = Arc::clone(&listener_called);

        let consumer = SyncBlocConsumer::new(cubit.clone())
            .listener(move |_old, _new| {
                listener_called_clone.fetch_add(1, Ordering::SeqCst);
            })
            .builder(|_| Box::new(Container::new()));

        // Initial layout
        let _ = consumer.layout(BoxConstraints::default());

        // Change state and layout again
        cubit.emit(1);
        let _ = consumer.layout(BoxConstraints::default());

        assert_eq!(listener_called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_sync_stateful_widget() {
        let cubit = SyncCubitCore::new(42);

        let widget = SyncStatefulWidget::new(cubit.clone(), |_state| {
            Box::new(Container::new())
        });

        // Should build without panicking
        let _ = widget.layout(BoxConstraints::default());

        cubit.emit(100);
        let _ = widget.layout(BoxConstraints::default());
    }

    #[test]
    fn test_sync_bloc_selector() {
        #[derive(Clone, PartialEq)]
        struct AppState {
            count: i32,
            name: String,
        }

        let cubit = SyncCubitCore::new(AppState {
            count: 0,
            name: "test".into(),
        });

        let build_count = Arc::new(AtomicI32::new(0));
        let build_count_clone = Arc::clone(&build_count);

        let selector = SyncBlocSelector::new(
            cubit.clone(),
            |state: &AppState| state.count,
            move |_count| {
                build_count_clone.fetch_add(1, Ordering::SeqCst);
                Box::new(Container::new())
            },
        );

        // First build
        let _ = selector.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 1);

        // Change name only - should NOT rebuild (selection unchanged)
        cubit.emit(AppState {
            count: 0,
            name: "changed".into(),
        });
        let _ = selector.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 1);

        // Change count - SHOULD rebuild
        cubit.emit(AppState {
            count: 1,
            name: "changed".into(),
        });
        let _ = selector.layout(BoxConstraints::default());
        assert_eq!(build_count.load(Ordering::SeqCst), 2);
    }
}
