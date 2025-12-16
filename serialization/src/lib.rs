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
pub mod keys;
pub mod serializer;

pub use error::{Result, SerializationError};
pub use keys::{JsonStructSerializer, SerializableStruct, StructSerializer};
