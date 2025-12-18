pub mod channel_keys;
pub mod errors;
pub mod registry;

pub use channel_keys::*;
pub use errors::*;
pub use registry::*;

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
            Err(errors::RegistryError::TypeMismatch { key, .. }) => {
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
