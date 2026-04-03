use super::channel::IpcMessage;
use super::{IpcError, Result};

pub struct IpcSecurity {
    enable_encryption: bool,
    enable_authentication: bool,
}

impl IpcSecurity {
    pub fn new() -> Self {
        Self {
            enable_encryption: false,
            enable_authentication: false,
        }
    }

    pub fn with_encryption(mut self, enable: bool) -> Self {
        self.enable_encryption = enable;
        self
    }

    pub fn with_authentication(mut self, enable: bool) -> Self {
        self.enable_authentication = enable;
        self
    }

    pub fn validate_message(&self, message: &IpcMessage) -> Result<()> {
        if self.enable_authentication {
            if message.source.is_empty() {
                return Err(IpcError::SecurityError(
                    "Message source cannot be empty".to_string(),
                ));
            }
            if message.target.is_empty() {
                return Err(IpcError::SecurityError(
                    "Message target cannot be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn encrypt_message(&self, message: &mut IpcMessage) -> Result<()> {
        if self.enable_encryption {
            for byte in &mut message.payload {
                *byte ^= 0xFF;
            }
        }
        Ok(())
    }

    pub fn decrypt_message(&self, message: &mut IpcMessage) -> Result<()> {
        if self.enable_encryption {
            for byte in &mut message.payload {
                *byte ^= 0xFF;
            }
        }
        Ok(())
    }
}

impl Default for IpcSecurity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_validation() {
        let security = IpcSecurity::new().with_authentication(true);

        let mut message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        assert!(security.validate_message(&message).is_ok());

        message.source = String::new();
        assert!(security.validate_message(&message).is_err());
    }

    #[test]
    fn test_encryption() {
        let security = IpcSecurity::new().with_encryption(true);

        let mut message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        let original_payload = message.payload.clone();

        security.encrypt_message(&mut message).unwrap();
        assert_ne!(message.payload, original_payload);

        security.decrypt_message(&mut message).unwrap();
        assert_eq!(message.payload, original_payload);
    }
}
