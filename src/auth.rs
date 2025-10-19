use std::iter;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{EkidenError, Result};
use crate::types::{AuthorizeParams, AuthorizeResponse};
use crate::utils::{format, KeyPair};
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

/// Authentication manager for the Ekiden client
#[derive(Debug, Clone)]
pub struct Auth {
    key_pair: Option<KeyPair>,
    token: Option<String>,
}

impl Auth {
    /// Create a new authentication manager
    pub fn new() -> Self {
        Self {
            key_pair: None,
            token: None,
        }
    }

    /// Set the key pair for signing operations
    pub fn with_key_pair(mut self, key_pair: KeyPair) -> Self {
        self.key_pair = Some(key_pair);
        self
    }

    /// Set the key pair from a private key hex string
    pub fn with_private_key(mut self, private_key: &str) -> Result<Self> {
        let key_pair = KeyPair::from_private_key(private_key)?;
        self.key_pair = Some(key_pair);
        Ok(self)
    }

    /// Set the authentication token
    pub fn with_token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Get the current authentication token
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Set the authentication token
    pub fn set_token<S: Into<String>>(&mut self, token: S) {
        self.token = Some(token.into());
    }

    /// Clear the authentication token
    pub fn clear_token(&mut self) {
        self.token = None;
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get the public key if available
    pub fn public_key(&self) -> Option<String> {
        self.key_pair.as_ref().map(|kp| kp.public_key())
    }

    /// Generate authorization parameters for the /authorize endpoint
    pub fn generate_authorize_params(&self) -> Result<AuthorizeParams> {
        let mut rng = thread_rng();
        let key_pair = self
            .key_pair
            .as_ref()
            .ok_or_else(|| EkidenError::auth("No key pair available for signing"))?;

        
        let public_key = key_pair.public_key();

        // Validate the generated parameters
        format::validate_public_key(&public_key)?;
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let nonce: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(10) // Generate a string of 10 characters
            .collect();

        let signature = key_pair.sign_authorize(timestamp, &nonce);
        format::validate_signature(&signature)?;

        Ok(AuthorizeParams {
            signature: format::normalize_signature(&signature)?,
            public_key: format::normalize_public_key(&public_key)?,
            timestamp_ms: timestamp,
            nonce,
        })
    }

    /// Sign a message with the current key pair
    pub fn sign_message(&self, message: &[u8]) -> Result<String> {
        let key_pair = self
            .key_pair
            .as_ref()
            .ok_or_else(|| EkidenError::auth("No key pair available for signing"))?;

        let signature = key_pair.sign(message);
        format::normalize_signature(&signature)
    }

    /// Sign arbitrary data as JSON string
    pub fn sign_json<T: serde::Serialize>(&self, data: &T) -> Result<String> {
        let json_str = serde_json::to_string(data)?;
        self.sign_message(json_str.as_bytes())
    }

    /// Generate a bearer token header value
    pub fn bearer_token(&self) -> Option<String> {
        self.token.as_ref().map(|token| format!("Bearer {}", token))
    }

    /// Check if a key pair is available
    pub fn has_key_pair(&self) -> bool {
        self.key_pair.is_some()
    }

    /// Process an authorization response and store the token
    pub fn process_authorize_response(&mut self, response: AuthorizeResponse) {
        self.token = Some(response.token);
    }

    /// Create auth headers for HTTP requests
    pub fn auth_headers(&self) -> std::collections::HashMap<String, String> {
        let mut headers = std::collections::HashMap::new();

        if let Some(token) = &self.token {
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        }

        headers
    }

    /// Ensure the client has a valid authentication token
    pub fn ensure_authenticated(&self) -> Result<()> {
        if self.token.is_none() {
            return Err(EkidenError::auth(
                "Not authenticated. Please call authorize() first.",
            ));
        }
        Ok(())
    }

    /// Ensure the client has a key pair for signing
    pub fn ensure_key_pair(&self) -> Result<&KeyPair> {
        self.key_pair
            .as_ref()
            .ok_or_else(|| EkidenError::auth("No key pair available. Please set a private key."))
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder pattern for creating authenticated clients
pub struct AuthBuilder {
    auth: Auth,
}

impl AuthBuilder {
    /// Create a new auth builder
    pub fn new() -> Self {
        Self { auth: Auth::new() }
    }

    /// Set the private key
    pub fn private_key<S: AsRef<str>>(mut self, private_key: S) -> Result<Self> {
        self.auth = self.auth.with_private_key(private_key.as_ref())?;
        Ok(self)
    }

    /// Set the authentication token
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.auth = self.auth.with_token(token);
        self
    }

    /// Set the key pair
    pub fn key_pair(mut self, key_pair: KeyPair) -> Self {
        self.auth = self.auth.with_key_pair(key_pair);
        self
    }

    /// Build the auth instance
    pub fn build(self) -> Auth {
        self.auth
    }
}

impl Default for AuthBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::KeyPair;

    #[test]
    fn test_auth_creation() {
        let auth = Auth::new();
        assert!(!auth.is_authenticated());
        assert!(!auth.has_key_pair());
    }

    #[test]
    fn test_auth_with_key_pair() {
        let key_pair = KeyPair::generate();
        let auth = Auth::new().with_key_pair(key_pair);

        assert!(auth.has_key_pair());
        assert!(auth.public_key().is_some());
    }

    #[test]
    fn test_auth_with_private_key() {
        let key_pair = KeyPair::generate();
        let private_key = key_pair.private_key();

        let auth = Auth::new().with_private_key(&private_key).unwrap();
        assert!(auth.has_key_pair());
        assert_eq!(auth.public_key().unwrap(), key_pair.public_key());
    }

    #[test]
    fn test_auth_with_token() {
        let auth = Auth::new().with_token("test_token");
        assert!(auth.is_authenticated());
        assert_eq!(auth.token(), Some("test_token"));
    }

    #[test]
    fn test_generate_authorize_params() {
        let key_pair = KeyPair::generate();
        let auth = Auth::new().with_key_pair(key_pair.clone());

        let params = auth.generate_authorize_params().unwrap();
        assert!(!params.signature.is_empty());
        assert_eq!(params.public_key, key_pair.public_key());
    }

    #[test]
    fn test_sign_message() {
        let key_pair = KeyPair::generate();
        let auth = Auth::new().with_key_pair(key_pair);

        let message = b"test message";
        let signature = auth.sign_message(message).unwrap();
        assert!(!signature.is_empty());
        assert!(signature.starts_with("0x"));
    }

    #[test]
    fn test_auth_builder() {
        let key_pair = KeyPair::generate();
        let private_key = key_pair.private_key();

        let auth = AuthBuilder::new()
            .private_key(&private_key)
            .unwrap()
            .token("test_token")
            .build();

        assert!(auth.is_authenticated());
        assert!(auth.has_key_pair());
        assert_eq!(auth.token(), Some("test_token"));
    }

    #[test]
    fn test_ensure_authenticated() {
        let auth = Auth::new();
        assert!(auth.ensure_authenticated().is_err());

        let auth = Auth::new().with_token("test_token");
        assert!(auth.ensure_authenticated().is_ok());
    }

    #[test]
    fn test_ensure_key_pair() {
        let auth = Auth::new();
        assert!(auth.ensure_key_pair().is_err());

        let key_pair = KeyPair::generate();
        let auth = Auth::new().with_key_pair(key_pair);
        assert!(auth.ensure_key_pair().is_ok());
    }

    #[test]
    fn test_auth_headers() {
        let auth = Auth::new().with_token("test_token");
        let headers = auth.auth_headers();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test_token".to_string())
        );
    }
}
