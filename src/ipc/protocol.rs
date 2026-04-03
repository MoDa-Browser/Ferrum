use super::channel::IpcMessage;
use super::{IpcError, Result};

#[derive(Debug, Clone, Default)]
pub enum IpcProtocol {
    #[default]
    Json,
    Binary,
    Custom(String),
}

impl IpcProtocol {
    pub fn serialize(&self, message: &IpcMessage) -> Result<Vec<u8>> {
        match self {
            IpcProtocol::Json => serde_json::to_vec(message).map_err(|e| {
                IpcError::SerializationError(format!("JSON serialization failed: {}", e))
            }),
            IpcProtocol::Binary => bincode::serialize(message).map_err(|e| {
                IpcError::SerializationError(format!("Binary serialization failed: {}", e))
            }),
            IpcProtocol::Custom(name) => Err(IpcError::SerializationError(format!(
                "Custom protocol '{}' not implemented",
                name
            ))),
        }
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<IpcMessage> {
        match self {
            IpcProtocol::Json => serde_json::from_slice(data).map_err(|e| {
                IpcError::SerializationError(format!("JSON deserialization failed: {}", e))
            }),
            IpcProtocol::Binary => bincode::deserialize(data).map_err(|e| {
                IpcError::SerializationError(format!("Binary deserialization failed: {}", e))
            }),
            IpcProtocol::Custom(name) => Err(IpcError::SerializationError(format!(
                "Custom protocol '{}' not implemented",
                name
            ))),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_protocol() {
        let protocol = IpcProtocol::Json;
        let message = IpcMessage::new("source", "target", vec![1, 2, 3]);

        let serialized = protocol.serialize(&message).unwrap();
        let deserialized = protocol.deserialize(&serialized).unwrap();

        assert_eq!(deserialized.source, message.source);
        assert_eq!(deserialized.target, message.target);
    }
}
