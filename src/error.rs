use thiserror::Error;

pub type Result<T> = std::result::Result<T, EkidenError>;

#[derive(Error, Debug)]
pub enum EkidenError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Tungstenite WebSocket error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Cryptography error: {0}")]
    Crypto(String),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("General error: {0}")]
    General(String),

    #[error("Timeout error")]
    Timeout,

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Aptos error: {0}")]
    Aptos(String),
}

impl EkidenError {
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::Auth(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }

    pub fn crypto<S: Into<String>>(msg: S) -> Self {
        Self::Crypto(msg.into())
    }

    pub fn api(status: u16, message: String) -> Self {
        Self::Api { status, message }
    }

    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    pub fn general<S: Into<String>>(msg: S) -> Self {
        Self::General(msg.into())
    }
    pub fn aptos<S: Into<String>>(msg: S) -> Self {
        Self::Aptos(msg.into())
    }
}
