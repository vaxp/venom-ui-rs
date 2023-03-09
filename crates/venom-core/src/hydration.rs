//! Hydration - State Persistence and Restoration
//!
//! This module provides tools for persisting state and restoring it later:
//!
//! - [`Hydratable`] - Trait for serializable state
//! - [`HydrationStorage`] - Storage backend trait
//! - [`MemoryStorage`] - In-memory storage for testing
//! - [`FileStorage`] - File-based persistent storage
//! - [`HydratedCubit`] - Cubit with automatic persistence
//!
//! # Example
//!
//! ```
//! use venom_core::{HydratedCubit, MemoryStorage, Hydratable};
//!
//! #[derive(Clone, PartialEq, Default)]
//! struct Counter(i32);
//!
//! impl Hydratable for Counter {
//!     fn dehydrate(&self) -> Vec<u8> {
//!         self.0.to_le_bytes().to_vec()
//!     }
//!
//!     fn hydrate(data: &[u8]) -> Option<Self> {
//!         if data.len() >= 4 {
//!             let bytes: [u8; 4] = data[..4].try_into().ok()?;
//!             Some(Counter(i32::from_le_bytes(bytes)))
//!         } else {
//!             None
//!         }
//!     }
//! }
//!
//! let storage = MemoryStorage::new();
//! let cubit = HydratedCubit::new("counter", storage, Counter(0));
//!
//! cubit.emit(Counter(42));
//! cubit.persist().unwrap();
//!
//! // Later, hydrate from storage
//! assert_eq!(cubit.state().0, 42);
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;

use crate::subscription::{StateStream, SubscriptionHandle};

// ============================================================================
// HYDRATABLE TRAIT
// ============================================================================

/// Trait for types that can be serialized/deserialized
///
/// Implement this trait to enable state persistence.
///
/// # Example
///
/// ```
/// use venom_core::Hydratable;
///
/// struct MyState {
///     count: i32,
///     name: String,
/// }
///
/// impl Hydratable for MyState {
///     fn dehydrate(&self) -> Vec<u8> {
///         // Simple format: 4 bytes for count + rest is name
///         let mut data = self.count.to_le_bytes().to_vec();
///         data.extend(self.name.as_bytes());
///         data
///     }
///
///     fn hydrate(data: &[u8]) -> Option<Self> {
///         if data.len() < 4 {
///             return None;
///         }
///         let count = i32::from_le_bytes(data[..4].try_into().ok()?);
///         let name = String::from_utf8(data[4..].to_vec()).ok()?;
///         Some(Self { count, name })
///     }
/// }
/// ```
pub trait Hydratable: Sized {
    /// Serialize state to bytes
    fn dehydrate(&self) -> Vec<u8>;

    /// Deserialize state from bytes
    fn hydrate(data: &[u8]) -> Option<Self>;
}

// ============================================================================
// HYDRATION ERROR
// ============================================================================

/// Errors that can occur during hydration/storage operations
#[derive(Debug, Clone)]
pub enum HydrationError {
    /// Storage key not found
    NotFound(String),
    /// Failed to read from storage
    ReadError(String),
    /// Failed to write to storage
    WriteError(String),
    /// Failed to deserialize data
    DeserializeError(String),
    /// Failed to serialize data
    SerializeError(String),
    /// IO error occurred
    IoError(String),
}

impl fmt::Display for HydrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(key) => write!(f, "Key not found: {}", key),
            Self::ReadError(msg) => write!(f, "Read error: {}", msg),
            Self::WriteError(msg) => write!(f, "Write error: {}", msg),
            Self::DeserializeError(msg) => write!(f, "Deserialize error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for HydrationError {}

// ============================================================================
// HYDRATION STORAGE TRAIT
// ============================================================================

/// Storage backend for hydration
///
/// Implement this trait to provide custom storage backends
/// (e.g., localStorage, IndexedDB, cloud storage).
pub trait HydrationStorage {
    /// Save data to storage
    fn save(&self, key: &str, data: &[u8]) -> Result<(), HydrationError>;

    /// Load data from storage
    fn load(&self, key: &str) -> Result<Vec<u8>, HydrationError>;

    /// Delete data from storage
    fn delete(&self, key: &str) -> Result<(), HydrationError>;

    /// Check if key exists
    fn exists(&self, key: &str) -> bool;

    /// List all keys (optional)
    fn keys(&self) -> Vec<String> {
        Vec::new()
    }

    /// Clear all data (optional)
    fn clear(&self) -> Result<(), HydrationError> {
        Ok(())
    }
}

// ============================================================================
// MEMORY STORAGE
// ============================================================================

/// In-memory storage (useful for testing)
///
/// Data is not persisted across application restarts.
///
/// # Example
///
/// ```
/// use venom_core::{MemoryStorage, HydrationStorage};
///
/// let storage = MemoryStorage::new();
///
/// storage.save("key", b"value").unwrap();
/// assert_eq!(storage.load("key").unwrap(), b"value");
///
/// storage.delete("key").unwrap();
/// assert!(!storage.exists("key"));
/// ```
#[derive(Default)]
pub struct MemoryStorage {
    data: RefCell<HashMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a snapshot of all data
    pub fn snapshot(&self) -> HashMap<String, Vec<u8>> {
        self.data.borrow().clone()
    }

    /// Restore from snapshot
    pub fn restore(&self, snapshot: HashMap<String, Vec<u8>>) {
        *self.data.borrow_mut() = snapshot;
    }
}

impl HydrationStorage for MemoryStorage {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), HydrationError> {
        self.data.borrow_mut().insert(key.to_string(), data.to_vec());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Vec<u8>, HydrationError> {
        self.data
            .borrow()
            .get(key)
            .cloned()
            .ok_or_else(|| HydrationError::NotFound(key.to_string()))
    }

    fn delete(&self, key: &str) -> Result<(), HydrationError> {
        self.data.borrow_mut().remove(key);
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        self.data.borrow().contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        self.data.borrow().keys().cloned().collect()
    }

    fn clear(&self) -> Result<(), HydrationError> {
        self.data.borrow_mut().clear();
        Ok(())
    }
}

impl Clone for MemoryStorage {
    fn clone(&self) -> Self {
        Self {
            data: RefCell::new(self.data.borrow().clone()),
        }
    }
}

// Implement HydrationStorage for Rc<MemoryStorage> to allow sharing storage
impl HydrationStorage for Rc<MemoryStorage> {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), HydrationError> {
        (**self).save(key, data)
    }

    fn load(&self, key: &str) -> Result<Vec<u8>, HydrationError> {
        (**self).load(key)
    }

    fn delete(&self, key: &str) -> Result<(), HydrationError> {
        (**self).delete(key)
    }

    fn exists(&self, key: &str) -> bool {
        (**self).exists(key)
    }

    fn keys(&self) -> Vec<String> {
        (**self).keys()
    }

    fn clear(&self) -> Result<(), HydrationError> {
        (**self).clear()
    }
}

// ============================================================================
// FILE STORAGE
// ============================================================================

/// File-based storage for persistent state
///
/// Stores each key as a separate file in a directory.
///
/// # Example
///
/// ```ignore
/// use venom_core::{FileStorage, HydrationStorage};
///
/// let storage = FileStorage::new("./state");
///
/// storage.save("counter", b"42").unwrap();
/// // Creates file: ./state/counter
/// ```
pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    /// Create a new file storage with base path
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn key_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

impl HydrationStorage for FileStorage {
    fn save(&self, key: &str, data: &[u8]) -> Result<(), HydrationError> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.base_path)
            .map_err(|e| HydrationError::IoError(e.to_string()))?;

        std::fs::write(self.key_path(key), data)
            .map_err(|e| HydrationError::WriteError(e.to_string()))
    }

    fn load(&self, key: &str) -> Result<Vec<u8>, HydrationError> {
        std::fs::read(self.key_path(key)).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                HydrationError::NotFound(key.to_string())
            } else {
                HydrationError::ReadError(e.to_string())
            }
        })
    }

    fn delete(&self, key: &str) -> Result<(), HydrationError> {
        let path = self.key_path(key);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| HydrationError::IoError(e.to_string()))?;
        }
        Ok(())
    }

    fn exists(&self, key: &str) -> bool {
        self.key_path(key).exists()
    }

    fn keys(&self) -> Vec<String> {
        std::fs::read_dir(&self.base_path)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn clear(&self) -> Result<(), HydrationError> {
        if self.base_path.exists() {
            for key in self.keys() {
                self.delete(&key)?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// HYDRATED CUBIT
// ============================================================================

/// Cubit with automatic state persistence
///
/// Automatically saves state to storage and can restore it on startup.
///
/// # Example
///
/// ```
/// use venom_core::{HydratedCubit, MemoryStorage, Hydratable};
///
/// #[derive(Clone, PartialEq, Default)]
/// struct AppState {
///     theme: String,
///     volume: u8,
/// }
///
/// impl Hydratable for AppState {
///     fn dehydrate(&self) -> Vec<u8> {
///         let mut data = Vec::new();
///         data.push(self.volume);
///         data.extend(self.theme.as_bytes());
///         data
///     }
///
///     fn hydrate(data: &[u8]) -> Option<Self> {
///         if data.is_empty() {
///             return None;
///         }
///         Some(Self {
///             volume: data[0],
///             theme: String::from_utf8(data[1..].to_vec()).ok()?,
///         })
///     }
/// }
///
/// let storage = MemoryStorage::new();
/// let cubit = HydratedCubit::new("settings", storage, AppState::default());
///
/// cubit.emit(AppState { theme: "dark".into(), volume: 80 });
/// cubit.persist().unwrap();
/// ```
pub struct HydratedCubit<S: Hydratable> {
    stream: StateStream<S>,
    storage: Rc<dyn HydrationStorage>,
    key: String,
    auto_persist: bool,
}

impl<S: Hydratable + Clone + PartialEq + 'static> HydratedCubit<S> {
    /// Create a new hydrated cubit
    ///
    /// If a persisted state exists in storage, it will be loaded.
    /// Otherwise, the default value is used.
    pub fn new(key: impl Into<String>, storage: impl HydrationStorage + 'static, default: S) -> Self {
        let key = key.into();
        let storage = Rc::new(storage) as Rc<dyn HydrationStorage>;

        // Try to hydrate from storage
        let initial = storage
            .load(&key)
            .ok()
            .and_then(|data| S::hydrate(&data))
            .unwrap_or(default);

        Self {
            stream: StateStream::new(initial),
            storage,
            key,
            auto_persist: false,
        }
    }

    /// Create with auto-persist enabled
    ///
    /// Every state change is automatically persisted.
    pub fn auto_persisted(
        key: impl Into<String>,
        storage: impl HydrationStorage + 'static,
        default: S,
    ) -> Self {
        let mut cubit = Self::new(key, storage, default);
        cubit.auto_persist = true;
        cubit
    }

    /// Enable/disable auto-persist
    pub fn set_auto_persist(&mut self, enabled: bool) {
        self.auto_persist = enabled;
    }

    /// Get current state
    pub fn state(&self) -> S {
        self.stream.state()
    }

    /// Get state version
    pub fn version(&self) -> u64 {
        self.stream.version()
    }

    /// Emit a new state
    pub fn emit(&self, state: S) -> bool {
        let changed = self.stream.emit(state);
        
        if changed && self.auto_persist {
            let _ = self.persist();
        }
        
        changed
    }

    /// Update state with function
    pub fn update<F: FnOnce(&S) -> S>(&self, f: F) -> bool {
        let new_state = {
            self.stream.with_state(f)
        };
        self.emit(new_state)
    }

    /// Persist current state to storage
    pub fn persist(&self) -> Result<(), HydrationError> {
        let state = self.stream.state();
        let data = state.dehydrate();
        self.storage.save(&self.key, &data)
    }

    /// Reload state from storage
    pub fn reload(&self) -> Result<bool, HydrationError> {
        let data = self.storage.load(&self.key)?;
        if let Some(state) = S::hydrate(&data) {
            self.stream.emit(state);
            Ok(true)
        } else {
            Err(HydrationError::DeserializeError(
                "Failed to hydrate state".into(),
            ))
        }
    }

    /// Clear persisted state
    pub fn clear(&self) -> Result<(), HydrationError> {
        self.storage.delete(&self.key)
    }

    /// Get storage key
    pub fn key(&self) -> &str {
        &self.key
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

impl<S: Hydratable + std::fmt::Debug> std::fmt::Debug for HydratedCubit<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HydratedCubit")
            .field("key", &self.key)
            .field("auto_persist", &self.auto_persist)
            .field("stream", &self.stream)
            .finish()
    }
}

// ============================================================================
// HYDRATABLE IMPLEMENTATIONS FOR COMMON TYPES
// ============================================================================

impl Hydratable for String {
    fn dehydrate(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        String::from_utf8(data.to_vec()).ok()
    }
}

impl Hydratable for i32 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 4 {
            Some(i32::from_le_bytes(data[..4].try_into().ok()?))
        } else {
            None
        }
    }
}

impl Hydratable for i64 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 8 {
            Some(i64::from_le_bytes(data[..8].try_into().ok()?))
        } else {
            None
        }
    }
}

impl Hydratable for u32 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 4 {
            Some(u32::from_le_bytes(data[..4].try_into().ok()?))
        } else {
            None
        }
    }
}

impl Hydratable for u64 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 8 {
            Some(u64::from_le_bytes(data[..8].try_into().ok()?))
        } else {
            None
        }
    }
}

impl Hydratable for bool {
    fn dehydrate(&self) -> Vec<u8> {
        vec![if *self { 1 } else { 0 }]
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        data.first().map(|b| *b != 0)
    }
}

impl Hydratable for f32 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 4 {
            Some(f32::from_le_bytes(data[..4].try_into().ok()?))
        } else {
            None
        }
    }
}

impl Hydratable for f64 {
    fn dehydrate(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn hydrate(data: &[u8]) -> Option<Self> {
        if data.len() >= 8 {
            Some(f64::from_le_bytes(data[..8].try_into().ok()?))
        } else {
            None
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage() {
        let storage = MemoryStorage::new();

        storage.save("key1", b"value1").unwrap();
        storage.save("key2", b"value2").unwrap();

        assert!(storage.exists("key1"));
        assert!(storage.exists("key2"));
        assert!(!storage.exists("key3"));

        assert_eq!(storage.load("key1").unwrap(), b"value1");
        assert_eq!(storage.load("key2").unwrap(), b"value2");

        storage.delete("key1").unwrap();
        assert!(!storage.exists("key1"));
    }

    #[test]
    fn test_memory_storage_keys() {
        let storage = MemoryStorage::new();

        storage.save("a", b"1").unwrap();
        storage.save("b", b"2").unwrap();
        storage.save("c", b"3").unwrap();

        let mut keys = storage.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_hydratable_i32() {
        let value: i32 = 42;
        let data = value.dehydrate();
        let restored = i32::hydrate(&data).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_hydratable_string() {
        let value = "Hello, World!".to_string();
        let data = value.dehydrate();
        let restored = String::hydrate(&data).unwrap();
        assert_eq!(value, restored);
    }

    #[test]
    fn test_hydratable_bool() {
        assert_eq!(bool::hydrate(&true.dehydrate()), Some(true));
        assert_eq!(bool::hydrate(&false.dehydrate()), Some(false));
    }

    #[test]
    fn test_hydrated_cubit() {
        let storage = Rc::new(MemoryStorage::new());
        let cubit = HydratedCubit::new("test", storage.clone(), 0i32);

        assert_eq!(cubit.state(), 0);

        cubit.emit(42);
        assert_eq!(cubit.state(), 42);

        cubit.persist().unwrap();

        // Verify data was saved - use the same Rc-wrapped storage
        let saved_data = storage.load("test").unwrap();
        assert_eq!(i32::hydrate(&saved_data), Some(42));
    }

    #[test]
    fn test_hydrated_cubit_auto_persist() {
        let storage = Rc::new(MemoryStorage::new());
        let cubit = HydratedCubit::auto_persisted("counter", storage.clone(), 0i32);

        cubit.emit(10);
        cubit.emit(20);
        cubit.emit(30);

        // Should have auto-persisted - use the same Rc-wrapped storage
        let saved = storage.load("counter").unwrap();
        assert_eq!(i32::hydrate(&saved), Some(30));
    }

    #[test]
    fn test_hydrated_cubit_reload() {
        let storage = MemoryStorage::new();

        // Save initial data
        storage.save("reload_test", &100i32.dehydrate()).unwrap();

        // Create cubit - should load persisted value
        let cubit = HydratedCubit::new("reload_test", storage.clone(), 0i32);
        assert_eq!(cubit.state(), 100);
    }

    #[test]
    fn test_hydration_error_display() {
        let err = HydrationError::NotFound("key".into());
        assert_eq!(format!("{}", err), "Key not found: key");
    }
}
