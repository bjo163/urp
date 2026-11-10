//! Shared error taxonomy. Transport adapters decide how errors are exposed.

/// A domain-independent error that can cross application boundaries.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KernelError {
    /// A supplied value failed validation.
    #[error("validation failed for {field}")]
    Validation {
        /// Name of the invalid field.
        field: &'static str,
        /// Safe, user-facing validation detail.
        message: String,
    },
    /// The requested resource does not exist or is not visible.
    #[error("resource was not found")]
    NotFound,
    /// The operation conflicts with the current resource state.
    #[error("operation conflicts with the current resource state")]
    Conflict,
    /// The current actor is not permitted to perform the operation.
    #[error("operation is not permitted")]
    Forbidden,
    /// The request requires an authenticated actor.
    #[error("authentication is required")]
    Unauthenticated,
    /// A required dependency is temporarily unavailable.
    #[error("a required dependency is unavailable")]
    Unavailable,
}

impl KernelError {
    /// Returns a stable machine-readable error code.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "validation_error",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Forbidden => "forbidden",
            Self::Unauthenticated => "unauthenticated",
            Self::Unavailable => "unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KernelError;

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(KernelError::NotFound.code(), "not_found");
        assert_eq!(KernelError::Forbidden.code(), "forbidden");
    }
}
