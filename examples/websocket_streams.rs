use ekiden_rust_sdk::{ws::channels, EkidenClient, KeyPair, WsEvent};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see WebSocket events
    tracing_subscriber::fmt::init();

    println!("🔌 Starting WebSocket Streams Example");

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

    // Create client
    let client = EkidenClient::local()?;
    client.set_private_key(&key_pair.private_key()).await?;

    // Connect to WebSockets
    match client.connect_websocket().await {
        Ok(_) => println!("✅ WebSocket connected"),
        Err(e) => {
            println!("❌ Failed to connect WebSocket: {}", e);
            println!("Make sure the Ekiden Gateway API is running locally");
            return Ok(());
        }
    }

    // Wait a moment for connection to stabilize
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Example market address (you might need to adjust this)
    let market_addr = "0x2245549db5a39c5644f4ff69aaab297df252649b8b59e18a9130d48f01d09ea";

    // Start multiple concurrent streams
    let client_clone1 = client.clone();
    let client_clone2 = client.clone();
    let client_clone3 = client.clone();

    let market_addr1 = market_addr.to_string();
    let market_addr2 = market_addr.to_string();
    let user_addr = key_pair.public_key(); // Use public key as user identifier

    // Spawn orderbook stream handler
    let orderbook_handle = tokio::spawn(async move {
        if let Ok(mut orderbook_rx) = client_clone1.subscribe_orderbook(&market_addr1).await {
            println!("📊 Subscribed to orderbook for {}", market_addr1);

            let mut count = 0;
            while count < 5 {
                // Limit to 5 events for demo
                match timeout(Duration::from_secs(10), orderbook_rx.recv()).await {
                    Ok(Ok(event)) => {
                        match event {
                            WsEvent::OrderbookSnapshot {
                                market_addr,
                                bids,
                                asks,
                                timestamp,
                            } => {
                                println!("📸 Orderbook Snapshot for {}:", market_addr);
                                println!(
                                    "  Bids: {} levels, Asks: {} levels",
                                    bids.len(),
                                    asks.len()
                                );
                                println!("  Timestamp: {}", timestamp);

                                // Show top levels
                                if let Some(best_bid) = bids.first() {
                                    println!("  Best Bid: {} @ {}", best_bid.size, best_bid.price);
                                }
                                if let Some(best_ask) = asks.first() {
                                    println!("  Best Ask: {} @ {}", best_ask.size, best_ask.price);
                                }
                            }
                            WsEvent::OrderbookUpdate {
                                market_addr,
                                bids,
                                asks,
                                timestamp,
                            } => {
                                println!("🔄 Orderbook Update for {}:", market_addr);
                                println!("  Updated bids: {}, asks: {}", bids.len(), asks.len());
                                println!("  Timestamp: {}", timestamp);
                            }
                            _ => {}
                        }
                        count += 1;
                    }
                    Ok(Err(e)) => {
                        println!("❌ Orderbook stream error: {}", e);
                        break;
                    }
                    Err(_) => {
                        println!("⏰ Orderbook stream timeout");
                        break;
                    }
                }
            }
        } else {
            println!("❌ Failed to subscribe to orderbook");
        }
    });

    // Spawn trades stream handler
    let trades_handle = tokio::spawn(async move {
        if let Ok(mut trades_rx) = client_clone2.subscribe_trades(&market_addr2).await {
            println!("💰 Subscribed to trades for {}", market_addr2);

            let mut count = 0;
            while count < 3 {
                // Limit to 3 events for demo
                match timeout(Duration::from_secs(10), trades_rx.recv()).await {
                    Ok(Ok(event)) => {
                        if let WsEvent::Trade {
                            market_addr,
                            price,
                            size,
                            side,
                            timestamp,
                        } = event
                        {
                            println!("💸 New Trade in {}:", market_addr);
                            println!("  {} {} at price {}", side.to_uppercase(), size, price);
                            println!("  Timestamp: {}", timestamp);
                            count += 1;
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ Trades stream error: {}", e);
                        break;
                    }
                    Err(_) => {
                        println!("⏰ Trades stream timeout");
                        break;
                    }
                }
            }
        } else {
            println!("❌ Failed to subscribe to trades");
        }
    });

    // Spawn user stream handler (requires authentication)
    let user_handle = tokio::spawn(async move {
        // Try to authenticate first
        match client_clone3.authorize().await {
            Ok(_) => {
                println!("🔐 Authenticated for user stream");

                if let Ok(mut user_rx) = client_clone3.subscribe_user(&user_addr).await {
                    println!("👤 Subscribed to user updates for {}", user_addr);

                    let mut count = 0;
                    while count < 3 {
                        // Limit to 3 events for demo
                        match timeout(Duration::from_secs(5), user_rx.recv()).await {
                            Ok(Ok(event)) => {
                                match event {
                                    WsEvent::OrderUpdate { order } => {
                                        println!("📋 Order Update:");
                                        println!("  Order ID: {}", order.sid);
                                        println!("  Status: {}", order.status);
                                        println!(
                                            "  Side: {}, Size: {}, Price: {}",
                                            order.side, order.size, order.price
                                        );
                                    }
                                    WsEvent::PositionUpdate { position } => {
                                        println!("📍 Position Update:");
                                        println!("  Market: {}", position.market_addr);
                                        println!(
                                            "  Side: {}, Size: {}",
                                            position.side, position.size
                                        );
                                        println!(
                                            "  Entry Price: {}, Mark Price: {}",
                                            position.entry_price, position.mark_price
                                        );
                                    }
                                    WsEvent::BalanceUpdate { vault } => {
                                        println!("💳 Balance Update:");
                                        println!("  Vault: {}", vault.vault_addr);
                                        println!(
                                            "  Available: {}, Locked: {}",
                                            vault.available_balance, vault.locked_balance
                                        );
                                    }
                                    _ => {}
                                }
                                count += 1;
                            }
                            Ok(Err(e)) => {
                                println!("❌ User stream error: {}", e);
                                break;
                            }
                            Err(_) => {
                                println!("⏰ User stream timeout (no user events)");
                                break;
                            }
                        }
                    }
                } else {
                    println!("❌ Failed to subscribe to user updates");
                }
            }
            Err(e) => {
                println!("❌ Failed to authenticate for user stream: {}", e);
            }
        }
    });

    // Demonstrate manual channel subscription
    let custom_channel = channels::orderbook("0x1234567890abcdef");
    println!(
        "📡 Attempting to subscribe to custom channel: {}",
        custom_channel
    );

    // Wait for all streams to process some events
    println!("\n⏳ Waiting for stream events (up to 15 seconds)...");

    let (orderbook_result, trades_result, user_result) =
        tokio::join!(orderbook_handle, trades_handle, user_handle);

    // Check results
    if let Err(e) = orderbook_result {
        println!("❌ Orderbook task error: {}", e);
    }
    if let Err(e) = trades_result {
        println!("❌ Trades task error: {}", e);
    }
    if let Err(e) = user_result {
        println!("❌ User task error: {}", e);
    }

    // Show active subscriptions
    if client.is_websocket_connected().await {
        println!("\n📊 WebSocket connection status: Connected");
    } else {
        println!("\n📊 WebSocket connection status: Disconnected");
    }

    // Cleanup
    println!("\n🧹 Cleaning up...");
    if let Err(e) = client.disconnect_websocket().await {
        println!("⚠️  Error disconnecting WebSocket: {}", e);
    } else {
        println!("✅ WebSocket disconnected");
    }

    println!("🎉 WebSocket example completed!");
    Ok(())
}
