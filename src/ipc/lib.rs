pub mod channel;
pub mod protocol;
pub mod security;

pub use channel::{
    BroadcastChannel, ChannelManager, IpcChannel, IpcMessage, MessagePriority, MessageType,
    ZeroCopyMessage,
};
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
    #[error("Capability verification failed: {0}")]
    CapabilityError(String),
    #[error("Message expired")]
    MessageExpired,
    #[error("Connection hijacked: {0}")]
    ConnectionHijacked(String),
    #[error("Time error: {0}")]
    TimeError(String),
}

pub type Result<T> = std::result::Result<T, IpcError>;
