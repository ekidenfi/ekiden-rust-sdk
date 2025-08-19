use crate::error::{EkidenError, Result};
use crate::types::*;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};
use url::Url;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsReceiver = SplitStream<WsStream>;

/// WebSocket client for Ekiden real-time data
#[derive(Debug)]
pub struct WebSocketClient {
    url: Url,
    sender: Option<Arc<Mutex<WsSink>>>,
    subscriptions: Arc<RwLock<HashMap<String, broadcast::Sender<WsEvent>>>>,
    connection_status: Arc<RwLock<ConnectionStatus>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed(String),
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(url: Url) -> Self {
        Self {
            url,
            sender: None,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            connection_status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
        }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.url);
        *self.connection_status.write().await = ConnectionStatus::Connecting;

        let (ws_stream, _) = connect_async(self.url.as_str())
            .await
            .map_err(|e| EkidenError::WebSocket(format!("Failed to connect: {}", e)))?;
        let (sink, stream) = ws_stream.split();

        self.sender = Some(Arc::new(Mutex::new(sink)));
        *self.connection_status.write().await = ConnectionStatus::Connected;

        // Start the message handling loop
        let subscriptions = self.subscriptions.clone();
        let connection_status = self.connection_status.clone();

        tokio::spawn(async move {
            Self::handle_messages(stream, subscriptions, connection_status).await;
        });

        info!("WebSocket connected successfully");
        Ok(())
    }

    /// Disconnect from the WebSocket server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(sender) = &self.sender {
            let mut sink = sender.lock().await;
            let _ = sink.close().await;
        }

        self.sender = None;
        *self.connection_status.write().await = ConnectionStatus::Disconnected;

        // Clear all subscriptions
        self.subscriptions.write().await.clear();

        info!("WebSocket disconnected");
        Ok(())
    }

    /// Get the current connection status
    pub async fn connection_status(&self) -> ConnectionStatus {
        self.connection_status.read().await.clone()
    }

    /// Check if the client is connected
    pub async fn is_connected(&self) -> bool {
        matches!(
            *self.connection_status.read().await,
            ConnectionStatus::Connected
        )
    }

    /// Send a ping message
    pub async fn ping(&self) -> Result<()> {
        self.send_request(WsRequest::Ping).await
    }

    /// Subscribe to a channel and receive events
    pub async fn subscribe(&self, channel: &str) -> Result<broadcast::Receiver<WsEvent>> {
        let (tx, rx) = broadcast::channel(1000);

        // Store the subscription
        self.subscriptions
            .write()
            .await
            .insert(channel.to_string(), tx);

        // Send subscription request
        self.send_request(WsRequest::Subscribe {
            channel: channel.to_string(),
        })
        .await?;

        info!("Subscribed to channel: {}", channel);
        Ok(rx)
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) -> Result<()> {
        // Remove the subscription
        self.subscriptions.write().await.remove(channel);

        // Send unsubscription request
        self.send_request(WsRequest::Unsubscribe {
            channel: channel.to_string(),
        })
        .await?;

        info!("Unsubscribed from channel: {}", channel);
        Ok(())
    }

    /// Subscribe to orderbook updates for a market
    pub async fn subscribe_orderbook(
        &self,
        market_addr: &str,
    ) -> Result<broadcast::Receiver<WsEvent>> {
        let channel = format!("orderbook/{}", market_addr);
        self.subscribe(&channel).await
    }

    /// Subscribe to trade updates for a market
    pub async fn subscribe_trades(
        &self,
        market_addr: &str,
    ) -> Result<broadcast::Receiver<WsEvent>> {
        let channel = format!("trades/{}", market_addr);
        self.subscribe(&channel).await
    }

    // /// Subscribe to user-specific updates (orders, positions, balances)
    // pub async fn subscribe_user(&self, user_addr: &str) -> Result<broadcast::Receiver<WsEvent>> {
    //     let channel = format!("user/{}", user_addr);
    //     self.subscribe(&channel).await
    // }

    /// Send a WebSocket request
    async fn send_request(&self, request: WsRequest) -> Result<()> {
        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| EkidenError::network("WebSocket not connected"))?;

        let message = serde_json::to_string(&request)?;
        let mut sink = sender.lock().await;
        sink.send(Message::Text(message.into())).await?;

        debug!("Sent WebSocket request: {:?}", request);
        Ok(())
    }

    /// Handle incoming WebSocket messages
    async fn handle_messages(
        mut stream: WsReceiver,
        subscriptions: Arc<RwLock<HashMap<String, broadcast::Sender<WsEvent>>>>,
        connection_status: Arc<RwLock<ConnectionStatus>>,
    ) {
        while let Some(message) = stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = Self::process_message(&text, &subscriptions).await {
                        error!("Error processing WebSocket message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by server");
                    *connection_status.write().await = ConnectionStatus::Disconnected;
                    break;
                }
                Ok(_) => {
                    // Ignore other message types
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    *connection_status.write().await = ConnectionStatus::Failed(e.to_string());
                    break;
                }
            }
        }
    }

    /// Process a WebSocket message
    async fn process_message(
        text: &str,
        subscriptions: &Arc<RwLock<HashMap<String, broadcast::Sender<WsEvent>>>>,
    ) -> Result<()> {
        let response: WsResponse = serde_json::from_str(text)?;

        match response {
            WsResponse::Pong => {
                debug!("Received pong");
            }
            WsResponse::Subscribed { channel } => {
                info!("Successfully subscribed to channel: {}", channel);
            }
            WsResponse::Unsubscribed { channel } => {
                info!("Successfully unsubscribed from channel: {}", channel);
            }
            WsResponse::Event { channel, data } => {
                debug!("Received event for channel {}: {:?}", channel, data);

                // Forward the event to subscribers
                let subscriptions = subscriptions.read().await;
                if let Some(sender) = subscriptions.get(&channel) {
                    if let Err(e) = sender.send(data) {
                        debug!("No active receivers for channel {}: {}", channel, e);
                    }
                }
            }
            WsResponse::Error { message } => {
                error!("WebSocket error: {}", message);
            }
        }

        Ok(())
    }

    /// Get all active subscriptions
    pub async fn active_subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.keys().cloned().collect()
    }

    /// Check if subscribed to a specific channel
    pub async fn is_subscribed(&self, channel: &str) -> bool {
        self.subscriptions.read().await.contains_key(channel)
    }
}

/// Builder for WebSocket client configuration
#[derive(Debug)]
pub struct WebSocketClientBuilder {
    url: Option<Url>,
}

impl WebSocketClientBuilder {
    pub fn new() -> Self {
        Self { url: None }
    }

    pub fn url<U: Into<Url>>(mut self, url: U) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn build(self) -> Result<WebSocketClient> {
        let url = self
            .url
            .ok_or_else(|| EkidenError::config("WebSocket URL is required"))?;
        Ok(WebSocketClient::new(url))
    }
}

impl Default for WebSocketClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for creating market-specific channels
pub mod channels {
    /// Create an orderbook channel for a market
    pub fn orderbook(market_addr: &str) -> String {
        format!("orderbook/{}", market_addr)
    }

    /// Create a trades channel for a market
    pub fn trades(market_addr: &str) -> String {
        format!("trades/{}", market_addr)
    }

    /// Create a user channel for user-specific updates
    pub fn user(user_addr: &str) -> String {
        format!("user/{}", user_addr)
    }

    /// Create a candles channel for a market and interval
    pub fn candles(market_addr: &str, interval: &str) -> String {
        format!("candles/{}/{}", market_addr, interval)
    }
}

/// Event stream wrapper for easier handling
pub struct EventStream {
    receiver: broadcast::Receiver<WsEvent>,
    channel: String,
}

impl EventStream {
    pub fn new(receiver: broadcast::Receiver<WsEvent>, channel: String) -> Self {
        Self { receiver, channel }
    }

    /// Get the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }

    /// Receive the next event
    pub async fn recv(&mut self) -> Result<WsEvent> {
        self.receiver.recv().await.map_err(|e| match e {
            broadcast::error::RecvError::Closed => EkidenError::ConnectionClosed,
            broadcast::error::RecvError::Lagged(_) => EkidenError::general("Event stream lagged"),
        })
    }

    /// Try to receive an event without blocking
    pub fn try_recv(&mut self) -> Result<WsEvent> {
        self.receiver.try_recv().map_err(|e| match e {
            broadcast::error::TryRecvError::Empty => EkidenError::general("No events available"),
            broadcast::error::TryRecvError::Closed => EkidenError::ConnectionClosed,
            broadcast::error::TryRecvError::Lagged(_) => {
                EkidenError::general("Event stream lagged")
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let url = Url::parse("ws://localhost:3010/ws").unwrap();
        let client = WebSocketClient::new(url);

        assert_eq!(
            client.connection_status().await,
            ConnectionStatus::Disconnected
        );
        assert!(!client.is_connected().await);
    }

    #[test]
    fn test_channel_helpers() {
        assert_eq!(channels::orderbook("0x123"), "orderbook/0x123");
        assert_eq!(channels::trades("0x456"), "trades/0x456");
        assert_eq!(channels::user("0x789"), "user/0x789");
        assert_eq!(channels::candles("0x123", "1m"), "candles/0x123/1m");
    }

    #[test]
    fn test_websocket_builder() {
        let url = Url::parse("ws://localhost:3010/ws").unwrap();
        let client = WebSocketClientBuilder::new()
            .url(url.clone())
            .build()
            .unwrap();

        assert_eq!(client.url, url);
    }
}
