use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Capability {
    NetworkAccess,
    FileSystemRead,
    FileSystemWrite,
    ProcessSpawn,
    SystemCall(String),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    id: String,
    capabilities: HashSet<Capability>,
    expiry: Option<u64>,
}

impl CapabilityToken {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capabilities: HashSet::new(),
            expiry: None,
        }
    }

    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.insert(capability);
        self
    }

    pub fn with_capabilities(mut self, capabilities: impl IntoIterator<Item = Capability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    pub fn with_expiry(mut self, timestamp: u64) -> Self {
        self.expiry = Some(timestamp);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn capabilities(&self) -> &HashSet<Capability> {
        &self.capabilities
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry {
            let current = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            current > expiry
        } else {
            false
        }
    }

    pub fn verify(&self, required: &HashSet<Capability>) -> bool {
        if self.is_expired() {
            return false;
        }
        required.is_subset(&self.capabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = CapabilityToken::new("test-token")
            .with_capability(Capability::NetworkAccess)
            .with_capability(Capability::FileSystemRead);

        assert_eq!(token.id(), "test-token");
        assert!(token.capabilities().contains(&Capability::NetworkAccess));
        assert!(token.capabilities().contains(&Capability::FileSystemRead));
    }

    #[test]
    fn test_token_verification() {
        let token = CapabilityToken::new("test-token").with_capability(Capability::NetworkAccess);

        let required = vec![Capability::NetworkAccess].into_iter().collect();
        assert!(token.verify(&required));

        let required = vec![Capability::NetworkAccess, Capability::FileSystemWrite]
            .into_iter()
            .collect();
        assert!(!token.verify(&required));
    }
}
