pub mod manager;
pub mod namespace;

pub use manager::Sandbox;
pub use namespace::NamespaceConfig;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Failed to create sandbox: {0}")]
    CreationFailed(String),
    #[error("Namespace isolation failed: {0}")]
    NamespaceError(String),
    #[error("Process spawn failed: {0}")]
    ProcessError(String),
}

pub type Result<T> = std::result::Result<T, SandboxError>;
