/// Errors that can occur during registry operations
#[derive(Debug, PartialEq)]
pub enum RegistryError {
    KeyNotFound(String),
    CycleDetected(String),
    DuplicateOutputKey(String),
    MissingProducer(String),
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
            RegistryError::CycleDetected(details) => {
                write!(f, "Cycle detected in registry: {}", details)
            }
            RegistryError::DuplicateOutputKey(key) => {
                write!(f, "Duplicate output key '{key}' in registry")
            }
            RegistryError::MissingProducer(err) => {
                write!(f, "Missing producer error: {err}")
            }
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
