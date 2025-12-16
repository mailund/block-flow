use std::fmt;

/// Serialization error types
#[derive(Debug)]
pub enum SerializationError {
    /// JSON serialization/deserialization error
    Json(serde_json::Error),
    /// IO error
    Io(std::io::Error),
    /// Custom error message
    Custom(String),
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::Json(e) => write!(f, "JSON error: {}", e),
            SerializationError::Io(e) => write!(f, "IO error: {}", e),
            SerializationError::Custom(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for SerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SerializationError::Json(e) => Some(e),
            SerializationError::Io(e) => Some(e),
            SerializationError::Custom(_) => None,
        }
    }
}

// Implement From conversions for easy error handling
impl From<serde_json::Error> for SerializationError {
    fn from(error: serde_json::Error) -> Self {
        SerializationError::Json(error)
    }
}

impl From<std::io::Error> for SerializationError {
    fn from(error: std::io::Error) -> Self {
        SerializationError::Io(error)
    }
}

/// Result type alias for serialization operations
pub type Result<T> = std::result::Result<T, SerializationError>;
