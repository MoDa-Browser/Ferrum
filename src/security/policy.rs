use crate::{Capability, Result, SecurityError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 安全策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// 策略ID
    pub id: String,
    /// 策略描述
    pub description: String,
    /// 允许的能力列表
    pub allowed_capabilities: Vec<Capability>,
    /// 拒绝的能力列表
    pub denied_capabilities: Vec<Capability>,
    /// 默认策略（true表示默认拒绝，false表示默认允许）
    pub default_deny: bool,
}

impl SecurityPolicy {
    pub fn new(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            allowed_capabilities: Vec::new(),
            denied_capabilities: Vec::new(),
            default_deny: true,
        }
    }

    pub fn with_allowed_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.allowed_capabilities = capabilities;
        self
    }

    pub fn with_denied_capabilities(mut self, capabilities: Vec<Capability>) -> Self {
        self.denied_capabilities = capabilities;
        self
    }

    pub fn with_default_deny(mut self, default_deny: bool) -> Self {
        self.default_deny = default_deny;
        self
    }

    /// 检查是否允许特定能力
    pub fn allows_capability(&self, capability: &Capability) -> bool {
        // 如果在拒绝列表中，直接拒绝
        if self.denied_capabilities.contains(capability) {
            return false;
        }

        // 如果在允许列表中，直接允许
        if self.allowed_capabilities.contains(capability) {
            return true;
        }

        // 返回默认策略
        !self.default_deny
    }
}

/// 策略管理器
pub struct PolicyManager {
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加安全策略
    pub fn add_policy(&self, policy: SecurityPolicy) -> Result<()> {
        let mut policies = self
            .policies
            .write()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;
        
        policies.insert(policy.id.clone(), policy);
        Ok(())
    }

    /// 获取策略
    pub fn get_policy(&self, policy_id: &str) -> Result<SecurityPolicy> {
        let policies = self
            .policies
            .read()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;
        
        policies
            .get(policy_id)
            .cloned()
            .ok_or_else(|| SecurityError::PermissionDenied(format!("Policy {} not found", policy_id)))
    }

    /// 检查资源是否具有特定能力
    pub fn check_resource_capability(
        &self,
        resource_id: &str,
        capability: &Capability,
    ) -> Result<bool> {
        let policies = self
            .policies
            .read()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;

        // 查找与资源匹配的策略
        if let Some(policy) = policies.get(resource_id) {
            Ok(policy.allows_capability(capability))
        } else {
            // 如果没有找到特定策略，返回错误
            Err(SecurityError::PermissionDenied(format!(
                "No policy defined for resource {}",
                resource_id
            )))
        }
    }

    /// 删除策略
    pub fn remove_policy(&self, policy_id: &str) -> Result<()> {
        let mut policies = self
            .policies
            .write()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;
        
        if policies.remove(policy_id).is_some() {
            Ok(())
        } else {
            Err(SecurityError::PermissionDenied(format!(
                "Policy {} not found",
                policy_id
            )))
        }
    }

    /// 列出所有策略ID
    pub fn list_policies(&self) -> Result<Vec<String>> {
        let policies = self
            .policies
            .read()
            .map_err(|e| SecurityError::PermissionDenied(format!("Lock poisoned: {}", e)))?;
        
        Ok(policies.keys().cloned().collect())
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_policy() {
        let policy = SecurityPolicy::new("test-policy", "Test policy")
            .with_allowed_capabilities(vec![Capability::NetworkAccess])
            .with_denied_capabilities(vec![Capability::FileSystemWrite])
            .with_default_deny(true);

        assert!(policy.allows_capability(&Capability::NetworkAccess));
        assert!(!policy.allows_capability(&Capability::FileSystemWrite));
        assert!(!policy.allows_capability(&Capability::FileSystemRead)); // 默认拒绝
    }

    #[test]
    fn test_policy_manager() {
        let manager = PolicyManager::new();

        let policy = SecurityPolicy::new("test-resource", "Test resource policy")
            .with_allowed_capabilities(vec![Capability::NetworkAccess])
            .with_denied_capabilities(vec![Capability::FileSystemWrite]);

        assert!(manager.add_policy(policy).is_ok());

        assert!(manager
            .check_resource_capability("test-resource", &Capability::NetworkAccess)
            .unwrap());
        assert!(!manager
            .check_resource_capability("test-resource", &Capability::FileSystemWrite)
            .unwrap());
        
        assert!(manager
            .check_resource_capability("test-resource", &Capability::FileSystemRead)
            .is_err()); // 没有明确允许，默认拒绝
    }
}