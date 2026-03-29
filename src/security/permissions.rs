use super::capabilities::Capability;
use super::{Result, SecurityError};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    pub allowed_capabilities: Vec<Capability>,
    pub denied_capabilities: Vec<Capability>,
}

pub struct PermissionManager {
    policies: Arc<RwLock<HashMap<String, PermissionPolicy>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_policy(&self, resource_id: impl Into<String>, policy: PermissionPolicy) -> Result<()> {
        let mut policies = self.policies.write()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;
        policies.insert(resource_id.into(), policy);
        Ok(())
    }

    pub fn check_permission(&self, resource_id: &str, capability: &Capability) -> Result<bool> {
        let policies = self.policies.read()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;

        if let Some(policy) = policies.get(resource_id) {
            if policy.denied_capabilities.contains(capability) {
                return Ok(false);
            }
            if policy.allowed_capabilities.contains(capability) {
                return Ok(true);
            }
        }

        Err(SecurityError::PermissionDenied(format!(
            "No policy defined for resource {} with capability {:?}",
            resource_id, capability
        )))
    }

    pub fn remove_policy(&self, resource_id: &str) {
        let mut policies = self.policies.write().unwrap();
        policies.remove(resource_id);
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check() {
        let manager = PermissionManager::new();

        let policy = PermissionPolicy {
            allowed_capabilities: vec![Capability::NetworkAccess],
            denied_capabilities: vec![Capability::FileSystemWrite],
        };

        manager.add_policy("test-resource", policy);

        assert!(manager
            .check_permission("test-resource", &Capability::NetworkAccess)
            .unwrap());
        assert!(!manager
            .check_permission("test-resource", &Capability::FileSystemWrite)
            .unwrap());
    }
}
