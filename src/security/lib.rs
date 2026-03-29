pub mod capabilities;
pub mod permissions;
pub mod validation;

pub use capabilities::{Capability, CapabilityToken};
pub use permissions::{PermissionManager, PermissionPolicy};
pub use validation::Validator;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid capability token")]
    InvalidToken,
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

pub type Result<T> = std::result::Result<T, SecurityError>;
