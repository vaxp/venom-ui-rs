//! Provider System for State Management
//!
//! Providers make cubits and blocs available to the widget tree.
//! This module provides utilities for:
//! - Providing state to descendants
//! - Building UIs based on state
//! - Listening to state changes for side effects
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      BlocProvider                           │
//! │  ┌──────────────────────────────────────────────────────┐  │
//! │  │                    BlocBuilder                        │  │
//! │  │  ┌────────────────────────────────────────────────┐  │  │
//! │  │  │                  Child Widget                   │  │  │
//! │  │  │  (Can access bloc via Provider::of)            │  │  │
//! │  │  └────────────────────────────────────────────────┘  │  │
//! │  └──────────────────────────────────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```ignore
//! use venom_core::{BlocProvider, BlocBuilder, CubitCore};
//!
//! // Provide a cubit to descendants
//! let provider = BlocProvider::new(
//!     CounterCubit::new(),
//!     BlocBuilder::new(|cubit: &CounterCubit| {
//!         // Build widget based on cubit state
//!         Text::new(format!("Count: {}", cubit.state()))
//!     })
//! );
//! ```

use crate::subscription::SubscriptionHandle;
use crate::cubit::CubitCore;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ============================================================================
// PROVIDER CONTEXT
// ============================================================================

/// Context for storing and retrieving provided values
///
/// This is a simple dependency injection container that holds
/// blocs/cubits by their type.
#[derive(Default)]
pub struct ProviderContext {
    providers: HashMap<TypeId, Rc<dyn Any>>,
}

impl ProviderContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a value of type T
    pub fn provide<T: 'static>(&mut self, value: T) {
        self.providers.insert(TypeId::of::<T>(), Rc::new(value));
    }

    /// Get a reference to a provided value
    pub fn get<T: 'static>(&self) -> Option<Rc<T>> {
        self.providers
            .get(&TypeId::of::<T>())
            .and_then(|v| v.clone().downcast::<T>().ok())
    }

    /// Check if a type is provided
    pub fn has<T: 'static>(&self) -> bool {
        self.providers.contains_key(&TypeId::of::<T>())
    }

    /// Remove a provided value
    pub fn remove<T: 'static>(&mut self) -> bool {
        self.providers.remove(&TypeId::of::<T>()).is_some()
    }

    /// Merge another context into this one
    pub fn merge(&mut self, other: ProviderContext) {
        self.providers.extend(other.providers);
    }
}

impl std::fmt::Debug for ProviderContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderContext")
            .field("provider_count", &self.providers.len())
            .finish()
    }
}

// ============================================================================
// BLOC PROVIDER
// ============================================================================

/// Provides a bloc/cubit to its descendants
///
/// # Example
///
/// ```
/// use venom_core::{BlocProviderConfig, CubitCore};
///
/// let counter = CubitCore::new(0);
///
/// let config = BlocProviderConfig {
///     cubit: counter,
///     lazy: false, // Create immediately
/// };
/// ```
#[derive(Clone)]
pub struct BlocProviderConfig<B> {
    /// The bloc/cubit instance
    pub cubit: B,
    /// Whether to create lazily (on first access)
    pub lazy: bool,
}

impl<B> BlocProviderConfig<B> {
    /// Create a new provider config
    pub fn new(cubit: B) -> Self {
        Self { cubit, lazy: false }
    }

    /// Create with lazy initialization
    pub fn lazy(cubit: B) -> Self {
        Self { cubit, lazy: true }
    }
}

// ============================================================================
// BLOC BUILDER
// ============================================================================

/// Rebuilds its child when the bloc state changes
///
/// `BlocBuilder` subscribes to the bloc and calls the builder function
/// whenever the state changes.
///
/// # Type Parameters
///
/// * `B` - The bloc/cubit type
/// * `S` - The state type
///
/// # Example
///
/// ```
/// use venom_core::{BlocBuilderConfig, CubitCore, Cubit};
///
/// let counter = CubitCore::new(0);
///
/// let config = BlocBuilderConfig {
///     cubit: counter.clone(),
///     build_when: Some(Box::new(|old, new| old != new)),
/// };
///
/// // In real usage, this would integrate with the widget system
/// let current_state = config.cubit.state();
/// println!("Current count: {}", current_state);
/// ```
pub struct BlocBuilderConfig<S> {
    /// The cubit core to observe
    pub cubit: CubitCore<S>,
    /// Optional condition for when to rebuild
    pub build_when: Option<Box<dyn Fn(&S, &S) -> bool>>,
}

impl<S: Clone + PartialEq + 'static> BlocBuilderConfig<S> {
    /// Create a new builder config
    pub fn new(cubit: CubitCore<S>) -> Self {
        Self {
            cubit,
            build_when: None,
        }
    }

    /// Set a condition for when to rebuild
    pub fn build_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + 'static,
    {
        self.build_when = Some(Box::new(condition));
        self
    }

    /// Subscribe to state changes
    pub fn subscribe<F>(&self, on_change: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        // Note: In a full implementation, we'd filter based on build_when
        // For now, we always subscribe and let should_rebuild be called separately
        self.cubit.subscribe(on_change)
    }

    /// Check if should rebuild for the given state change
    pub fn should_rebuild(&self, old: &S, new: &S) -> bool {
        if let Some(condition) = &self.build_when {
            condition(old, new)
        } else {
            old != new
        }
    }
}

// ============================================================================
// BLOC LISTENER
// ============================================================================

/// Executes side effects in response to state changes
///
/// Unlike `BlocBuilder`, `BlocListener` doesn't rebuild widgets.
/// It's used for one-time reactions like navigation, showing dialogs, etc.
///
/// # Example
///
/// ```
/// use venom_core::{BlocListenerConfig, CubitCore};
///
/// let counter = CubitCore::new(0);
///
/// let config = BlocListenerConfig {
///     cubit: counter.clone(),
///     listen_when: Some(Box::new(|old, new| *new > 10)),
/// };
/// ```
pub struct BlocListenerConfig<S> {
    /// The cubit to listen to
    pub cubit: CubitCore<S>,
    /// Optional condition for when to trigger the listener
    pub listen_when: Option<Box<dyn Fn(&S, &S) -> bool>>,
}

impl<S: Clone + PartialEq + 'static> BlocListenerConfig<S> {
    /// Create a new listener config
    pub fn new(cubit: CubitCore<S>) -> Self {
        Self {
            cubit,
            listen_when: None,
        }
    }

    /// Set a condition for when to listen
    pub fn listen_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + 'static,
    {
        self.listen_when = Some(Box::new(condition));
        self
    }

    /// Subscribe with a listener callback
    pub fn listen<F>(&self, callback: F) -> SubscriptionHandle
    where
        F: Fn(&S, &S) + 'static,
    {
        // Note: In a full implementation, we'd filter based on listen_when
        // For now, we always listen and let should_listen be called separately
        self.cubit.subscribe(callback)
    }

    /// Check if should trigger for the given state change
    pub fn should_listen(&self, old: &S, new: &S) -> bool {
        if let Some(condition) = &self.listen_when {
            condition(old, new)
        } else {
            true
        }
    }
}

// ============================================================================
// BLOC CONSUMER
// ============================================================================

/// Combines BlocListener and BlocBuilder
///
/// Use when you need both to rebuild UI and execute side effects
/// based on state changes.
pub struct BlocConsumerConfig<S> {
    /// The cubit to consume
    pub cubit: CubitCore<S>,
    /// Condition for rebuilding
    pub build_when: Option<Box<dyn Fn(&S, &S) -> bool>>,
    /// Condition for listening
    pub listen_when: Option<Box<dyn Fn(&S, &S) -> bool>>,
}

impl<S: Clone + PartialEq + 'static> BlocConsumerConfig<S> {
    /// Create a new consumer config
    pub fn new(cubit: CubitCore<S>) -> Self {
        Self {
            cubit,
            build_when: None,
            listen_when: None,
        }
    }

    /// Set rebuild condition
    pub fn build_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + 'static,
    {
        self.build_when = Some(Box::new(condition));
        self
    }

    /// Set listen condition
    pub fn listen_when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&S, &S) -> bool + 'static,
    {
        self.listen_when = Some(Box::new(condition));
        self
    }
}

// ============================================================================
// MULTI BLOC PROVIDER
// ============================================================================

/// Provides multiple blocs/cubits at once
///
/// # Example
///
/// ```
/// use venom_core::{MultiProviderBuilder, CubitCore};
///
/// let counter = CubitCore::new(0);
/// let toggle = CubitCore::new(false);
///
/// let mut builder = MultiProviderBuilder::new();
/// builder.add(counter);
/// builder.add(toggle);
///
/// let context = builder.build();
///
/// // Now both cubits are available in the context
/// assert!(context.has::<CubitCore<i32>>());
/// assert!(context.has::<CubitCore<bool>>());
/// ```
#[derive(Default)]
pub struct MultiProviderBuilder {
    context: ProviderContext,
}

impl MultiProviderBuilder {
    /// Create a new multi-provider builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a provider
    pub fn add<T: 'static>(&mut self, value: T) -> &mut Self {
        self.context.provide(value);
        self
    }

    /// Build the final context
    pub fn build(self) -> ProviderContext {
        self.context
    }
}

// ============================================================================
// REPOSITORY PROVIDER
// ============================================================================

/// Provides a repository (data source) to descendants
///
/// Similar to BlocProvider but for data repositories.
/// Useful for dependency injection of data access layers.
///
/// # Example
///
/// ```
/// use venom_core::RepositoryProvider;
///
/// trait UserRepository {
///     fn get_user(&self, id: u32) -> Option<String>;
/// }
///
/// struct MockUserRepo;
/// impl UserRepository for MockUserRepo {
///     fn get_user(&self, id: u32) -> Option<String> {
///         Some(format!("User {}", id))
///     }
/// }
///
/// let provider = RepositoryProvider::new(MockUserRepo);
/// ```
pub struct RepositoryProvider<R> {
    repository: Rc<R>,
}

impl<R> RepositoryProvider<R> {
    /// Create a new repository provider
    pub fn new(repository: R) -> Self {
        Self {
            repository: Rc::new(repository),
        }
    }

    /// Get a reference to the repository
    pub fn get(&self) -> Rc<R> {
        Rc::clone(&self.repository)
    }
}

impl<R> Clone for RepositoryProvider<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Rc::clone(&self.repository),
        }
    }
}

// ============================================================================
// SCOPE
// ============================================================================

/// Scoped provider that disposes when dropped
///
/// Useful for feature-specific blocs that should be cleaned up
/// when the feature is no longer in use.
pub struct ScopedProvider<B> {
    bloc: Rc<RefCell<Option<B>>>,
    on_dispose: Option<Box<dyn FnOnce(&B)>>,
}

impl<B> ScopedProvider<B> {
    /// Create a new scoped provider
    pub fn new(bloc: B) -> Self {
        Self {
            bloc: Rc::new(RefCell::new(Some(bloc))),
            on_dispose: None,
        }
    }

    /// Set a callback to run when disposed
    pub fn on_dispose<F: FnOnce(&B) + 'static>(mut self, callback: F) -> Self {
        self.on_dispose = Some(Box::new(callback));
        self
    }

    /// Get a reference to the bloc
    pub fn get(&self) -> Option<std::cell::Ref<'_, B>> {
        let borrow = self.bloc.borrow();
        if borrow.is_some() {
            Some(std::cell::Ref::map(borrow, |o| o.as_ref().unwrap()))
        } else {
            None
        }
    }

    /// Manually dispose the bloc
    pub fn dispose(&mut self) {
        if let Some(bloc) = self.bloc.borrow_mut().take() {
            if let Some(on_dispose) = self.on_dispose.take() {
                on_dispose(&bloc);
            }
        }
    }
}

impl<B> Drop for ScopedProvider<B> {
    fn drop(&mut self) {
        self.dispose();
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_context() {
        let mut ctx = ProviderContext::new();
        
        ctx.provide(42i32);
        ctx.provide("hello".to_string());

        assert!(ctx.has::<i32>());
        assert!(ctx.has::<String>());
        assert!(!ctx.has::<f64>());

        assert_eq!(*ctx.get::<i32>().unwrap(), 42);
        assert_eq!(*ctx.get::<String>().unwrap(), "hello");
    }

    #[test]
    fn test_provider_context_remove() {
        let mut ctx = ProviderContext::new();
        ctx.provide(42i32);

        assert!(ctx.has::<i32>());
        assert!(ctx.remove::<i32>());
        assert!(!ctx.has::<i32>());
    }

    #[test]
    fn test_multi_provider_builder() {
        let mut builder = MultiProviderBuilder::new();
        
        builder
            .add(42i32)
            .add("hello".to_string())
            .add(3.14f64);

        let ctx = builder.build();

        assert!(ctx.has::<i32>());
        assert!(ctx.has::<String>());
        assert!(ctx.has::<f64>());
    }

    #[test]
    fn test_bloc_builder_config() {
        let cubit = CubitCore::new(0);
        let config = BlocBuilderConfig::new(cubit.clone())
            .build_when(|old, new| new > old);

        assert!(config.should_rebuild(&0, &1));
        assert!(!config.should_rebuild(&1, &0));
    }

    #[test]
    fn test_bloc_listener_config() {
        let cubit = CubitCore::new(0);
        let config = BlocListenerConfig::new(cubit.clone())
            .listen_when(|_, new| *new > 10);

        assert!(config.should_listen(&0, &15));
        assert!(!config.should_listen(&0, &5));
    }

    #[test]
    fn test_repository_provider() {
        struct TestRepo {
            data: Vec<i32>,
        }

        let repo = RepositoryProvider::new(TestRepo {
            data: vec![1, 2, 3],
        });

        let repo_ref = repo.get();
        assert_eq!(repo_ref.data, vec![1, 2, 3]);

        // Clone works
        let repo2 = repo.clone();
        assert_eq!(repo2.get().data, vec![1, 2, 3]);
    }

    #[test]
    fn test_scoped_provider() {
        use std::cell::Cell;

        let disposed = Rc::new(Cell::new(false));
        let disposed_clone = Rc::clone(&disposed);

        {
            let _provider = ScopedProvider::new(42i32)
                .on_dispose(move |_| disposed_clone.set(true));

            assert!(!disposed.get());
        } // provider dropped here

        assert!(disposed.get());
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = ProviderContext::new();
        ctx1.provide(42i32);

        let mut ctx2 = ProviderContext::new();
        ctx2.provide("hello".to_string());

        ctx1.merge(ctx2);

        assert!(ctx1.has::<i32>());
        assert!(ctx1.has::<String>());
    }
}
