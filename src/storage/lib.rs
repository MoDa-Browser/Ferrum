use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("IO error: {0}")]
    IoError(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub struct SecureStorage {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl SecureStorage {
    pub fn new(key: &[u8; 32]) -> Self {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key).unwrap();
        Self {
            key: LessSafeKey::new(unbound_key),
            rng: SystemRandom::new(),
        }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedData> {
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes).map_err(|e| {
            StorageError::EncryptionFailed(format!("Failed to generate nonce: {}", e))
        })?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut ciphertext = plaintext.to_vec();

        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|e| StorageError::EncryptionFailed(format!("Encryption failed: {}", e)))?;

        Ok(EncryptedData {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }

    pub fn decrypt(&self, data: &EncryptedData) -> Result<Vec<u8>> {
        let nonce = Nonce::assume_unique_for_key(
            data
                .nonce
                .as_slice()
                .try_into()
                .map_err(|_| StorageError::DecryptionFailed("Invalid nonce length".to_string()))?,
        );

        let mut plaintext = data.ciphertext.clone();

        self.key
            .open_in_place(nonce, Aad::empty(), &mut plaintext)
            .map_err(|e| StorageError::DecryptionFailed(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = [0u8; 32];
        let storage = SecureStorage::new(&key);

        let plaintext = b"Hello, World!";
        let encrypted = storage.encrypt(plaintext).unwrap();
        let decrypted = storage.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
