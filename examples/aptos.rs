use ekiden_rust_sdk::aptos::vault::VaultContract;
use std::time::Duration;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    println!("🚀 Starting Ekiden SDK Vault Example");

    let testnet_usdc = "0x9967e130f7419f791c240acc17dde966ec84ad41652e2e87083ee613f460d019";

    let ekiden_contract = "0xa436c4c966963e91da2471718cdfa6df58182ff171c7fdb07655a3bc2dc63ff9";
    let private = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Missing argument for private key"))?;
    let vault_contract = VaultContract::new(ekiden_contract, testnet_usdc, "testnet");
    let deposit_tx = vault_contract
        .deposit_into_user(10000000u64, &private)
        .await?;
    println!("Deposit transaction: {:?}", deposit_tx);
    // sleep 2 second
    tokio::time::sleep(Duration::from_secs(2)).await;
    let withdraw_tx = vault_contract
        .withdraw_from_user(50000u64, &private)
        .await?;
    println!("Withdraw transaction: {:?}", withdraw_tx);

    Ok(())
}
