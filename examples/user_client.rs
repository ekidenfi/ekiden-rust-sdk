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
        .local()? // Use local development environment
        .private_key(key_pair.private_key())
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
        .get_user_leverage("0x42f1ee729364e2095a2f08019a56b310ba8980288dd1c6bbbd769f19182c692c")
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
