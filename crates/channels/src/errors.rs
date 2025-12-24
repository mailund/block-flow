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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_key_not_found() {
        let err = RegistryError::KeyNotFound("missing".to_string());
        assert_eq!(err.to_string(), "Key 'missing' not found in registry");
    }

    #[test]
    fn display_cycle_detected() {
        let err = RegistryError::CycleDetected("A -> B -> A".to_string());
        assert_eq!(err.to_string(), "Cycle detected in registry: A -> B -> A");
    }

    #[test]
    fn display_duplicate_output_key() {
        let err = RegistryError::DuplicateOutputKey("out".to_string());
        assert_eq!(err.to_string(), "Duplicate output key 'out' in registry");
    }

    #[test]
    fn display_missing_producer() {
        let err = RegistryError::MissingProducer("no producer for x".to_string());
        assert_eq!(err.to_string(), "Missing producer error: no producer for x");
    }

    #[test]
    fn display_type_mismatch() {
        let err = RegistryError::TypeMismatch {
            key: "k".to_string(),
            expected: "i32",
            found: "unknown",
        };
        assert_eq!(
            err.to_string(),
            "Type mismatch for key 'k': expected i32, found unknown"
        );
    }

    #[test]
    fn debug_and_partial_eq_are_sane() {
        let a = RegistryError::KeyNotFound("x".to_string());
        let b = RegistryError::KeyNotFound("x".to_string());
        let c = RegistryError::KeyNotFound("y".to_string());
        assert_eq!(a, b);
        assert_ne!(a, c);

        let dbg = format!("{:?}", RegistryError::CycleDetected("cycle".to_string()));
        assert!(dbg.contains("CycleDetected"));
    }
}
