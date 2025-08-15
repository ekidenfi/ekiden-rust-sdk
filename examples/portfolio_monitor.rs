use ekiden_rust_sdk::{EkidenClient, KeyPair, PortfolioResponse, WsEvent};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("📊 Starting Portfolio Monitor Example");

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
    let user_addr = key_pair.public_key(); // Use public key as user identifier

    // Setup client
    let client = EkidenClient::local()?;
    client.set_private_key(&key_pair.private_key()).await?;

    // Try to authenticate
    match client.authorize().await {
        Ok(_) => println!("✅ Authenticated successfully"),
        Err(e) => {
            println!("❌ Authentication failed: {}", e);
            println!("This example requires a running Ekiden Gateway API");
            return Ok(());
        }
    }

    // Connect WebSocket for real-time updates
    if let Err(e) = client.connect_websocket().await {
        println!("⚠️  WebSocket connection failed: {}", e);
        println!("Continuing with polling mode only...");
    } else {
        println!("🔌 WebSocket connected for real-time updates");
    }

    // Create portfolio monitor
    let mut monitor = PortfolioMonitor::new(client, user_addr);

    // Start monitoring
    monitor.start_monitoring().await?;

    println!("🎉 Portfolio monitoring completed!");
    Ok(())
}

struct PortfolioMonitor {
    client: EkidenClient,
    user_addr: String,
    last_portfolio: Option<PortfolioResponse>,
    _position_cache: HashMap<String, f64>, // market_addr -> unrealized_pnl
}

impl PortfolioMonitor {
    fn new(client: EkidenClient, user_addr: String) -> Self {
        Self {
            client,
            user_addr,
            last_portfolio: None,
            _position_cache: HashMap::new(),
        }
    }

    async fn start_monitoring(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🚀 Starting portfolio monitoring...");

        // Get initial portfolio snapshot
        self.update_portfolio_snapshot().await?;

        // Set up WebSocket listener for real-time updates
        let ws_handle = self.setup_realtime_updates().await;

        // Set up periodic portfolio updates
        let polling_handle = self.setup_periodic_polling().await;

        // Run both concurrently for a demo period
        println!("⏳ Monitoring for 30 seconds...");
        tokio::time::sleep(Duration::from_secs(30)).await;

        // Cleanup
        if let Some(handle) = ws_handle {
            handle.abort();
        }
        polling_handle.abort();

        Ok(())
    }

    async fn update_portfolio_snapshot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self.client.get_user_portfolio().await {
            Ok(portfolio) => {
                self.display_portfolio_summary(&portfolio);
                self.detect_portfolio_changes(&portfolio);
                self.last_portfolio = Some(portfolio);
            }
            Err(e) => {
                println!("❌ Failed to fetch portfolio: {}", e);
            }
        }
        Ok(())
    }

    fn display_portfolio_summary(&self, portfolio: &PortfolioResponse) {
        println!("\n💼 Portfolio Summary");
        println!("═══════════════════════════════════");
        println!(
            "🏦 Total Value: ${:.2}",
            portfolio.summary.total_value as f64 / 1e6
        );
        println!(
            "💰 Available Balance: ${:.2}",
            portfolio.summary.available_balance as f64 / 1e6
        );
        println!(
            "🔒 Locked Balance: ${:.2}",
            portfolio.summary.locked_balance as f64 / 1e6
        );
        println!(
            "📈 Unrealized PnL: ${:.2}",
            portfolio.summary.unrealized_pnl as f64 / 1e6
        );
        println!(
            "📊 Margin Used: ${:.2}",
            portfolio.summary.margin_used as f64 / 1e6
        );
        println!(
            "🆓 Margin Available: ${:.2}",
            portfolio.summary.margin_available as f64 / 1e6
        );

        if !portfolio.positions.is_empty() {
            println!("\n📍 Active Positions ({}):", portfolio.positions.len());
            println!("─────────────────────────────────");
            for (i, position) in portfolio.positions.iter().enumerate() {
                let pnl_color = if position.unrealized_pnl >= 0 {
                    "🟢"
                } else {
                    "🔴"
                };
                println!(
                    "{}. {} {} - Size: {}, Entry: {}, Mark: {}, PnL: {} ${:.2}",
                    i + 1,
                    position.symbol,
                    position.side.to_uppercase(),
                    position.size,
                    position.entry_price,
                    position.mark_price,
                    pnl_color,
                    position.unrealized_pnl as f64 / 1e6
                );
            }
        }

        if !portfolio.vaults.is_empty() {
            println!("\n🏛️  Vault Balances ({}):", portfolio.vaults.len());
            println!("─────────────────────────────────");
            for (i, vault) in portfolio.vaults.iter().enumerate() {
                println!(
                    "{}. {} - Balance: {}, Available: {}, USD Value: ${:.2}",
                    i + 1,
                    vault.symbol,
                    vault.balance,
                    vault.available_balance,
                    vault.usd_value as f64 / 1e6
                );
            }
        }
    }

    fn detect_portfolio_changes(&mut self, new_portfolio: &PortfolioResponse) {
        if let Some(ref old_portfolio) = self.last_portfolio {
            // Check for value changes
            let value_change =
                new_portfolio.summary.total_value as i64 - old_portfolio.summary.total_value as i64;
            if value_change != 0 {
                let change_color = if value_change > 0 { "🟢 +" } else { "🔴 " };
                println!(
                    "\n💹 Portfolio Value Change: {}{:.2}",
                    change_color,
                    value_change as f64 / 1e6
                );
            }

            // Check for PnL changes
            let pnl_change =
                new_portfolio.summary.unrealized_pnl - old_portfolio.summary.unrealized_pnl;
            if pnl_change != 0 {
                let pnl_color = if pnl_change > 0 { "🟢 +" } else { "🔴 " };
                println!("📊 PnL Change: {}{:.2}", pnl_color, pnl_change as f64 / 1e6);
            }

            // Check for new or closed positions
            let old_positions: HashMap<&str, &_> = old_portfolio
                .positions
                .iter()
                .map(|p| (p.market_addr.as_str(), p))
                .collect();

            for position in &new_portfolio.positions {
                if !old_positions.contains_key(position.market_addr.as_str()) {
                    println!(
                        "🆕 New Position: {} {} - Size: {}",
                        position.symbol,
                        position.side.to_uppercase(),
                        position.size
                    );
                }
            }

            let new_positions: HashMap<&str, &_> = new_portfolio
                .positions
                .iter()
                .map(|p| (p.market_addr.as_str(), p))
                .collect();

            for (market_addr, old_position) in old_positions {
                if !new_positions.contains_key(market_addr) {
                    println!(
                        "❌ Closed Position: {} {} - Size: {}",
                        old_position.symbol,
                        old_position.side.to_uppercase(),
                        old_position.size
                    );
                }
            }
        }
    }

    async fn setup_realtime_updates(&self) -> Option<tokio::task::JoinHandle<()>> {
        if !self.client.is_websocket_connected().await {
            return None;
        }

        match self.client.subscribe_user(&self.user_addr).await {
            Ok(mut user_rx) => {
                println!("🔔 Subscribed to real-time user updates");

                let _client = self.client.clone();
                Some(tokio::spawn(async move {
                    while let Ok(event) = user_rx.recv().await {
                        match event {
                            WsEvent::OrderUpdate { order } => {
                                println!(
                                    "\n📋 Order Update: {} - {} {} {} at {}",
                                    order.sid,
                                    order.status.to_uppercase(),
                                    order.side.to_uppercase(),
                                    order.size,
                                    order.price
                                );
                            }
                            WsEvent::PositionUpdate { position } => {
                                let pnl_color = if position.unrealized_pnl >= 0 {
                                    "🟢"
                                } else {
                                    "🔴"
                                };
                                println!(
                                    "\n📍 Position Update: {} {} - Size: {}, PnL: {} ${:.2}",
                                    position.market_addr,
                                    position.side.to_uppercase(),
                                    position.size,
                                    pnl_color,
                                    position.unrealized_pnl as f64 / 1e6
                                );
                            }
                            WsEvent::BalanceUpdate { vault } => {
                                println!(
                                    "\n💳 Balance Update: {} - Available: {}, Locked: {}",
                                    vault.vault_addr, vault.available_balance, vault.locked_balance
                                );
                            }
                            _ => {}
                        }
                    }
                }))
            }
            Err(e) => {
                println!("⚠️  Failed to subscribe to user updates: {}", e);
                None
            }
        }
    }

    async fn setup_periodic_polling(&mut self) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let _user_addr = self.user_addr.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Fetch latest portfolio data
                match client.get_user_portfolio().await {
                    Ok(portfolio) => {
                        // For simplicity, just log that we updated
                        // In a real monitor, you'd want to store and compare state
                        println!(
                            "🔄 Portfolio refreshed - Total Value: ${:.2}",
                            portfolio.summary.total_value as f64 / 1e6
                        );
                    }
                    Err(_) => {
                        // Silently continue on errors during periodic updates
                    }
                }
            }
        })
    }
}
