use super::channel::IpcMessage;
use super::{IpcError, Result};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct IpcSecurity {
    enable_encryption: bool,
    enable_authentication: bool,
    encryption_key: Option<LessSafeKey>,
    rng: SystemRandom,
    allowed_sources: HashSet<String>,
    message_signatures: HashSet<String>,
    session_tokens: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedPayload {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageSignature {
    message_id: String,
    signature: Vec<u8>,
    timestamp: u64,
}

impl IpcSecurity {
    pub fn new() -> Self {
        Self {
            enable_encryption: false,
            enable_authentication: false,
            encryption_key: None,
            rng: SystemRandom::new(),
            allowed_sources: HashSet::new(),
            message_signatures: HashSet::new(),
            session_tokens: HashSet::new(),
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

    pub fn add_allowed_source(&mut self, source: impl Into<String>) {
        self.allowed_sources.insert(source.into());
    }

    pub fn remove_allowed_source(&mut self, source: &str) {
        self.allowed_sources.remove(source);
    }

    pub fn generate_session_token(&mut self) -> String {
        let mut token_bytes = [0u8; 32];
        self.rng.fill(&mut token_bytes).unwrap();
        let token = hex::encode(token_bytes);
        self.session_tokens.insert(token.clone());
        token
    }

    pub fn validate_session_token(&self, token: &str) -> bool {
        self.session_tokens.contains(token)
    }

    pub fn revoke_session_token(&mut self, token: &str) {
        self.session_tokens.remove(token);
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

            if !self.allowed_sources.is_empty()
                && !self.allowed_sources.contains(&message.source)
            {
                return Err(IpcError::CapabilityError(format!(
                    "Source '{}' is not in allowed sources",
                    message.source
                )));
            }

            if message.is_expired() {
                return Err(IpcError::MessageExpired);
            }
        }
        Ok(())
    }

    pub fn sign_message(&self, message: &IpcMessage) -> Result<Vec<u8>> {
        let mut signature_data = message.id.clone().into_bytes();
        signature_data.extend_from_slice(&message.source.clone().into_bytes());
        signature_data.extend_from_slice(&message.target.clone().into_bytes());
        signature_data.extend_from_slice(&message.payload);

        if let Some(key) = &self.encryption_key {
            let mut nonce_bytes = [0u8; 12];
            self.rng.fill(&mut nonce_bytes).map_err(|e| {
                IpcError::SecurityError(format!("Failed to generate nonce: {}", e))
            })?;

            let nonce = Nonce::assume_unique_for_key(nonce_bytes);
            let mut signature = signature_data.clone();

            key.seal_in_place_append_tag(nonce, Aad::empty(), &mut signature)
                .map_err(|e| IpcError::SecurityError(format!("Signing failed: {}", e)))?;

            Ok(signature)
        } else {
            Err(IpcError::SecurityError(
                "Encryption key not set for signing".to_string(),
            ))
        }
    }

    pub fn verify_signature(&self, message: &IpcMessage, signature: &[u8]) -> Result<bool> {
        if let Some(key) = &self.encryption_key {
            let mut signature_data = message.id.clone().into_bytes();
            signature_data.extend_from_slice(&message.source.clone().into_bytes());
            signature_data.extend_from_slice(&message.target.clone().into_bytes());
            signature_data.extend_from_slice(&message.payload);

            let mut nonce_bytes = [0u8; 12];
            self.rng.fill(&mut nonce_bytes).map_err(|e| {
                IpcError::SecurityError(format!("Failed to generate nonce: {}", e))
            })?;

            let nonce = Nonce::assume_unique_for_key(nonce_bytes);
            let mut signature_copy = signature.to_vec();

            match key.open_in_place(nonce, Aad::empty(), &mut signature_copy) {
                Ok(_) => {
                    let expected_data = &signature_copy[..signature_copy.len() - 16];
                    Ok(expected_data == signature_data.as_slice())
                }
                Err(_) => Ok(false),
            }
        } else {
            Err(IpcError::SecurityError(
                "Encryption key not set for verification".to_string(),
            ))
        }
    }

    pub fn detect_connection_hijacking(&self, message: &IpcMessage) -> Result<()> {
        if self.enable_authentication {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if current_time - message.timestamp > 300 {
                return Err(IpcError::ConnectionHijacked(
                    "Message timestamp is too old, possible connection hijacking".to_string(),
                ));
            }

            if !self.validate_session_token(&message.id) {
                return Err(IpcError::ConnectionHijacked(
                    "Invalid session token".to_string(),
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

        let message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        assert!(security.validate_message(&message).is_ok());

        let mut empty_source_message = message.clone();
        empty_source_message.source = String::new();
        assert!(security.validate_message(&empty_source_message).is_err());
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

    #[test]
    fn test_source_validation() {
        let mut security = IpcSecurity::new().with_authentication(true);
        security.add_allowed_source("allowed_source");

        let message = IpcMessage::new("allowed_source", "target", vec![1, 2, 3]);
        assert!(security.validate_message(&message).is_ok());

        let message = IpcMessage::new("disallowed_source", "target", vec![1, 2, 3]);
        assert!(security.validate_message(&message).is_err());
    }

    #[test]
    fn test_session_token() {
        let mut security = IpcSecurity::new();
        let token = security.generate_session_token();
        assert!(security.validate_session_token(&token));

        security.revoke_session_token(&token);
        assert!(!security.validate_session_token(&token));
    }
}
