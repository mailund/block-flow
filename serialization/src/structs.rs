//! Struct serialization
//!
//! This module provides traits and implementations for serializing structs.
//! Generic serialization support for any serde-compatible struct.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::error::Result;

/// Trait for serializable structs
///
/// This allows any struct with Serialize and Deserialize to be serialized uniformly.
///
/// # Examples
///
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use serialization::structs::SerializableStruct;
///
/// #[derive(Serialize, Deserialize)]
/// struct MyConfig {
///     pub field_a: String,
///     pub field_b: i32,
/// }
///
/// impl SerializableStruct for MyConfig {}
/// ```
pub trait SerializableStruct: Serialize + for<'de> Deserialize<'de> {}

/// Trait for serializing structs
///
/// This trait allows different serialization backends (JSON, TOML, etc.)
/// to be used for struct serialization.
pub trait StructSerializer {
    /// Serialize struct to bytes
    fn serialize<S: SerializableStruct>(&self, data: &S) -> Result<Vec<u8>>;

    /// Deserialize struct from bytes
    fn deserialize<S: SerializableStruct>(&self, data: &[u8]) -> Result<S>;

    /// Serialize struct to a writer
    fn serialize_to_writer<S: SerializableStruct, W: Write>(
        &self,
        data: &S,
        writer: W,
    ) -> Result<()>;

    /// Deserialize struct from a reader
    fn deserialize_from_reader<S: SerializableStruct, R: Read>(&self, reader: R) -> Result<S>;
}

/// JSON implementation of StructSerializer
///
/// Uses the low-level JSON serializer for efficient byte-oriented operations.
///
/// # Examples
///
/// ```rust
/// use serialization::structs::{JsonStructSerializer, StructSerializer, SerializableStruct};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct TestData {
///     pub field_a: String,
///     pub field_b: i32,
/// }
///
/// impl SerializableStruct for TestData {}
///
/// let serializer = JsonStructSerializer::new();
/// let data = TestData {
///     field_a: "value".to_string(),
///     field_b: 42,
/// };
///
/// let bytes = serializer.serialize(&data).unwrap();
/// let restored: TestData = serializer.deserialize(&bytes).unwrap();
/// assert_eq!(data, restored);
/// ```
pub struct JsonStructSerializer;

impl JsonStructSerializer {
    /// Create a new JSON struct serializer
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonStructSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl StructSerializer for JsonStructSerializer {
    fn serialize<S: SerializableStruct>(&self, data: &S) -> Result<Vec<u8>> {
        crate::serializer::Serializer::to_json_pretty(data)
    }

    fn deserialize<S: SerializableStruct>(&self, data: &[u8]) -> Result<S> {
        crate::serializer::Serializer::from_json(data)
    }

    fn serialize_to_writer<S: SerializableStruct, W: Write>(
        &self,
        data: &S,
        writer: W,
    ) -> Result<()> {
        crate::serializer::Serializer::to_json_pretty_writer(data, writer)
    }

    fn deserialize_from_reader<S: SerializableStruct, R: Read>(&self, reader: R) -> Result<S> {
        crate::serializer::Serializer::from_json_reader(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestConfigA {
        pub field_a: String,
        pub field_b: String,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestConfigB {
        pub value_x: String,
        pub value_y: i32,
    }

    impl SerializableStruct for TestConfigA {}
    impl SerializableStruct for TestConfigB {}

    fn create_test_config_a() -> TestConfigA {
        TestConfigA {
            field_a: "sensor_data".to_string(),
            field_b: "block_config".to_string(),
        }
    }

    fn create_test_config_b() -> TestConfigB {
        TestConfigB {
            value_x: "processed_data".to_string(),
            value_y: 42,
        }
    }

    #[test]
    fn test_json_struct_serializer_roundtrip() {
        let serializer = JsonStructSerializer::new();
        let config = create_test_config_a();

        let bytes = serializer.serialize(&config).unwrap();
        let restored: TestConfigA = serializer.deserialize(&bytes).unwrap();

        assert_eq!(config, restored);
    }

    #[test]
    fn test_json_struct_serializer_different_types() {
        let serializer = JsonStructSerializer::new();
        let config = create_test_config_b();

        let bytes = serializer.serialize(&config).unwrap();
        let restored: TestConfigB = serializer.deserialize(&bytes).unwrap();

        assert_eq!(config, restored);
    }

    #[test]
    fn test_json_struct_serializer_writer_reader() {
        let serializer = JsonStructSerializer::new();
        let config = create_test_config_a();
        let mut buffer = Vec::new();

        // Write to buffer
        serializer
            .serialize_to_writer(&config, &mut buffer)
            .unwrap();

        // Read from buffer
        let restored: TestConfigA = serializer.deserialize_from_reader(&buffer[..]).unwrap();

        assert_eq!(config, restored);
    }

    #[test]
    fn test_json_output_format() {
        let serializer = JsonStructSerializer::new();
        let config = TestConfigA {
            field_a: "input_channel".to_string(),
            field_b: "config_channel".to_string(),
        };

        let bytes = serializer.serialize(&config).unwrap();
        let json_str = String::from_utf8(bytes).unwrap();

        // Should be pretty-printed JSON
        assert!(json_str.contains("  ")); // Has indentation
        assert!(json_str.contains("\"field_a\""));
        assert!(json_str.contains("\"input_channel\""));
        assert!(json_str.contains("\"field_b\""));
        assert!(json_str.contains("\"config_channel\""));
    }

    #[test]
    fn test_name_parameter_ignored_in_json() {
        let serializer = JsonStructSerializer::new();
        let config = create_test_config_a();

        // The name parameter doesn't affect JSON serialization
        let bytes1 = serializer.serialize(&config).unwrap();
        let bytes2 = serializer.serialize(&config).unwrap();

        assert_eq!(bytes1, bytes2);

        // Both should deserialize to the same thing regardless of name
        let restored1: TestConfigA = serializer.deserialize(&bytes1).unwrap();
        let restored2: TestConfigA = serializer.deserialize(&bytes2).unwrap();

        assert_eq!(restored1, restored2);
        assert_eq!(restored1, config);
    }
}
