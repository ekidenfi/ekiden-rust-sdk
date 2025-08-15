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

# For Aptos blockchain utilities (optional)
ekiden-rust-sdk = { version = "0.1.0", features = ["aptos"] }
```

## Configuration

### Environment Presets

```rust
// Production
let client = EkidenClient::production().await?;

// Staging
let client = EkidenClient::staging().await?;

// Local development
let client = EkidenClient::local().await?;
```

### Custom Configuration

```rust
use enhanced_ekiden_rust_sdk::{EkidenConfig, EkidenClient};
use std::time::Duration;

let config = EkidenConfig::new("https://your-api.com/api/v1")?
    .with_timeout(Duration::from_secs(30))
    .with_user_agent("MyApp/1.0")
    .with_logging(true);

let client = EkidenClient::new(config)?;
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

## Authentication

The SDK handles Ed25519 signature-based authentication:

```rust
use enhanced_ekiden_rust_sdk::{KeyPair, Auth};

// Generate a new key pair
let key_pair = KeyPair::generate();
println!("Address: {}", key_pair.address()?);
println!("Public Key: {}", key_pair.public_key());

// Or use an existing private key
let key_pair = KeyPair::from_private_key("0x1234...")?;

// Set up authentication
client.set_private_key(&key_pair.private_key()).await?;
let auth_response = client.authorize().await?;
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
use enhanced_ekiden_rust_sdk::{SendIntentParams, ActionPayload};

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

## Aptos Integration (Optional)

When the `aptos` feature is enabled, you get additional utilities for Aptos blockchain interactions:

```rust
use enhanced_ekiden_rust_sdk::aptos::{AptosVault, utils};

// Create Aptos client
let vault = AptosVault::testnet()?;

// Get account balance
let balance = vault.get_apt_balance("0x123...").await?;
println!("APT Balance: {}", balance);

// Fund account from faucet (testnet/devnet)
vault.fund_account("0x123...", Some(1_000_000_000)).await?; // 10 APT

// Create and fund new account
let (private_key, address) = vault.create_and_fund_account().await?;
println!("New account: {} with key: {}", address, utils::private_key_to_hex(&private_key));

// Transfer APT
let tx_hash = vault.transfer_apt(&private_key, "0x456...", 1_000_000).await?;
println!("Transfer transaction: {}", tx_hash);
```

## Error Handling

The SDK provides comprehensive error types:

```rust
use enhanced_ekiden_rust_sdk::{EkidenError, Result};

match client.get_markets(Default::default()).await {
    Ok(markets) => println!("Found {} markets", markets.len()),
    Err(EkidenError::Http(e)) => eprintln!("HTTP error: {}", e),
    Err(EkidenError::Auth(e)) => eprintln!("Auth error: {}", e),
    Err(EkidenError::Api { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Type Safety

All API responses are strongly typed:

```rust
use enhanced_ekiden_rust_sdk::{
    MarketResponse, OrderResponse, PositionResponse,
    OrderSide, OrderType, Pagination
};

// Type-safe pagination
let pagination = Pagination::new(50, 0);
let pagination = Pagination::with_page(1, 25);

// Enum types for order operations
let side = OrderSide::Buy;
let order_type = OrderType::Limit;

// Structured responses
let market: MarketResponse = client.get_market_by_symbol("BTC-USD").await?.unwrap();
println!("Market: {} - Min order: {}", market.symbol, market.min_order_size);
```

## Validation

Built-in validation for addresses, signatures, and other inputs:

```rust
use enhanced_ekiden_rust_sdk::utils::format;

// Validate addresses
format::validate_address("0x123...")?;
let normalized = format::normalize_address("123...")?; // Adds 0x prefix

// Validate public keys and signatures
format::validate_public_key("0x456...")?;
format::validate_signature("0x789...")?;
```

## Configuration Options

```rust
use enhanced_ekiden_rust_sdk::EkidenConfig;
use std::time::Duration;

let config = EkidenConfig::production()?
    .with_timeout(Duration::from_secs(30))
    .with_user_agent("MyTrader/1.0")
    .with_max_retries(3)
    .with_retry_delay(Duration::from_millis(500))
    .with_logging(true);
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

## Support

- [API Documentation](https://docs.ekiden.fi)
- [GitHub Issues](https://github.com/ekidenfi/ekiden-rust-sdk/issues)
- [Discord Community](https://discord.gg/ekiden)
