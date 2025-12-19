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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;
    use std::io;

    #[test]
    fn display_json_error_includes_prefix() {
        // Create a real serde_json::Error by parsing invalid JSON
        let err = serde_json::from_str::<serde_json::Value>("{ not valid json }").unwrap_err();
        let se = SerializationError::Json(err);

        let s = se.to_string();
        assert!(s.starts_with("JSON error: "));
    }

    #[test]
    fn display_io_error_includes_prefix() {
        let io_err = io::Error::new(io::ErrorKind::Other, "boom");
        let se = SerializationError::Io(io_err);

        let s = se.to_string();
        assert_eq!(s, "IO error: boom");
    }

    #[test]
    fn display_custom_error_includes_prefix_and_message() {
        let se = SerializationError::Custom("hello".to_string());
        let s = se.to_string();
        assert_eq!(s, "Serialization error: hello");
    }

    #[test]
    fn source_for_json_error_is_some() {
        let err = serde_json::from_str::<serde_json::Value>("{ not valid json }").unwrap_err();
        let se = SerializationError::Json(err);

        let src = se.source();
        assert!(src.is_some());
        assert!(src.unwrap().to_string().len() > 0);
    }

    #[test]
    fn source_for_io_error_is_some() {
        let io_err = io::Error::new(io::ErrorKind::Other, "boom");
        let se = SerializationError::Io(io_err);

        let src = se.source();
        assert!(src.is_some());
        assert_eq!(src.unwrap().to_string(), "boom");
    }

    #[test]
    fn source_for_custom_error_is_none() {
        let se = SerializationError::Custom("hello".to_string());
        assert!(se.source().is_none());
    }

    #[test]
    fn from_serde_json_error_converts_to_json_variant() {
        let err = serde_json::from_str::<serde_json::Value>("{ not valid json }").unwrap_err();
        let se: SerializationError = err.into();

        match se {
            SerializationError::Json(_) => {}
            _ => panic!("expected SerializationError::Json"),
        }
    }

    #[test]
    fn from_io_error_converts_to_io_variant() {
        let err = io::Error::new(io::ErrorKind::Other, "boom");
        let se: SerializationError = err.into();

        match se {
            SerializationError::Io(e) => assert_eq!(e.to_string(), "boom"),
            _ => panic!("expected SerializationError::Io"),
        }
    }

    #[test]
    fn result_type_alias_works() {
        fn ok() -> Result<u32> {
            Ok(7)
        }
        fn fail() -> Result<u32> {
            Err(SerializationError::Custom("nope".to_string()))
        }

        assert_eq!(ok().unwrap(), 7);
        assert_eq!(fail().unwrap_err().to_string(), "Serialization error: nope");
    }
}
