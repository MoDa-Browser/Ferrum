use super::{NetworkError, Result};
use reqwest::Client;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await.map_err(|e| {
            NetworkError::ConnectionFailed(format!("GET request failed: {}", e))
        })?;

        let text = response.text().await.map_err(|e| {
            NetworkError::ConnectionFailed(format!("Failed to read response: {}", e))
        })?;

        Ok(text)
    }

    pub async fn post(&self, url: &str, body: &str) -> Result<String> {
        let response = self.client.post(url).body(body.to_string()).send().await.map_err(|e| {
            NetworkError::ConnectionFailed(format!("POST request failed: {}", e))
        })?;

        let text = response.text().await.map_err(|e| {
            NetworkError::ConnectionFailed(format!("Failed to read response: {}", e))
        })?;

        Ok(text)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpClient::new();
        assert_eq!(client.client.timeout(), None);
    }
}
