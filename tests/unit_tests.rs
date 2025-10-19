use std::iter;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ekiden_rust_sdk::{
    utils::{format, Crypto},
    Auth, EkidenClient, EkidenConfig, EkidenError, KeyPair, OrderSide, Pagination,
};

#[tokio::test]
async fn test_client_creation() {
    let client = EkidenClient::default_config().unwrap();
    assert!(!client.is_authenticated().await);
}

#[tokio::test]
async fn test_config_creation() {
    let config = EkidenConfig::local().unwrap();
    assert_eq!(config.base_url.as_str(), "http://localhost:3010/api/v1");

    let config = EkidenConfig::new("https://example.com/api/v1").unwrap();
    assert_eq!(config.base_url.as_str(), "https://example.com/api/v1");
}

#[tokio::test]
async fn test_key_pair_generation() {
    let key_pair = KeyPair::generate();

    let public_key = key_pair.public_key();
    let private_key = key_pair.private_key();

    assert!(public_key.starts_with("0x"));
    assert_eq!(public_key.len(), 66); // 0x + 64 hex chars
    assert!(private_key.starts_with("0x"));
    assert_eq!(private_key.len(), 66);
}

#[tokio::test]
async fn test_key_pair_from_private_key() {
    let key_pair1 = KeyPair::generate();
    let private_key_hex = key_pair1.private_key();

    let key_pair2 = KeyPair::from_private_key(&private_key_hex).unwrap();

    assert_eq!(key_pair1.public_key(), key_pair2.public_key());
    assert_eq!(key_pair1.private_key(), key_pair2.private_key());
}

#[tokio::test]
async fn test_signature_verification() {
    let key_pair = KeyPair::generate();
    let message = b"test message";

    let signature = key_pair.sign(message);
    let public_key = key_pair.public_key();

    let is_valid = Crypto::verify_signature(message, &signature, &public_key).unwrap();
    assert!(is_valid);

    // Test with wrong message
    let is_valid = Crypto::verify_signature(b"wrong message", &signature, &public_key).unwrap();
    assert!(!is_valid);
}

#[tokio::test]
async fn test_authorize_signature() {
    let key_pair = KeyPair::generate();
    let now = SystemTime::now();
    let mut rng = thread_rng();
    let timestamp = now.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
    let nonce: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(10) // Generate a string of 10 characters
        .collect();
    let signature = key_pair.sign_authorize(timestamp, &nonce);
    let public_key = key_pair.public_key();

    let is_valid = Crypto::verify_signature(format!("AUTHORIZE|{}|{}", timestamp, nonce).as_bytes(), &signature, &public_key).unwrap();
    assert!(is_valid);
}

#[tokio::test]
async fn test_auth_creation() {
    let auth = Auth::new();
    assert!(!auth.is_authenticated());
    assert!(!auth.has_key_pair());
    assert!(auth.token().is_none());
}

#[tokio::test]
async fn test_auth_with_key_pair() {
    let key_pair = KeyPair::generate();
    let auth = Auth::new().with_key_pair(key_pair.clone());

    assert!(auth.has_key_pair());
    assert_eq!(auth.public_key().unwrap(), key_pair.public_key());
}

#[tokio::test]
async fn test_auth_generate_authorize_params() {
    let key_pair = KeyPair::generate();
    let auth = Auth::new().with_key_pair(key_pair.clone());

    let params = auth.generate_authorize_params().unwrap();
    assert!(!params.signature.is_empty());
    assert_eq!(params.public_key, key_pair.public_key());

    // Verify the signature
    let is_valid =
        Crypto::verify_signature(b"AUTHORIZE", &params.signature, &params.public_key).unwrap();
    assert!(is_valid);
}

#[test]
fn test_hex_prefix_handling() {
    assert_eq!(format::ensure_hex_prefix("123"), "0x123");
    assert_eq!(format::ensure_hex_prefix("0x123"), "0x123");
    assert_eq!(format::strip_hex_prefix("0x123"), "123");
    assert_eq!(format::strip_hex_prefix("123"), "123");
}

#[tokio::test]
async fn test_client_auth_flow() {
    let client = EkidenClient::default_config().unwrap();
    let key_pair = KeyPair::generate();

    // Set private key
    client
        .set_private_key(&key_pair.private_key())
        .await
        .unwrap();

    // Check that we have the right public key and address
    assert_eq!(client.public_key().await.unwrap(), key_pair.public_key());

    // We're not authenticated yet
    assert!(!client.is_authenticated().await);

    // Set a token manually
    client.set_token("test_token").await;
    assert!(client.is_authenticated().await);
    assert_eq!(client.token().await.unwrap(), "test_token");
}

#[test]
fn test_pagination() {
    let pagination = Pagination::new(50, 0);
    assert_eq!(pagination.limit, Some(50));
    assert_eq!(pagination.offset, Some(0));

    let pagination = Pagination::with_page(1, 25);
    assert_eq!(pagination.page, Some(1));
    assert_eq!(pagination.page_size, Some(25));

    let query_params = pagination.to_query_params();
    assert_eq!(query_params.get("page"), Some(&"1".to_string()));
    assert_eq!(query_params.get("page_size"), Some(&"25".to_string()));
}

#[test]
fn test_order_side_serialization() {
    let side = OrderSide::Buy;
    let serialized = serde_json::to_string(&side).unwrap();
    assert_eq!(serialized, "\"buy\"");

    let side = OrderSide::Sell;
    let serialized = serde_json::to_string(&side).unwrap();
    assert_eq!(serialized, "\"sell\"");
}

#[tokio::test]
async fn test_error_types() {
    // Test different error types
    let auth_error = EkidenError::auth("test auth error");
    assert!(matches!(auth_error, EkidenError::Auth(_)));

    let config_error = EkidenError::config("test config error");
    assert!(matches!(config_error, EkidenError::Config(_)));

    let api_error = EkidenError::api(404, "Not found".to_string());
    assert!(matches!(api_error, EkidenError::Api { status: 404, .. }));

    let validation_error = EkidenError::validation("test validation error");
    assert!(matches!(validation_error, EkidenError::Validation(_)));
}

#[test]
fn test_ws_request_serialization() {
    use ekiden_rust_sdk::WsRequest;

    let ping = WsRequest::Ping;
    let serialized = serde_json::to_string(&ping).unwrap();
    assert!(serialized.contains("ping"));

    let subscribe = WsRequest::Subscribe {
        channel: "orderbook/0x123".to_string(),
    };
    let serialized = serde_json::to_string(&subscribe).unwrap();
    assert!(serialized.contains("subscribe"));
    assert!(serialized.contains("orderbook/0x123"));
}
