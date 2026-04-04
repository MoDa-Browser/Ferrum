pub mod channel;
pub mod protocol;
pub mod security;

pub use channel::{IpcChannel, IpcMessage};
pub use protocol::IpcProtocol;
pub use security::IpcSecurity;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Security error: {0}")]
    SecurityError(String),
}

pub type Result<T> = std::result::Result<T, IpcError>;
