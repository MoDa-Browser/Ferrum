use super::{NetworkError, Result};

#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub verify_certificates: bool,
    pub min_tls_version: TlsVersion,
    pub max_tls_version: TlsVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    Tls1_0,
    Tls1_1,
    Tls1_2,
    Tls1_3,
}

impl TlsConfig {
    pub fn new() -> Self {
        Self {
            verify_certificates: true,
            min_tls_version: TlsVersion::Tls1_2,
            max_tls_version: TlsVersion::Tls1_3,
        }
    }

    pub fn with_verify_certificates(mut self, verify: bool) -> Self {
        self.verify_certificates = verify;
        self
    }

    pub fn with_min_tls_version(mut self, version: TlsVersion) -> Self {
        self.min_tls_version = version;
        self
    }

    pub fn with_max_tls_version(mut self, version: TlsVersion) -> Self {
        self.max_tls_version = version;
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.min_tls_version > self.max_tls_version {
            return Err(NetworkError::TlsError(
                "Min TLS version cannot be greater than max TLS version".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config() {
        let config = TlsConfig::new()
            .with_verify_certificates(true)
            .with_min_tls_version(TlsVersion::Tls1_2);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_tls_config() {
        let config = TlsConfig::new()
            .with_min_tls_version(TlsVersion::Tls1_3)
            .with_max_tls_version(TlsVersion::Tls1_2);

        assert!(config.validate().is_err());
    }
}
