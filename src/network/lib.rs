pub mod http;
pub mod tls;

pub use http::HttpClient;
pub use tls::TlsConfig;
pub use tls::TlsVersion;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("DNS resolution failed: {0}")]
    DnsError(String),
}

pub type Result<T> = std::result::Result<T, NetworkError>;
