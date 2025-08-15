use crate::error::{EkidenError, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// Configuration for the Ekiden client
#[derive(Debug, Clone)]
pub struct EkidenConfig {
    /// Base URL for the API (e.g., "https://api.ekiden.fi")
    pub base_url: Url,
    /// WebSocket URL (e.g., "wss://api.ekiden.fi/ws")
    pub ws_url: Url,
    /// Request timeout
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
    /// Maximum number of retries for requests
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Whether to enable request/response logging
    pub enable_logging: bool,
    /// API version
    pub api_version: String,
}

impl Default for EkidenConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("http://localhost:3010/api/v1").unwrap(),
            ws_url: Url::parse("ws://localhost:3010/ws").unwrap(),
            timeout: Duration::from_secs(30),
            user_agent: format!("ekiden-rust-sdk/{}", env!("CARGO_PKG_VERSION")),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            enable_logging: false,
            api_version: "v1".to_string(),
        }
    }
}

impl EkidenConfig {
    /// Create a new configuration with the given base URL
    pub fn new<S: AsRef<str>>(base_url: S) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())?;
        let ws_url = Self::derive_ws_url(&base_url)?;

        Ok(Self {
            base_url,
            ws_url,
            ..Default::default()
        })
    }

    /// Create configuration for production environment
    pub fn production() -> Result<Self> {
        Self::new("https://api.ekiden.fi/api/v1")
    }

    /// Create configuration for staging environment
    pub fn staging() -> Result<Self> {
        Self::new("https://api.staging.ekiden.fi/api/v1")
        //   NOTE: wsURL: "wss://api.staging.ekiden.fi/ws",
    }
    /// Create configuration for TESTNET environment
    pub fn testnet() -> Result<Self> {
        Self::new("https://api.staging.ekiden.fi/api/v1")
    }

    /// Create configuration for local development
    pub fn local() -> Result<Self> {
        Self::new("http://localhost:3010/api/v1")
    }

    /// Set the WebSocket URL
    pub fn with_ws_url<S: AsRef<str>>(mut self, ws_url: S) -> Result<Self> {
        self.ws_url = Url::parse(ws_url.as_ref())?;
        Ok(self)
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the user agent
    pub fn with_user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the retry delay
    pub fn with_retry_delay(mut self, retry_delay: Duration) -> Self {
        self.retry_delay = retry_delay;
        self
    }

    /// Enable or disable logging
    pub fn with_logging(mut self, enable_logging: bool) -> Self {
        self.enable_logging = enable_logging;
        self
    }

    /// Set the API version
    pub fn with_api_version<S: Into<String>>(mut self, api_version: S) -> Self {
        self.api_version = api_version.into();
        self
    }

    /// Derive WebSocket URL from HTTP URL
    fn derive_ws_url(base_url: &Url) -> Result<Url> {
        let mut ws_url = base_url.clone();

        // Convert HTTP(S) scheme to WS(S)
        match ws_url.scheme() {
            "http" => ws_url
                .set_scheme("ws")
                .map_err(|_| EkidenError::config("Failed to set WS scheme"))?,
            "https" => ws_url
                .set_scheme("wss")
                .map_err(|_| EkidenError::config("Failed to set WSS scheme"))?,
            _ => {
                return Err(EkidenError::config(
                    "Invalid URL scheme, expected http or https",
                ));
            }
        }

        // Remove /api/v1 from path and add /ws
        ws_url.set_path("/ws");

        Ok(ws_url)
    }

    /// Get the full API URL for a given path
    pub fn api_url(&self, path: &str) -> String {
        let mut url = self.base_url.clone();
        let current_path = url.path().trim_end_matches('/');
        let new_path = format!("{}/{}", current_path, path.trim_start_matches('/'));
        url.set_path(&new_path);
        url.to_string()
    }

    /// Get the WebSocket URL
    pub fn websocket_url(&self) -> &Url {
        &self.ws_url
    }
}

/// Environment-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Production,
    Staging,
    Development,
    Local,
    Custom(String),
}

impl Environment {
    pub fn base_url(&self) -> &str {
        match self {
            Environment::Production => "https://api.ekiden.fi/api/v1",
            Environment::Staging => "https://staging-api.ekiden.fi/api/v1",
            Environment::Development => "https://dev-api.ekiden.fi/api/v1",
            Environment::Local => "http://localhost:3010/api/v1",
            Environment::Custom(url) => url,
        }
    }

    pub fn ws_url(&self) -> &str {
        match self {
            Environment::Production => "wss://api.ekiden.fi/ws",
            Environment::Staging => "wss://staging-api.ekiden.fi/ws",
            Environment::Development => "wss://dev-api.ekiden.fi/ws",
            Environment::Local => "ws://localhost:3010/ws",
            Environment::Custom(_) => "ws://localhost:3010/ws", // Default for custom URLs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = EkidenConfig::default();
        assert_eq!(config.base_url.as_str(), "http://localhost:3010/api/v1");
        assert_eq!(config.ws_url.as_str(), "ws://localhost:3010/ws");
    }

    #[test]
    fn test_api_url_generation() {
        let config = EkidenConfig::default();
        assert_eq!(
            config.api_url("orders"),
            "http://localhost:3010/api/v1/orders"
        );
        assert_eq!(
            config.api_url("/orders"),
            "http://localhost:3010/api/v1/orders"
        );
    }

    #[test]
    fn test_ws_url_derivation() {
        let config = EkidenConfig::new("https://api.example.com/api/v1").unwrap();
        assert_eq!(config.ws_url.as_str(), "wss://api.example.com/ws");
    }
}
