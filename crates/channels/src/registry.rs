use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::errors;

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

    pub fn has(&self, key: impl Into<String>) -> bool {
        let key = key.into();
        self.store.contains_key(&key)
    }

    /// Put a value into the registry
    pub fn put<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        let key = key.into();
        self.store.insert(key, Rc::new(RefCell::new(value)));
    }

    /// Get a value from the registry
    pub fn get<T: 'static>(
        &self,
        key: impl AsRef<str>,
    ) -> Result<Rc<RefCell<T>>, errors::RegistryError> {
        let key = key.as_ref();

        match self.store.get(key) {
            Some(value) => {
                // The value is stored as Rc<dyn Any>, but actually contains Rc<RefCell<T>>
                // We need to downcast the Rc itself
                value.clone().downcast::<RefCell<T>>().map_err(|_| {
                    errors::RegistryError::TypeMismatch {
                        key: key.to_string(),
                        expected: std::any::type_name::<T>(),
                        found: "unknown",
                    }
                })
            }
            None => Err(errors::RegistryError::KeyNotFound(key.to_string())),
        }
    }

    /// Ensure a key exists in the registry, creating it with Default if it doesn't.
    /// Returns the Rc<RefCell<T>> for the key. If the key exists but has the wrong type,
    /// an error is returned.
    pub fn ensure<T: Default + 'static>(
        &mut self,
        key: impl Into<String>,
    ) -> Result<Rc<RefCell<T>>, errors::RegistryError> {
        let key = key.into();

        // Check if key already exists and try to get it
        if let Ok(existing) = self.get::<T>(&key) {
            return Ok(existing);
        }

        if self.store.contains_key(&key) {
            // Key exists but type mismatch
            return Err(errors::RegistryError::TypeMismatch {
                key,
                expected: std::any::type_name::<T>(),
                found: "Unexpected Type",
            });
        }

        // Key doesn't exist create new entry
        let value = Rc::new(RefCell::new(T::default()));
        self.store.insert(key, value.clone());
        Ok(value)
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
