use ekiden_rust_sdk::{EkidenClient, EkidenClientBuilder, KeyPair, ListOrdersParams, Pagination};
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
        .staging()? // Use local development environment
        .private_key(key_pair.private_key())
        .timeout(Duration::from_secs(10))
        .with_logging(true)
        .build()
        .await?;

    // Check if we can connect (optional - for demo purposes)
    println!("✅ Client created successfully");

    // Try to authenticate (this might fail if no local API is running)
    match client.authorize().await {
        Ok(auth_response) => {
            println!("✅ Authenticated successfully");
            println!("Token: {}", auth_response.token);

            // Now we can make authenticated requests
            demonstrate_authenticated_api(&client).await?;
        }
        Err(e) => {
            println!("⚠️  Authentication failed: {}", e);
            println!("This is expected if no local API server is running");
        }
    }

    // Demonstrate public API calls (these work without authentication)
    demonstrate_public_api(&client).await?;

    println!("🎉 Example completed!");
    Ok(())
}

async fn demonstrate_public_api(client: &EkidenClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📊 Demonstrating Public API calls...");

    // Get all markets
    match client.get_markets(Default::default()).await {
        Ok(markets) => {
            println!("✅ Found {} markets", markets.len());

            // Display first few markets
            for (i, market) in markets.iter().take(3).enumerate() {
                println!(
                    "  {}. {} - Base: {}, Quote: {}",
                    i + 1,
                    market.symbol,
                    market.base_addr,
                    market.quote_addr
                );
            }

            // If we have markets, demonstrate other calls
            if let Some(market) = markets.first() {
                demonstrate_market_data(client, &market.base_addr).await?;
            }
        }
        Err(e) => {
            println!("❌ Failed to get markets: {}", e);
        }
    }

    Ok(())
}

async fn demonstrate_market_data(
    client: &EkidenClient,
    market_addr: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📈 Demonstrating Market Data for {}...", market_addr);

    // Get orders for this market
    let order_params = ListOrdersParams {
        market_addr: market_addr.to_string(),
        side: Some("buy".to_string()),
        pagination: Pagination::new(10, 0),
    };

    match client.get_orders(order_params).await {
        Ok(orders) => {
            println!("✅ Found {} buy orders", orders.len());

            for (i, order) in orders.iter().take(3).enumerate() {
                println!(
                    "  {}. Order {} - Size: {}, Price: {}, Status: {}",
                    i + 1,
                    order.sid,
                    order.size,
                    order.price,
                    order.status
                );
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get orders: {}", e);
        }
    }

    // Get recent fills
    match client.get_recent_fills(market_addr, Some(10)).await {
        Ok(fills) => {
            println!("✅ Found {} recent fills", fills.len());

            for (i, fill) in fills.iter().take(3).enumerate() {
                println!(
                    "  {}. Fill {} - Size: {}, Price: {}, Side: {}",
                    i + 1,
                    fill.sid,
                    fill.size,
                    fill.price,
                    fill.side
                );
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get fills: {}", e);
        }
    }

    // Get candles
    match client.get_recent_candles(market_addr, "1h", Some(5)).await {
        Ok(candles) => {
            println!("✅ Found {} candles", candles.len());

            for (i, candle) in candles.iter().take(3).enumerate() {
                println!(
                    "  {}. Candle - Open: {}, High: {}, Low: {}, Close: {}",
                    i + 1,
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close
                );
            }
        }
        Err(e) => {
            println!("⚠️  Failed to get candles: {}", e);
        }
    }

    Ok(())
}

async fn demonstrate_authenticated_api(
    client: &EkidenClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Demonstrating Authenticated API calls...");

    // Get user portfolio
    match client.get_user_portfolio().await {
        Ok(portfolio) => {
            println!("✅ Portfolio retrieved");
            println!("  Total Value: {:?}", portfolio.summary);
            // println!("  Positions: {}", portfolio.positions.len());
            // println!("  Vaults: {}", portfolio.vaults.len());
        }
        Err(e) => {
            println!("⚠️  Failed to get portfolio: {}", e);
        }
    }

    // Get user vaults
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

    // Get user positions
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
