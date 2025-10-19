use aptos_crypto::{
    PrivateKey, ValidCryptoMaterialStringExt,
};
use ekiden_core::sequencer::{ActionPayload, OrderCreate, OrderCreateAction, SigningIntent, TimeInForce};
use ekiden_rust_sdk::{EkidenClient, EkidenClientBuilder, KeyPair, SendIntentParams};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Ekiden SDK Basic Example");

    // Check for private key argument, otherwise generate a new key pair
    let args: Vec<String> = std::env::args().collect();

    let owner_key = if let Some(private_key) = args.get(1) {
        println!("Using provided private key {}", private_key);
        KeyPair::from_private_key(private_key)?
    } else {
        println!("No private key provided, generating new key pair");
        KeyPair::generate()
    };

    let trading_key = if let Some(private_key) = args.get(2) {
        println!("Using provided private key {}", private_key);
        KeyPair::from_private_key(private_key)?
    } else {
        println!("No private key provided, generating new key pair");
        KeyPair::generate()
    };

    println!("Public key: {}", owner_key.public_key());
    println!("Private key: {}", owner_key.private_key());

    println!("Public trading key: {}", trading_key.public_key());
    println!("Private trading key: {}", trading_key.private_key());

    // Create client with configuration
    let client = EkidenClientBuilder::new()
        .staging()? // Use local development environment
        .private_key(owner_key.private_key())
        .trading_private_key(trading_key.private_key())
        .timeout(Duration::from_secs(10))
        .with_logging(true)
        .build_and_auth()
        .await?;

    // Check if we can connect (optional - for demo purposes)
    println!("✅ Client created successfully");
    demonstrate_authenticated_api(&client, &trading_key.private_key()).await?;

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
    let market_addr = "0xa3a64c01b11ba0ec46c7b5715ac559d236f8c0b5667eafa3a5ec8e7b65cdc2e6";

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Create individual order
    let order = OrderCreate {
        market_addr: market_addr.to_string(),
        side: "buy".to_string(),
        size: 100000u64,
        price: 109640500000u64,
        r#type: "limit".to_string(),
        leverage: 20u64,
        is_cross: true,
        time_in_force: TimeInForce::GTC,
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
