//! # Block Flow
//!
//! A framework for building data processing pipelines using composable blocks.
//!
//! This library provides:
//! - Core traits and types for building blocks (`block_traits`)
//! - Concrete block implementations (`blocks`)
//! - A registry system for sharing data between blocks (`registry`)
//! - Procedural macros for defining blocks (`block_macros`)

// Re-export core traits and types
pub use block_traits::*;

// Re-export concrete block implementations
pub use blocks::*;

// Re-export registry functionality
pub use registry::*;

// Re-export macros
pub use block_macros::*;
