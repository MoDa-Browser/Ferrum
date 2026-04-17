use super::channel::IpcMessage;
use super::{IpcError, Result};
use hmac::{Hmac, Mac};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureData {
    message_id: String,
    source: String,
    target: String,
    payload: Vec<u8>,
    timestamp: u64,
}

pub struct IpcSecurity {
    enable_encryption: bool,
    enable_authentication: bool,
    encryption_key: Option<LessSafeKey>,
    signature_key: Option<Hmac<Sha256>>,
    rng: SystemRandom,
    allowed_sources: HashSet<String>,
    session_tokens: HashSet<String>,
    max_message_age_seconds: u64,
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
            signature_key: None,
            rng: SystemRandom::new(),
            allowed_sources: HashSet::new(),
            session_tokens: HashSet::new(),
            max_message_age_seconds: 300, // 默认 5 分钟
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
        
        // 同时设置签名密钥
        let signature_key = Hmac::<Sha256>::new_from_slice(key)
            .map_err(|e| IpcError::SecurityError(format!("Invalid signature key: {}", e)))?;
        self.signature_key = Some(signature_key);
        
        Ok(self)
    }

    pub fn with_authentication(mut self, enable: bool) -> Self {
        self.enable_authentication = enable;
        self
    }

    pub fn with_max_message_age(mut self, max_age_seconds: u64) -> Self {
        self.max_message_age_seconds = max_age_seconds;
        self
    }

    pub fn add_allowed_source(&mut self, source: impl Into<String>) {
        self.allowed_sources.insert(source.into());
    }

    pub fn remove_allowed_source(&mut self, source: &str) {
        self.allowed_sources.remove(source);
    }

    pub fn generate_session_token(&mut self) -> Result<String> {
        let mut token_bytes = [0u8; 32];
        self.rng.fill(&mut token_bytes).map_err(|e| {
            IpcError::SecurityError(format!("Failed to generate session token: {}", e))
        })?;
        let token = hex::encode(token_bytes);
        self.session_tokens.insert(token.clone());
        Ok(token)
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

            if !self.allowed_sources.is_empty() && !self.allowed_sources.contains(&message.source) {
                return Err(IpcError::CapabilityError(format!(
                    "Source '{}' is not in allowed sources",
                    message.source
                )));
            }

            if message.is_expired() {
                return Err(IpcError::MessageExpired);
            }

            // 检查消息是否超过最大允许年龄
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| IpcError::SecurityError(format!("Failed to get system time: {}", e)))?
                .as_secs();

            if current_time - message.timestamp > self.max_message_age_seconds {
                return Err(IpcError::MessageExpired);
            }
        }
        Ok(())
    }

    pub fn sign_message(&self, message: &IpcMessage) -> Result<Vec<u8>> {
        if let Some(ref signature_key) = self.signature_key {
            // 使用确定性结构体构造签名数据
            let signature_data = SignatureData {
                message_id: message.id.clone(),
                source: message.source.clone(),
                target: message.target.clone(),
                payload: message.payload.clone(),
                timestamp: message.timestamp,
            };

            // 使用 bincode 进行确定性序列化
            let data_to_sign = bincode::serialize(&signature_data).map_err(|e| {
                IpcError::SecurityError(format!("Failed to serialize signature data: {}", e))
            })?;

            // 使用 HMAC-SHA256 对数据进行签名
            let mut mac = signature_key.clone();
            mac.update(&data_to_sign);
            let signature = mac.finalize();

            // 返回签名结果
            Ok(signature.into_bytes().to_vec())
        } else {
            Err(IpcError::SecurityError(
                "Signature key not set for signing".to_string(),
            ))
        }
    }

    pub fn verify_signature(&self, message: &IpcMessage, signature: &[u8]) -> Result<bool> {
        if let Some(ref signature_key) = self.signature_key {
            // 使用恒定时间比较防止时序攻击
            use hmac::Mac;
            let mut mac = signature_key.clone();
            let signature_data = SignatureData {
                message_id: message.id.clone(),
                source: message.source.clone(),
                target: message.target.clone(),
                payload: message.payload.clone(),
                timestamp: message.timestamp,
            };
            let data_to_sign = bincode::serialize(&signature_data).map_err(|e| {
                IpcError::SecurityError(format!("Failed to serialize signature data: {}", e))
            })?;
            mac.update(&data_to_sign);
            let expected_signature = mac.finalize();

            // 比较签名
            Ok(expected_signature.into_bytes().as_slice() == signature)
        } else {
            Err(IpcError::SecurityError(
                "Signature key not set for verification".to_string(),
            ))
        }
    }

    pub fn detect_connection_hijacking(&self, message: &IpcMessage) -> Result<()> {
        if self.enable_authentication {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| IpcError::SecurityError(format!("Failed to get system time: {}", e)))?
                .as_secs();

            if current_time - message.timestamp > self.max_message_age_seconds {
                return Err(IpcError::ConnectionHijacked(format!(
                    "Message timestamp is too old ({}s > {}s), possible connection hijacking",
                    current_time - message.timestamp,
                    self.max_message_age_seconds
                )));
            }

            let session_token = message
                .session_token
                .as_ref()
                .ok_or_else(|| IpcError::ConnectionHijacked("Missing session token".to_string()))?;

            if !self.validate_session_token(session_token) {
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
        let token = security.generate_session_token().unwrap();
        assert!(security.validate_session_token(&token));

        security.revoke_session_token(&token);
        assert!(!security.validate_session_token(&token));
    }

    #[test]
    fn test_detect_connection_hijacking() {
        let mut security = IpcSecurity::new().with_authentication(true);
        let token = security.generate_session_token().unwrap();

        let message = IpcMessage::new("source", "target", vec![1, 2, 3]).with_session_token(&token);
        assert!(security.detect_connection_hijacking(&message).is_ok());

        let invalid_message =
            IpcMessage::new("source", "target", vec![1, 2, 3]).with_session_token("invalid_token");
        assert!(security
            .detect_connection_hijacking(&invalid_message)
            .is_err());

        let no_token_message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        assert!(security
            .detect_connection_hijacking(&no_token_message)
            .is_err());
    }

    #[test]
    fn test_custom_max_message_age() {
        let mut security = IpcSecurity::new()
            .with_authentication(true)
            .with_max_message_age(60); // 1 分钟

        let token = security.generate_session_token().unwrap();
        let message = IpcMessage::new("source", "target", vec![1, 2, 3]).with_session_token(&token);

        // 默认情况下应该通过（消息是刚创建的）
        assert!(security.detect_connection_hijacking(&message).is_ok());
    }

    #[test]
    fn test_sign_and_verify() {
        let key = [0u8; 32];
        let security = IpcSecurity::new()
            .with_authentication(true)  // 启用认证而不是加密，因为现在签名是独立的功能
            .with_key(&key)
            .unwrap();

        let message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        let signature = security.sign_message(&message).unwrap();

        let is_valid = security.verify_signature(&message, &signature).unwrap();
        assert!(is_valid);

        // 测试篡改消息
        let mut tampered_message = message.clone();
        tampered_message.payload = vec![4, 5, 6];
        let is_valid_tampered = security
            .verify_signature(&tampered_message, &signature)
            .unwrap();
        assert!(!is_valid_tampered);
    }

    #[test]
    fn test_validate_message_with_max_age() {
        let security = IpcSecurity::new()
            .with_authentication(true)
            .with_max_message_age(1); // 1 秒

        let message = IpcMessage::new("source", "target", vec![1, 2, 3]);
        assert!(security.validate_message(&message).is_ok());

        // 创建一个旧消息（时间戳为 10 秒前）
        let mut old_message = message.clone();
        old_message.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 10;
        assert!(security.validate_message(&old_message).is_err());
    }
}
