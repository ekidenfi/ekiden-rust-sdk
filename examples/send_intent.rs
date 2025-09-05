use aptos_crypto::{
    ed25519::Ed25519PrivateKey, PrivateKey, SigningKey, ValidCryptoMaterialStringExt,
};
use ekiden_core::sequencer::{
    ActionPayload, IntentSignatureBody, OrderCreate, OrderCreateAction, SigningIntent,
};
use ekiden_rust_sdk::{EkidenClient, EkidenClientBuilder, KeyPair, SendIntentParams};
use serde_json::json;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Ekiden SDK Basic Example");

    // Check for private key argument, otherwise generate a new key pair
    let args: Vec<String> = std::env::args().collect();
    let key_pair = if args.len() > 1 {
        // Use provided private key
        let private_key = &args[1];
        println!("Using provided private key {}", private_key);
        KeyPair::from_private_key(private_key)?
    } else {
        // Generate a new key pair for this example
        println!("No private key provided, generating new key pair");
        KeyPair::generate()
    };

    println!("Public key: {}", key_pair.public_key());
    println!("Private key: {}", key_pair.private_key());

    // Create client with configuration
    let client = EkidenClientBuilder::new()
        .staging()? // Use local development environment
        .private_key(&key_pair.private_key())
        .timeout(Duration::from_secs(10))
        .with_logging(true)
        .build_and_auth()
        .await?;

    // Check if we can connect (optional - for demo purposes)
    println!("✅ Client created successfully");
    demonstrate_authenticated_api(&client, &key_pair.private_key()).await?;

    // Try to authenticate (this might fail if no local API is running)

    println!("🎉 Example completed!");
    Ok(())
}

async fn demonstrate_authenticated_api(
    client: &EkidenClient,
    private_key_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Demonstrating Authenticated API calls...");

    // Create order create intent
    let market_addr = "0x0de96fc00b890cbda64ffbaa131c2a3fa5602b63bcb4331de02997fc9e14353b";

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create individual order
    let order = OrderCreate {
        market_addr: market_addr.to_string(),
        side: "buy".to_string(),
        size: 100000000u64,
        price: 3000000u64,
        r#type: "limit".to_string(),
        leverage: 10u64,
    };

    // Create the action with vector of orders
    let payload = ActionPayload::OrderCreate(OrderCreateAction {
        orders: vec![order], // Vector containing the order
    });

    let signature = client
        .sign_intent(private_key_str, &payload, nonce)
        .map_err(|e| {
            println!("Failed to sign intent: {}", e);
            e
        })?;

    // Send intent to the API
    let intent_body = SendIntentParams {
        payload,
        nonce,
        signature: signature.to_encoded_string()?,
    };

    // Get user portfolio
    let send_res = client.send_intent(intent_body).await;
    println!("Send Intent Response: {:?}", send_res);

    Ok(())
}
