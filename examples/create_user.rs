use std::time::Duration;
use ekiden_rust_sdk::{EkidenClientBuilder, KeyPair};
use ekiden_rust_sdk::vault::VaultContract;

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

    let funding_key = if let Some(private_key) = args.get(2) {
        println!("Using provided private key {}", private_key);
        KeyPair::from_private_key(private_key)?
    } else {
        println!("No private key provided, generating new key pair");
        KeyPair::generate()
    };

    let trading_key = if let Some(private_key) = args.get(3) {
        println!("Using provided private key {}", private_key);
        KeyPair::from_private_key(private_key)?
    } else {
        println!("No private key provided, generating new key pair");
        KeyPair::generate()
    };

    println!("Public key: {}", owner_key.public_key());
    println!("Private key: {}", owner_key.private_key());

    println!("Public funding key: {}", funding_key.public_key());
    println!("Private funding key: {}", funding_key.private_key());

    println!("Public trading key: {}", trading_key.public_key());
    println!("Private trading key: {}", trading_key.private_key());

    let testnet_usdc = "0x9967e130f7419f791c240acc17dde966ec84ad41652e2e87083ee613f460d019";

    let ekiden_contract = "0xa436c4c966963e91da2471718cdfa6df58182ff171c7fdb07655a3bc2dc63ff9";
    let vault_contract = VaultContract::new(ekiden_contract, testnet_usdc, "testnet");
    vault_contract.create_ekiden_user(&owner_key, &funding_key, &trading_key).await?;
    Ok(())
}
