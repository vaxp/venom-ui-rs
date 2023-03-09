//! Selectors - Derived State with Memoization
//!
//! Selectors allow you to derive and memoize computed values from state.
//! They only recompute when the selected portion of the state actually changes.
//!
//! # Benefits
//!
//! - **Performance**: Avoid unnecessary widget rebuilds
//! - **Reusability**: Define selection logic once, use everywhere
//! - **Composition**: Combine selectors to create complex derivations
//!
//! # Example
//!
//! ```
//! use venom_core::Selector;
//!
//! #[derive(Clone, PartialEq)]
//! struct AppState {
//!     items: Vec<String>,
//!     filter: String,
//! }
//!
//! // Selector for filtered items
//! let filtered_items = Selector::new(|state: &AppState| {
//!     state.items.iter()
//!         .filter(|item| item.contains(&state.filter))
//!         .cloned()
//!         .collect::<Vec<_>>()
//! });
//!
//! let state = AppState {
//!     items: vec!["apple".into(), "banana".into(), "apricot".into()],
//!     filter: "ap".into(),
//! };
//!
//! let result = filtered_items.select(&state);
//! assert_eq!(result, vec!["apple".to_string(), "apricot".to_string()]);
//! ```

use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// SELECTOR
// ============================================================================

/// A memoized selector for deriving values from state
///
/// Selectors cache their results and only recompute when
/// the input state changes (based on equality check).
///
/// # Type Parameters
///
/// * `S` - The source state type
/// * `T` - The derived/selected value type
///
/// # Example
///
/// ```
/// use venom_core::Selector;
///
/// let double = Selector::new(|n: &i32| n * 2);
///
/// assert_eq!(double.select(&5), 10);
/// assert_eq!(double.select(&10), 20);
/// ```
pub struct Selector<S, T> {
    select_fn: Box<dyn Fn(&S) -> T>,
    cache: RefCell<Option<(S, T)>>,
}

impl<S: Clone + PartialEq, T: Clone> Selector<S, T> {
    /// Create a new selector
    ///
    /// # Arguments
    ///
    /// * `select_fn` - Function that extracts/derives a value from state
    pub fn new<F>(select_fn: F) -> Self
    where
        F: Fn(&S) -> T + 'static,
    {
        Self {
            select_fn: Box::new(select_fn),
            cache: RefCell::new(None),
        }
    }

    /// Select a value from the state
    ///
    /// Returns cached value if state hasn't changed since last call.
    pub fn select(&self, state: &S) -> T {
        let mut cache = self.cache.borrow_mut();
        
        if let Some((cached_state, cached_value)) = cache.as_ref() {
            if cached_state == state {
                return cached_value.clone();
            }
        }

        let value = (self.select_fn)(state);
        *cache = Some((state.clone(), value.clone()));
        value
    }

    /// Check if the selection would produce a different value
    ///
    /// Useful for determining if a rebuild is necessary.
    pub fn would_change(&self, state: &S) -> bool {
        let cache = self.cache.borrow();
        
        if let Some((cached_state, _)) = cache.as_ref() {
            cached_state != state
        } else {
            true // No cache, so it would "change"
        }
    }

    /// Clear the cached value
    pub fn invalidate(&self) {
        *self.cache.borrow_mut() = None;
    }
}

impl<S, T> std::fmt::Debug for Selector<S, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Selector")
            .field("cached", &self.cache.borrow().is_some())
            .finish()
    }
}

// ============================================================================
// SELECTOR WITH EQUALITY CHECK
// ============================================================================

/// Selector that also checks if the selected value changed
///
/// This is useful when you want to avoid rebuilds even if the
/// source state changed but the derived value is the same.
///
/// # Example
///
/// ```
/// use venom_core::SelectorEq;
///
/// #[derive(Clone, PartialEq)]
/// struct State {
///     count: i32,
///     name: String,
/// }
///
/// // Only care about count
/// let count_selector = SelectorEq::new(|s: &State| s.count);
///
/// let state1 = State { count: 5, name: "Alice".into() };
/// let state2 = State { count: 5, name: "Bob".into() };
///
/// count_selector.select(&state1);
/// // This won't trigger a "change" even though state changed
/// assert!(!count_selector.value_changed(&state2));
/// ```
pub struct SelectorEq<S, T> {
    select_fn: Box<dyn Fn(&S) -> T>,
    cache: RefCell<Option<(S, T)>>,
}

impl<S: Clone + PartialEq, T: Clone + PartialEq> SelectorEq<S, T> {
    /// Create a new selector with equality checking
    pub fn new<F>(select_fn: F) -> Self
    where
        F: Fn(&S) -> T + 'static,
    {
        Self {
            select_fn: Box::new(select_fn),
            cache: RefCell::new(None),
        }
    }

    /// Select a value from the state
    pub fn select(&self, state: &S) -> T {
        let mut cache = self.cache.borrow_mut();
        
        if let Some((cached_state, cached_value)) = cache.as_ref() {
            if cached_state == state {
                return cached_value.clone();
            }
        }

        let value = (self.select_fn)(state);
        *cache = Some((state.clone(), value.clone()));
        value
    }

    /// Check if the selected value has changed
    ///
    /// This compares the new selected value with the cached one,
    /// not just the source state.
    pub fn value_changed(&self, state: &S) -> bool {
        let cache = self.cache.borrow();
        
        if let Some((_, cached_value)) = cache.as_ref() {
            let new_value = (self.select_fn)(state);
            new_value != *cached_value
        } else {
            true
        }
    }

    /// Clear the cache
    pub fn invalidate(&self) {
        *self.cache.borrow_mut() = None;
    }
}

// ============================================================================
// COMPOSED SELECTOR
// ============================================================================

/// Compose two selectors together
///
/// First applies selector A to get an intermediate value,
/// then applies selector B to that value.
///
/// # Example
///
/// ```
/// use venom_core::{Selector, ComposedSelector};
///
/// let get_list = Selector::new(|s: &(Vec<i32>, String)| s.0.clone());
/// let get_length = Selector::new(|v: &Vec<i32>| v.len());
///
/// let composed = ComposedSelector::new(get_list, get_length);
///
/// let state = (vec![1, 2, 3, 4, 5], "hello".into());
/// assert_eq!(composed.select(&state), 5);
/// ```
pub struct ComposedSelector<S, M, T> {
    first: Selector<S, M>,
    second: Selector<M, T>,
}

impl<S: Clone + PartialEq, M: Clone + PartialEq, T: Clone> ComposedSelector<S, M, T> {
    /// Create a composed selector from two selectors
    pub fn new(first: Selector<S, M>, second: Selector<M, T>) -> Self {
        Self { first, second }
    }

    /// Select through both selectors
    pub fn select(&self, state: &S) -> T {
        let intermediate = self.first.select(state);
        self.second.select(&intermediate)
    }
}

// ============================================================================
// MULTI SELECTOR
// ============================================================================

/// Combine multiple source values into one
///
/// Useful when your derived value depends on multiple pieces of state.
///
/// # Example
///
/// ```
/// use venom_core::Selector2;
///
/// let selector = Selector2::new(
///     |s: &(i32, String)| s.0,           // First selector
///     |s: &(i32, String)| s.1.len(),     // Second selector
///     |a, b| a + b as i32,               // Combiner
/// );
///
/// let state = (10, "hello".to_string());
/// assert_eq!(selector.select(&state), 15); // 10 + 5
/// ```
pub struct Selector2<S, A, B, T> {
    select_a: Box<dyn Fn(&S) -> A>,
    select_b: Box<dyn Fn(&S) -> B>,
    combine: Box<dyn Fn(A, B) -> T>,
    cache: RefCell<Option<(S, T)>>,
}

impl<S: Clone + PartialEq, A, B, T: Clone> Selector2<S, A, B, T> {
    /// Create a selector that combines two derived values
    pub fn new<FA, FB, FC>(select_a: FA, select_b: FB, combine: FC) -> Self
    where
        FA: Fn(&S) -> A + 'static,
        FB: Fn(&S) -> B + 'static,
        FC: Fn(A, B) -> T + 'static,
    {
        Self {
            select_a: Box::new(select_a),
            select_b: Box::new(select_b),
            combine: Box::new(combine),
            cache: RefCell::new(None),
        }
    }

    /// Select and combine values
    pub fn select(&self, state: &S) -> T {
        let mut cache = self.cache.borrow_mut();
        
        if let Some((cached_state, cached_value)) = cache.as_ref() {
            if cached_state == state {
                return cached_value.clone();
            }
        }

        let a = (self.select_a)(state);
        let b = (self.select_b)(state);
        let value = (self.combine)(a, b);
        *cache = Some((state.clone(), value.clone()));
        value
    }
}

/// Combine three source values
pub struct Selector3<S, A, B, C, T> {
    select_a: Box<dyn Fn(&S) -> A>,
    select_b: Box<dyn Fn(&S) -> B>,
    select_c: Box<dyn Fn(&S) -> C>,
    combine: Box<dyn Fn(A, B, C) -> T>,
    cache: RefCell<Option<(S, T)>>,
}

impl<S: Clone + PartialEq, A, B, C, T: Clone> Selector3<S, A, B, C, T> {
    /// Create a selector that combines three derived values
    pub fn new<FA, FB, FC, FCombine>(select_a: FA, select_b: FB, select_c: FC, combine: FCombine) -> Self
    where
        FA: Fn(&S) -> A + 'static,
        FB: Fn(&S) -> B + 'static,
        FC: Fn(&S) -> C + 'static,
        FCombine: Fn(A, B, C) -> T + 'static,
    {
        Self {
            select_a: Box::new(select_a),
            select_b: Box::new(select_b),
            select_c: Box::new(select_c),
            combine: Box::new(combine),
            cache: RefCell::new(None),
        }
    }

    /// Select and combine values
    pub fn select(&self, state: &S) -> T {
        let mut cache = self.cache.borrow_mut();
        
        if let Some((cached_state, cached_value)) = cache.as_ref() {
            if cached_state == state {
                return cached_value.clone();
            }
        }

        let a = (self.select_a)(state);
        let b = (self.select_b)(state);
        let c = (self.select_c)(state);
        let value = (self.combine)(a, b, c);
        *cache = Some((state.clone(), value.clone()));
        value
    }
}

// ============================================================================
// RESELECT-STYLE HELPERS
// ============================================================================

/// Create a simple field selector
///
/// # Example
///
/// ```
/// use venom_core::field_selector;
///
/// #[derive(Clone, PartialEq)]
/// struct User { name: String, age: u32 }
///
/// let name_selector = field_selector!(User, name, String);
/// let user = User { name: "Alice".into(), age: 30 };
/// assert_eq!(name_selector.select(&user), "Alice");
/// ```
#[macro_export]
macro_rules! field_selector {
    ($state:ty, $field:ident, $field_type:ty) => {
        $crate::Selector::<$state, $field_type>::new(|s| s.$field.clone())
    };
}

/// Create a selector from a closure with explicit types
///
/// # Example
///
/// ```
/// use venom_core::create_selector;
///
/// let double = create_selector!(i32 => i32, |n| n * 2);
/// assert_eq!(double.select(&5), 10);
/// ```
#[macro_export]
macro_rules! create_selector {
    ($source:ty => $target:ty, $closure:expr) => {
        $crate::Selector::<$source, $target>::new($closure)
    };
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_basic() {
        let double = Selector::new(|n: &i32| n * 2);
        
        assert_eq!(double.select(&5), 10);
        assert_eq!(double.select(&10), 20);
    }

    #[test]
    fn test_selector_caching() {
        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let selector = Selector::new(move |n: &i32| {
            *call_count_clone.borrow_mut() += 1;
            n * 2
        });

        // First call computes
        assert_eq!(selector.select(&5), 10);
        assert_eq!(*call_count.borrow(), 1);

        // Second call with same input uses cache
        assert_eq!(selector.select(&5), 10);
        assert_eq!(*call_count.borrow(), 1);

        // Different input recomputes
        assert_eq!(selector.select(&10), 20);
        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_selector_would_change() {
        let selector = Selector::new(|n: &i32| n * 2);
        
        selector.select(&5);
        
        assert!(!selector.would_change(&5));
        assert!(selector.would_change(&10));
    }

    #[test]
    fn test_selector_eq_value_changed() {
        #[derive(Clone, PartialEq)]
        struct State {
            count: i32,
            name: String,
        }

        let selector = SelectorEq::new(|s: &State| s.count);

        let state1 = State { count: 5, name: "Alice".into() };
        let state2 = State { count: 5, name: "Bob".into() };
        let state3 = State { count: 10, name: "Alice".into() };

        selector.select(&state1);

        // Same count, different name - no value change
        assert!(!selector.value_changed(&state2));
        
        // Different count - value changed
        assert!(selector.value_changed(&state3));
    }

    #[test]
    fn test_selector2() {
        let selector = Selector2::new(
            |s: &(i32, i32)| s.0,
            |s: &(i32, i32)| s.1,
            |a, b| a + b,
        );

        assert_eq!(selector.select(&(10, 20)), 30);
        assert_eq!(selector.select(&(5, 5)), 10);
    }

    #[test]
    fn test_composed_selector() {
        let get_vec = Selector::new(|s: &Vec<i32>| s.clone());
        let get_sum = Selector::new(|v: &Vec<i32>| v.iter().sum::<i32>());

        let composed = ComposedSelector::new(get_vec, get_sum);

        assert_eq!(composed.select(&vec![1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_invalidate() {
        let call_count = Rc::new(RefCell::new(0));
        let call_count_clone = Rc::clone(&call_count);

        let selector = Selector::new(move |n: &i32| {
            *call_count_clone.borrow_mut() += 1;
            n * 2
        });

        selector.select(&5);
        assert_eq!(*call_count.borrow(), 1);

        selector.invalidate();

        selector.select(&5);
        assert_eq!(*call_count.borrow(), 2); // Recomputed after invalidate
    }

    #[test]
    fn test_real_world_example() {
        #[derive(Clone, PartialEq)]
        struct TodoState {
            items: Vec<String>,
            filter: String,
            show_completed: bool,
        }

        let filtered_items = Selector::new(|state: &TodoState| {
            state.items.iter()
                .filter(|item| item.to_lowercase().contains(&state.filter.to_lowercase()))
                .cloned()
                .collect::<Vec<_>>()
        });

        let state = TodoState {
            items: vec!["Buy milk".into(), "Clean room".into(), "Buy bread".into()],
            filter: "buy".into(),
            show_completed: false,
        };

        let result = filtered_items.select(&state);
        assert_eq!(result, vec!["Buy milk".to_string(), "Buy bread".to_string()]);
    }
}
