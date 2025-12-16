//! Low-level serialization implementation
//!
//! This module contains the actual serialization logic and implementation details.
//! Users should interact with the higher-level interface in the main lib module.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

use crate::error::Result;

/// Low-level serialization implementation
pub struct Serializer;

impl Serializer {
    /// Serialize data to JSON bytes
    pub fn to_json<T>(data: &T) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        Ok(serde_json::to_vec(data)?)
    }

    /// Serialize data to pretty-printed JSON bytes
    pub fn to_json_pretty<T>(data: &T) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        Ok(serde_json::to_vec_pretty(data)?)
    }

    /// Deserialize data from JSON bytes
    pub fn from_json<T>(data: &[u8]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        Ok(serde_json::from_slice(data)?)
    }

    /// Serialize data to a JSON writer
    pub fn to_json_writer<T, W>(data: &T, writer: W) -> Result<()>
    where
        T: Serialize,
        W: Write,
    {
        serde_json::to_writer(writer, data)?;
        Ok(())
    }

    /// Serialize data to a pretty-printed JSON writer
    pub fn to_json_pretty_writer<T, W>(data: &T, writer: W) -> Result<()>
    where
        T: Serialize,
        W: Write,
    {
        serde_json::to_writer_pretty(writer, data)?;
        Ok(())
    }

    /// Deserialize data from a JSON reader
    pub fn from_json_reader<T, R>(reader: R) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Read,
    {
        Ok(serde_json::from_reader(reader)?)
    }

    /// Save data to a JSON file
    pub fn save_json_to_file<T>(data: &T, path: &std::path::Path) -> Result<()>
    where
        T: Serialize,
    {
        let file = std::fs::File::create(path)?;
        Self::to_json_pretty_writer(data, file)
    }

    /// Load data from a JSON file
    pub fn load_json_from_file<T>(path: &std::path::Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let file = std::fs::File::open(path)?;
        Self::from_json_reader(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        value: i32,
        optional: Option<String>,
        list: Vec<i32>,
    }

    fn create_test_data() -> TestData {
        TestData {
            name: "test".to_string(),
            value: 42,
            optional: Some("hello".to_string()),
            list: vec![1, 2, 3, 4, 5],
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let data = create_test_data();
        let bytes = Serializer::to_json(&data).unwrap();
        let restored: TestData = Serializer::from_json(&bytes).unwrap();
        assert_eq!(data, restored);
    }

    #[test]
    fn test_json_pretty_roundtrip() {
        let data = create_test_data();
        let bytes = Serializer::to_json_pretty(&data).unwrap();
        let restored: TestData = Serializer::from_json(&bytes).unwrap();
        assert_eq!(data, restored);

        // Verify it's actually pretty printed
        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains("  ")); // Should have indentation
    }

    #[test]
    fn test_json_file_roundtrip() {
        let data = create_test_data();
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.json");

        // Save and load
        Serializer::save_json_to_file(&data, &file_path).unwrap();
        let restored: TestData = Serializer::load_json_from_file(&file_path).unwrap();

        assert_eq!(data, restored);
    }

    #[test]
    fn test_json_writer_reader() {
        let data = create_test_data();
        let mut buffer = Vec::new();

        // Write to buffer
        Serializer::to_json_writer(&data, &mut buffer).unwrap();

        // Read from buffer
        let restored: TestData = Serializer::from_json_reader(&buffer[..]).unwrap();

        assert_eq!(data, restored);
    }
}
