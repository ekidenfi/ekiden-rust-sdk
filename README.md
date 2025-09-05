# Enhanced Ekiden Rust SDK

A comprehensive, type-safe Rust SDK for interacting with the Ekiden Gateway API and WebSocket streams.

## Features

- 🚀 **Complete API Coverage**: All REST endpoints supported
- 🔌 **WebSocket Support**: Real-time orderbook, trades, and user updates
- 🔐 **Built-in Authentication**: Ed25519 signature handling
- 🛡️ **Type Safety**: Full type definitions with validation
- ⚡ **Async/Await**: Modern async Rust with tokio
- 🔧 **Configurable**: Multiple environments and customizable settings
- 📊 **Aptos Integration**: Optional Aptos blockchain utilities
- 🧪 **Well Tested**: Comprehensive test coverage

## Setup

### Environment Variables

Before building or running the project, you need to set up your environment variables:

1. Edit `.env` and replace `your_github_token_here` with your actual GitHub Personal Access Token:

   ```bash
   GITHUB_TOKEN=ghp_your_actual_token_here
   ```

### Building the Project

Once your environment is set up, you can build the project:

```bash
cargo build
```

## Examples

Check the `examples/` directory for complete working examples:

- `basic_client.rs` - Basic API usage
- `websocket_streams.rs` - WebSocket integration
- `aptos.rs` - Aptos integration with deposit/withdraw functionality
- `portfolio_monitor.rs` - Portfolio monitoring

### Running Examples

To test the Aptos integration example with deposit and withdraw functionality:

```bash
cargo run --example aptos -- "YOUR_PRIVATE_KEY"
```

Replace `YOUR_PRIVATE_KEY` with your actual private key to test deposit and withdrawal operations on the Aptos network.

## Quick Start

```rust
use ekiden_rust_sdk::{EkidenClient, EkidenConfig, Pagination};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure client
    let client = EkidenClient::production().await?;

    // Set your private key for authentication
    client.set_private_key("0x1234...").await?;

    // Authenticate with the API
    let auth_response = client.authorize().await?;
    println!("Authenticated with token: {}", auth_response.token);

    // Get markets
    let markets = client.get_markets(Default::default()).await?;
    println!("Found {} markets", markets.len());

    // Get orderbook via WebSocket
    client.connect_websocket().await?;
    let mut orderbook_stream = client.subscribe_orderbook("0x123...").await?;

    // Listen for orderbook updates
    while let Ok(event) = orderbook_stream.recv().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ekiden-rust-sdk = "0.1.0"
```

### Builder Pattern

```rust
let client = EkidenClientBuilder::new()
    .production()?
    .private_key("0x1234...")
    .timeout(Duration::from_secs(10))
    .build_and_auth()
    .await?;
```

## API Methods

### Market Data

```rust
// Get all markets
let markets = client.get_markets(Default::default()).await?;

// Get specific market
let market = client.get_market_by_address("0x123...").await?;
let market = client.get_market_by_symbol("BTC-USD").await?;

// Get orders for a market
let orders = client.get_orders_by_side(
    "0x123...",
    OrderSide::Buy,
    Some(Pagination::new(50, 0))
).await?;

// Get recent trades
let fills = client.get_recent_fills("0x123...", Some(100)).await?;

// Get candlestick data
let candles = client.get_recent_candles("0x123...", "1h", Some(100)).await?;

// Get funding rates
let funding_rate = client.get_current_funding_rate("0x123...").await?;
```

### User Data (requires authentication)

```rust
// Get portfolio
let portfolio = client.get_user_portfolio().await?;

// Get positions
let positions = client.get_all_user_positions().await?;
let market_positions = client.get_user_positions_by_market("0x123...").await?;

// Get vaults (balances)
let vaults = client.get_all_user_vaults().await?;

// Get/Set leverage
let leverage = client.get_user_leverage("0x123...").await?;
client.set_user_leverage("0x123...", 10).await?;

// Get transaction history
let deposits = client.get_user_deposits("0xuser...").await?;
let withdrawals = client.get_user_withdrawals("0xuser...").await?;
```

### Trading (requires authentication)

```rust
use ekiden_rust_sdk::{SendIntentParams, ActionPayload};

// Create and send an intent (order, etc.)
let intent_params = SendIntentParams {
    actions: vec![
        ActionPayload {
            action_type: "place_order".to_string(),
            data: serde_json::json!({
                "market_addr": "0x123...",
                "side": "buy",
                "size": "1000000", // in base units
                "price": "50000000000", // in quote units
                "order_type": "limit"
            }),
        }
    ],
    signature: "0xsignature...".to_string(),
};

let result = client.send_intent(intent_params).await?;
```

## WebSocket Streams

### Real-time Market Data

```rust
// Connect to WebSocket
client.connect_websocket().await?;

// Subscribe to orderbook updates
let mut orderbook_rx = client.subscribe_orderbook("0x123...").await?;
tokio::spawn(async move {
    while let Ok(event) = orderbook_rx.recv().await {
        if let WsEvent::OrderbookSnapshot { bids, asks, .. } = event {
            println!("Orderbook - Bids: {}, Asks: {}", bids.len(), asks.len());
        }
    }
});

// Subscribe to trade updates
let mut trades_rx = client.subscribe_trades("0x123...").await?;
tokio::spawn(async move {
    while let Ok(event) = trades_rx.recv().await {
        if let WsEvent::Trade { price, size, side, .. } = event {
            println!("Trade: {} {} at {}", side, size, price);
        }
    }
});

// Subscribe to user updates (orders, positions, balances)
let user_addr = client.address().await?.unwrap();
let mut user_rx = client.subscribe_user(&user_addr).await?;
tokio::spawn(async move {
    while let Ok(event) = user_rx.recv().await {
        match event {
            WsEvent::OrderUpdate { order } => {
                println!("Order update: {} - {}", order.sid, order.status);
            }
            WsEvent::PositionUpdate { position } => {
                println!("Position update: {} {}", position.side, position.size);
            }
            WsEvent::BalanceUpdate { vault } => {
                println!("Balance update: {}", vault.available_balance);
            }
            _ => {}
        }
    }
});
```

### WebSocket Channel Management

```rust
use enhanced_ekiden_rust_sdk::ws::channels;

// Subscribe to specific channels
client.subscribe(&channels::orderbook("0x123...")).await?;
client.subscribe(&channels::trades("0x456...")).await?;
client.subscribe(&channels::user("0x789...")).await?;

// Check active subscriptions
let subscriptions = client.active_subscriptions().await;
println!("Active subscriptions: {:?}", subscriptions);

// Unsubscribe
client.unsubscribe(&channels::orderbook("0x123...")).await?;

// Disconnect
client.disconnect_websocket().await?;
```

## Testing

Run tests with:

```bash
# All tests
cargo test

# Integration tests (requires running API)
cargo test --test integration_tests

# With Aptos features
cargo test --features aptos

# With logging
RUST_LOG=debug cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details.
