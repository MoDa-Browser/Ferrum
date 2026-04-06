use super::channel::IpcMessage;
use super::{IpcError, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

pub struct IpcSecurity {
    enable_encryption: bool,
    enable_authentication: bool,
    encryption_key: Option<LessSafeKey>,
    rng: SystemRandom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedPayload {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

impl IpcSecurity {
    pub fn new() -> Self {
        Self {
            enable_encryption: false,
            enable_authentication: false,
            encryption_key: None,
            rng: SystemRandom::new(),
        }
    }

    pub fn with_encryption(mut self, enable: bool) -> Self {
        self.enable_encryption = enable;
        self
    }

    pub fn with_key(mut self, key: &[u8; 32]) -> Result<Self> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| IpcError::SecurityError(format!("Invalid encryption key: {}", e)))?;
        self.encryption_key = Some(LessSafeKey::new(unbound_key));
        Ok(self)
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
            let key = self
                .encryption_key
                .as_ref()
                .ok_or_else(|| IpcError::SecurityError("Encryption key not set".to_string()))?;

            let mut nonce_bytes = [0u8; 12];
            self.rng
                .fill(&mut nonce_bytes)
                .map_err(|e| IpcError::SecurityError(format!("Failed to generate nonce: {}", e)))?;

            let nonce = Nonce::assume_unique_for_key(nonce_bytes);
            let mut ciphertext = message.payload.clone();

            key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
                .map_err(|e| IpcError::SecurityError(format!("Encryption failed: {}", e)))?;

            let encrypted = EncryptedPayload {
                nonce: nonce_bytes.to_vec(),
                ciphertext,
            };

            message.payload = serde_json::to_vec(&encrypted).map_err(|e| {
                IpcError::SecurityError(format!("Failed to serialize encrypted data: {}", e))
            })?;
        }
        Ok(())
    }

    pub fn decrypt_message(&self, message: &mut IpcMessage) -> Result<()> {
        if self.enable_encryption {
            let key = self
                .encryption_key
                .as_ref()
                .ok_or_else(|| IpcError::SecurityError("Encryption key not set".to_string()))?;

            let encrypted: EncryptedPayload =
                serde_json::from_slice(&message.payload).map_err(|e| {
                    IpcError::SecurityError(format!("Failed to deserialize encrypted data: {}", e))
                })?;

            let nonce = Nonce::assume_unique_for_key(
                encrypted
                    .nonce
                    .as_slice()
                    .try_into()
                    .map_err(|_| IpcError::SecurityError("Invalid nonce length".to_string()))?,
            );

            let mut plaintext = encrypted.ciphertext.clone();

            key.open_in_place(nonce, Aad::empty(), &mut plaintext)
                .map_err(|e| IpcError::SecurityError(format!("Decryption failed: {}", e)))?;

            plaintext.truncate(plaintext.len() - 16);

            message.payload = plaintext;
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
        let key = [0u8; 32];
        let security = IpcSecurity::new()
            .with_encryption(true)
            .with_key(&key)
            .unwrap();

        let mut message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        let original_payload = message.payload.clone();

        security.encrypt_message(&mut message).unwrap();
        assert_ne!(message.payload, original_payload);

        security.decrypt_message(&mut message).unwrap();
        assert_eq!(message.payload, original_payload);
    }
}
