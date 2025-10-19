use ekiden_rust_sdk::{
    EkidenClient, EkidenClientBuilder, KeyPair,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Ekiden SDK Basic Example");

    // Check for private key argument, otherwise generate a new key pair
    let args: Vec<String> = std::env::args().collect();
    let owner = args
        .get(1)
        .ok_or_else(|| anyhow::anyhow!("Missing argument for private key"))?;
    let owner_key = KeyPair::from_private_key(owner)?;
    let funding = args
        .get(2)
        .ok_or_else(|| anyhow::anyhow!("Missing argument for private key"))?;
    let funding_key = KeyPair::from_private_key(funding)?;
    let trading = args
        .get(3)
        .ok_or_else(|| anyhow::anyhow!("Missing argument for private key"))?;
    let trading_key = KeyPair::from_private_key(trading)?;

    println!("Public key: {}", owner_key.public_key());
    println!("Private key: {}", owner_key.private_key());

    println!("Public funding key: {}", funding_key.public_key());
    println!("Private funding key: {}", funding_key.private_key());

    println!("Public trading key: {}", trading_key.public_key());
    println!("Private trading key: {}", trading_key.private_key());

    // Create client with configuration
    let client = EkidenClientBuilder::new()
        .staging()? // Use local development environment
        .private_key(owner_key.private_key())
        .funding_private_key(funding_key.private_key())
        .trading_private_key(trading_key.private_key())
        .timeout(Duration::from_secs(10))
        .with_logging(true)
        .build_and_auth()
        .await?;

    // Check if we can connect (optional - for demo purposes)
    println!("✅ Client created successfully");
    demonstrate_authenticated_api(&client).await?;

    // Try to authenticate (this might fail if no local API is running)

    println!("🎉 Example completed!");
    Ok(())
}

async fn demonstrate_authenticated_api(
    client: &EkidenClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Demonstrating Authenticated API calls...");

    // Get user portfolio
    let leverage_response = client
        .get_user_leverage("0xa3a64c01b11ba0ec46c7b5715ac559d236f8c0b5667eafa3a5ec8e7b65cdc2e6")
        .await;
    println!("Leverage Response: {:?}", leverage_response);

    // // Get user vaults
    match client.get_all_user_vaults().await {
        Ok(vaults) => {
            println!("✅ Found {} vaults", vaults.len());

            for (i, vault) in vaults.iter().take(3).enumerate() {
                println!(
                    "  {}. Vault {} - Balance: {}",
                    i + 1,
                    vault.addr,
                    vault.amount,
                );
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get vaults: {}", e);
        }
    }

    // // Get user positions
    match client.get_all_user_positions().await {
        Ok(positions) => {
            println!("✅ Found {} positions", positions.len());

            for (i, position) in positions.iter().take(3).enumerate() {
                println!(
                    "  {}. Position - Market: {}, Side: {}, Size: {}",
                    i + 1,
                    position.market_addr,
                    position.side,
                    position.size
                );
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get positions: {}", e);
        }
    }

    Ok(())
}
