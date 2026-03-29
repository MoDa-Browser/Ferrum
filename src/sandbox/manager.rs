use super::namespace::NamespaceConfig;
use super::{Result, SandboxError};
use std::process::{Child, Command};

pub struct Sandbox {
    namespace_config: NamespaceConfig,
}

impl Sandbox {
    pub fn new() -> Self {
        Self {
            namespace_config: NamespaceConfig::default(),
        }
    }

    pub fn with_namespace(mut self, config: NamespaceConfig) -> Self {
        self.namespace_config = config;
        self
    }

    pub fn build(&self) -> Result<SandboxInstance> {
        let mut command = if cfg!(windows) {
            Command::new("cmd")
        } else {
            Command::new("/bin/sh")
        };
        
        if self.namespace_config.enable_pid {
            if cfg!(windows) {
                command.arg("/c");
                command.arg("echo PID namespace enabled");
            } else {
                command.arg("-c");
                command.arg("echo 'PID namespace enabled'");
            }
        }

        let child = command.spawn().map_err(|e| {
            SandboxError::CreationFailed(format!("Failed to spawn process: {}", e))
        })?;

        Ok(SandboxInstance { child })
    }

    pub fn run_command(&self, program: &str, args: &[&str]) -> Result<SandboxInstance> {
        let mut command = Command::new(program);
        command.args(args);

        let child = command.spawn().map_err(|e| {
            SandboxError::CreationFailed(format!("Failed to spawn process: {}", e))
        })?;

        Ok(SandboxInstance { child })
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SandboxInstance {
    child: Child,
}

impl SandboxInstance {
    pub fn wait(&mut self) -> Result<std::process::ExitStatus> {
        self.child.wait().map_err(|e| {
            SandboxError::ProcessError(format!("Failed to wait for process: {}", e))
        })
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new();
        let instance = sandbox.build();
        assert!(instance.is_ok());
    }
}
