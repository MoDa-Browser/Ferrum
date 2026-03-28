use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("System call failed: {0}")]
    SystemCallFailed(String),
}

pub type Result<T> = std::result::Result<T, PlatformError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Platform::Unknown;
    }

    pub fn name(&self) -> &str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "macos",
            Platform::Windows => "windows",
            Platform::Unknown => "unknown",
        }
    }

    pub fn supports_sandbox(&self) -> bool {
        matches!(self, Platform::Linux | Platform::MacOS)
    }
}

pub struct PlatformInfo {
    platform: Platform,
    arch: String,
}

impl PlatformInfo {
    pub fn new() -> Self {
        Self {
            platform: Platform::current(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn arch(&self) -> &str {
        &self.arch
    }

    pub fn is_supported(&self) -> bool {
        self.platform != Platform::Unknown
    }
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_platform_info() {
        let info = PlatformInfo::new();
        assert!(info.is_supported());
    }
}
