use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Errors that can occur during registry operations
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    KeyNotFound(String),
    TypeMismatch {
        key: String,
        expected: &'static str,
        found: &'static str,
    },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::KeyNotFound(key) => write!(f, "Key '{}' not found in registry", key),
            RegistryError::TypeMismatch {
                key,
                expected,
                found,
            } => write!(
                f,
                "Type mismatch for key '{}': expected {}, found {}",
                key, expected, found
            ),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Trait for readers that can read values of type T
pub trait Reader<T> {
    fn read(&self) -> T;
}

/// Trait for keys that can create readers
pub trait InputKeys<T> {
    type ReaderType: Reader<T>;
    fn reader(&self, registry: &ChannelRegistry) -> Result<Self::ReaderType, RegistryError>;
}

/// Trait for writers that can write values of type T
pub trait Writer<T> {
    fn write(&self, output: &T);
}

/// Trait for keys that can create writers
pub trait OutputKeys<T> {
    type WriterType: Writer<T>;
    fn writer(&self, registry: &ChannelRegistry) -> Result<Self::WriterType, RegistryError>;
    fn register(&self, registry: &mut ChannelRegistry);
}

/// The registry for storing typed values
pub struct ChannelRegistry {
    store: HashMap<String, Rc<dyn Any>>,
}

impl ChannelRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Put a value into the registry
    pub fn put<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        let key = key.into();
        self.store.insert(key, Rc::new(RefCell::new(value)));
    }

    /// Get a value from the registry
    pub fn get<T: 'static>(&self, key: impl AsRef<str>) -> Result<Rc<RefCell<T>>, RegistryError> {
        let key = key.as_ref();

        match self.store.get(key) {
            Some(value) => {
                // The value is stored as Rc<dyn Any>, but actually contains Rc<RefCell<T>>
                // We need to downcast the Rc itself
                value
                    .clone()
                    .downcast::<RefCell<T>>()
                    .map_err(|_| RegistryError::TypeMismatch {
                        key: key.to_string(),
                        expected: std::any::type_name::<T>(),
                        found: "unknown",
                    })
            }
            None => Err(RegistryError::KeyNotFound(key.to_string())),
        }
    }

    /// Ensure a key exists in the registry, creating it with Default if it doesn't
    pub fn ensure<T: Default + 'static>(&mut self, key: impl Into<String>) -> Rc<RefCell<T>> {
        let key = key.into();

        // Check if key already exists and try to get it
        if let Ok(existing) = self.get::<T>(&key) {
            return existing;
        }

        // Key doesn't exist or wrong type, create new entry
        let value = Rc::new(RefCell::new(T::default()));
        self.store.insert(key, value.clone());
        value
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut registry = ChannelRegistry::new();

        // Put a value
        registry.put("test_key", 42i32);

        // Get it back
        let value = registry.get::<i32>("test_key").unwrap();
        assert_eq!(*value.borrow(), 42);
    }

    #[test]
    fn test_put_and_get_string() {
        let mut registry = ChannelRegistry::new();

        registry.put("message", "Hello, World!".to_string());

        let value = registry.get::<String>("message").unwrap();
        assert_eq!(*value.borrow(), "Hello, World!");
    }

    #[test]
    fn test_get_nonexistent_key() {
        let registry = ChannelRegistry::new();

        let result = registry.get::<i32>("missing");
        assert_eq!(
            result,
            Err(RegistryError::KeyNotFound("missing".to_string()))
        );
    }

    #[test]
    fn test_get_wrong_type() {
        let mut registry = ChannelRegistry::new();

        registry.put("number", 42i32);

        let result = registry.get::<String>("number");
        match result {
            Err(RegistryError::TypeMismatch { key, .. }) => {
                assert_eq!(key, "number");
            }
            _ => panic!("Expected TypeMismatch error"),
        }
    }

    #[test]
    fn test_ensure_new_key() {
        let mut registry = ChannelRegistry::new();

        let value = registry.ensure::<i32>("new_key");
        assert_eq!(*value.borrow(), 0); // Default for i32

        // Should be able to get it back
        let retrieved = registry.get::<i32>("new_key").unwrap();
        assert_eq!(*retrieved.borrow(), 0);
    }

    #[test]
    fn test_ensure_existing_key() {
        let mut registry = ChannelRegistry::new();

        registry.put("existing", 42i32);

        let value = registry.ensure::<i32>("existing");
        assert_eq!(*value.borrow(), 42); // Should get existing value

        // Modify through ensure reference
        *value.borrow_mut() = 100;

        // Should see change when getting again
        let retrieved = registry.get::<i32>("existing").unwrap();
        assert_eq!(*retrieved.borrow(), 100);
    }

    #[test]
    fn test_ensure_with_custom_default() {
        let mut registry = ChannelRegistry::new();

        #[derive(Default, PartialEq, Debug)]
        struct CustomStruct {
            value: i32,
        }

        let custom = registry.ensure::<CustomStruct>("custom");
        assert_eq!(*custom.borrow(), CustomStruct { value: 0 });
    }

    #[test]
    fn test_mutable_access() {
        let mut registry = ChannelRegistry::new();

        registry.put("counter", 0i32);

        let counter = registry.get::<i32>("counter").unwrap();
        *counter.borrow_mut() += 1;

        // The same Rc<RefCell<T>> should show the updated value
        assert_eq!(*counter.borrow(), 1);

        // Getting again should also show the updated value
        let updated = registry.get::<i32>("counter").unwrap();
        assert_eq!(*updated.borrow(), 1);
    }

    #[test]
    fn test_multiple_references() {
        let mut registry = ChannelRegistry::new();

        registry.put("shared", vec![1, 2, 3]);

        let ref1 = registry.get::<Vec<i32>>("shared").unwrap();
        let ref2 = registry.get::<Vec<i32>>("shared").unwrap();

        // Both should see the same data
        assert_eq!(*ref1.borrow(), vec![1, 2, 3]);
        assert_eq!(*ref2.borrow(), vec![1, 2, 3]);

        // Modify through one reference
        ref1.borrow_mut().push(4);

        // Other reference should see the change
        assert_eq!(*ref2.borrow(), vec![1, 2, 3, 4]);
    }
}
