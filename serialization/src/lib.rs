//! # Serialization
//!
//! A serialization library for the block-flow framework.
//!
//! Currently supports:
//! - JSON (human-readable, widely supported)
//!
//! Future support planned for:
//! - Protocol Buffers (efficient binary format)

pub mod error;
pub mod serializer;
pub mod structs;

pub use error::{Result, SerializationError};
pub use structs::read_struct_from_json;
pub use structs::{JsonStructSerializer, SerializableStruct, StructSerializer};
