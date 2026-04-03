use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamespaceConfig {
    pub enable_pid: bool,
    pub enable_network: bool,
    pub enable_mount: bool,
    pub enable_uts: bool,
    pub enable_ipc: bool,
    pub enable_user: bool,
}

impl NamespaceConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pid(mut self, enable: bool) -> Self {
        self.enable_pid = enable;
        self
    }

    pub fn with_network(mut self, enable: bool) -> Self {
        self.enable_network = enable;
        self
    }

    pub fn with_mount(mut self, enable: bool) -> Self {
        self.enable_mount = enable;
        self
    }

    pub fn with_uts(mut self, enable: bool) -> Self {
        self.enable_uts = enable;
        self
    }

    pub fn with_ipc(mut self, enable: bool) -> Self {
        self.enable_ipc = enable;
        self
    }

    pub fn with_user(mut self, enable: bool) -> Self {
        self.enable_user = enable;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_config() {
        let config = NamespaceConfig::new().with_pid(true).with_network(true);

        assert!(config.enable_pid);
        assert!(config.enable_network);
        assert!(!config.enable_mount);
    }
}
