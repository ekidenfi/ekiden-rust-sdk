use crate::auth::Auth;
use crate::config::EkidenConfig;
use crate::error::{EkidenError, Result};
use crate::types::*;
use crate::ws::WebSocketClient;
use aptos_crypto::{
    ed25519::Ed25519PrivateKey, ed25519::Signature,
    ValidCryptoMaterialStringExt,
};
use ekiden_core::sequencer::SigningIntent;
use ekiden_core::{
    sequencer::{ActionPayload, IntentSignatureBody},
};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
/// Main Ekiden client for interacting with the API and WebSocket
#[derive(Debug, Clone)]
pub struct EkidenClient {
    config: EkidenConfig,
    http_client: Client,
    auth: Arc<RwLock<Auth>>,
    funding_auth: Arc<RwLock<Auth>>,
    trading_auth: Arc<RwLock<Auth>>,
    ws_client: Option<Arc<RwLock<WebSocketClient>>>,
}

impl EkidenClient {
    /// Create a new Ekiden client with the given configuration
    pub fn new(config: EkidenConfig) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .build()?;

        let ws_client = Some(Arc::new(RwLock::new(WebSocketClient::new(
            config.websocket_url().clone(),
        ))));

        Ok(Self {
            config,
            http_client,
            auth: Arc::new(RwLock::new(Auth::new())),
            funding_auth: Arc::new(RwLock::new(Auth::new())),
            trading_auth: Arc::new(RwLock::new(Auth::new())),
            ws_client,
        })
    }

    /// Create a client with default configuration
    pub fn default_config() -> Result<Self> {
        Self::new(EkidenConfig::default())
    }

    /// Create a client for production environment
    pub fn production() -> Result<Self> {
        Self::new(EkidenConfig::production()?)
    }

    /// Create a client for staging environment
    pub fn staging() -> Result<Self> {
        Self::new(EkidenConfig::staging()?)
    }

    /// Create a client for local development
    pub fn local() -> Result<Self> {
        Self::new(EkidenConfig::local()?)
    }

    /// Set the private key for signing operations
    pub async fn set_private_key(&self, private_key: &str) -> Result<()> {
        let mut auth = self.auth.write().await;
        *auth = auth.clone().with_private_key(private_key)?;
        Ok(())
    }

    pub async fn set_funding_private_key(&self, private_key: &str) -> Result<()> {
        let mut auth = self.funding_auth.write().await;
        *auth = auth.clone().with_private_key(private_key)?;
        Ok(())
    }

    pub async fn set_trading_private_key(&self, private_key: &str) -> Result<()> {
        let mut auth = self.trading_auth.write().await;
        *auth = auth.clone().with_private_key(private_key)?;
        Ok(())
    }

    /// Set the authentication token
    pub async fn set_token(&self, token: &str) {
        let mut auth = self.auth.write().await;
        auth.set_token(token);
    }

    /// Get the current authentication token
    pub async fn token(&self) -> Option<String> {
        self.auth.read().await.token().map(|s| s.to_string())
    }

    /// Get the current authentication token
    pub async fn funding_token(&self) -> Option<String> {
        self.funding_auth.read().await.token().map(|s| s.to_string())
    }

    /// Get the current authentication token
    pub async fn trading_token(&self) -> Option<String> {
        self.trading_auth.read().await.token().map(|s| s.to_string())
    }

    /// Get the public key if available
    pub async fn public_key(&self) -> Option<String> {
        self.auth.read().await.public_key()
    }

    /// Check if the client is authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.auth.read().await.is_authenticated()
    }

    // ===== Authentication =====

    /// Authenticate with the API using the configured private key
    pub async fn authorize(&self) -> Result<AuthorizeResponse> {
        let auth_params = {
            let auth = self.auth.read().await;
            auth.generate_authorize_params()?
        };

        let response: AuthorizeResponse = self
            .request("authorize", RequestConfig::post(&auth_params)?)
            .await?;

        println!("Authorization successful: {}", response.token);
        // Store the token
        {
            let mut auth = self.auth.write().await;
            auth.process_authorize_response(response.clone());
        }

        info!("Successfully authenticated with Ekiden API");
        Ok(response)
    }

    pub async fn authorize_funding(&self) -> Result<AuthorizeResponse> {
        let auth_params = {
            let auth = self.funding_auth.read().await;
            auth.generate_authorize_params()?
        };

        let response: AuthorizeResponse = self
            .request("authorize", RequestConfig::post(&auth_params)?)
            .await?;

        println!("Authorization successful: {}", response.token);
        // Store the token
        {
            let mut auth = self.funding_auth.write().await;
            auth.process_authorize_response(response.clone());
        }

        info!("Successfully authenticated with Ekiden API");
        Ok(response)
    }

    pub async fn authorize_trading(&self) -> Result<AuthorizeResponse> {
        let auth_params = {
            let auth = self.trading_auth.read().await;
            auth.generate_authorize_params()?
        };

        let response: AuthorizeResponse = self
            .request("authorize", RequestConfig::post(&auth_params)?)
            .await?;

        println!("Authorization successful: {}", response.token);
        // Store the token
        {
            let mut auth = self.trading_auth.write().await;
            auth.process_authorize_response(response.clone());
        }

        info!("Successfully authenticated with Ekiden API");
        Ok(response)
    }

    // ===== Market Endpoints =====

    /// Get market information
    pub async fn get_markets(&self, params: ListMarketsParams) -> Result<Vec<MarketResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("market_info", config).await
    }

    /// Get a specific market by address
    pub async fn get_market_by_address(&self, market_addr: &str) -> Result<Option<MarketResponse>> {
        let params = ListMarketsParams {
            market_addr: Some(market_addr.to_string()),
            symbol: None,
            pagination: Pagination::default(),
        };
        let markets = self.get_markets(params).await?;
        Ok(markets.into_iter().next())
    }

    /// Get a specific market by symbol
    pub async fn get_market_by_symbol(&self, symbol: &str) -> Result<Option<MarketResponse>> {
        let params = ListMarketsParams {
            market_addr: None,
            symbol: Some(symbol.to_string()),
            pagination: Pagination::default(),
        };
        let markets = self.get_markets(params).await?;
        Ok(markets.into_iter().next())
    }

    // ===== Order Endpoints =====

    /// Get orders for a market
    pub async fn get_orders(&self, params: ListOrdersParams) -> Result<Vec<OrderResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("orders", config).await
    }

    /// Get orders for a specific market and side
    pub async fn get_orders_by_side(
        &self,
        market_addr: &str,
        side: OrderSide,
        pagination: Option<Pagination>,
    ) -> Result<Vec<OrderResponse>> {
        let params = ListOrdersParams {
            market_addr: market_addr.to_string(),
            side: Some(match side {
                OrderSide::Buy => "buy".to_string(),
                OrderSide::Sell => "sell".to_string(),
            }),
            pagination: pagination.unwrap_or_default(),
        };
        self.get_orders(params).await
    }

    // ===== Fill Endpoints =====

    /// Get fills (trades) for a market
    pub async fn get_fills(&self, params: ListFillsParams) -> Result<Vec<FillResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("fills", config).await
    }

    /// Get recent fills for a market
    pub async fn get_recent_fills(
        &self,
        market_addr: &str,
        limit: Option<u32>,
    ) -> Result<Vec<FillResponse>> {
        let params = ListFillsParams {
            market_addr: market_addr.to_string(),
            pagination: Pagination {
                limit,
                offset: Some(0),
                page: None,
                page_size: None,
            },
        };
        self.get_fills(params).await
    }

    // ===== User Endpoints =====

    /// Get user vaults
    pub async fn get_user_vaults(&self, params: ListVaultsParams) -> Result<Vec<VaultResponse>> {
        let config = RequestConfig::get()
            .with_query(params.to_query_params())
            .with_auth(self.token().await.unwrap_or_default());
        self.request("user/vaults", config).await
    }

    /// Get all user vaults
    pub async fn get_all_user_vaults(&self) -> Result<Vec<VaultResponse>> {
        let params = ListVaultsParams {
            pagination: Pagination::default(),
        };
        self.get_user_vaults(params).await
    }

    /// Get user positions
    pub async fn get_user_positions(
        &self,
        params: ListPositionsParams,
    ) -> Result<Vec<PositionResponse>> {
        let config = RequestConfig::get()
            .with_query(params.to_query_params())
            .with_auth(self.token().await.unwrap_or_default());
        self.request("user/positions", config).await
    }

    /// Get user positions for a specific market
    pub async fn get_user_positions_by_market(
        &self,
        market_addr: &str,
    ) -> Result<Vec<PositionResponse>> {
        let params = ListPositionsParams {
            market_addr: Some(market_addr.to_string()),
            pagination: Pagination::default(),
        };
        self.get_user_positions(params).await
    }

    /// Get all user positions
    pub async fn get_all_user_positions(&self) -> Result<Vec<PositionResponse>> {
        let params = ListPositionsParams {
            market_addr: None,
            pagination: Pagination::default(),
        };
        self.get_user_positions(params).await
    }

    /// Get user leverage for a market
    pub async fn get_user_leverage(&self, market_addr: &str) -> Result<LeverageResponse> {
        let params = GetUserLeverageParams {
            market_addr: market_addr.to_string(),
        };
        let config = RequestConfig::get()
            .with_query(params.to_query_params())
            .with_auth(self.token().await.unwrap_or_default());
        self.request("user/leverage", config).await
    }

    /// Set user leverage for a market
    pub async fn set_user_leverage(
        &self,
        market_addr: &str,
        leverage: u64,
    ) -> Result<LeverageResponse> {
        let params = SetUserLeverageParams {
            market_addr: market_addr.to_string(),
            leverage,
        };
        let config =
            RequestConfig::post(&params)?.with_auth(self.token().await.unwrap_or_default());
        self.request("user/leverage", config).await
    }

    /// Get user portfolio
    pub async fn get_user_portfolio(&self) -> Result<PortfolioResponse> {
        let config = RequestConfig::get().with_auth(self.token().await.unwrap_or_default());
        println!("Fetching user portfolio... {:?}", config);
        self.request("user/portfolio", config).await
    }

    pub fn sign_intent(
        &self,
        private_key_str: &str,
        payload: &ActionPayload,
        nonce: u64,
    ) -> Result<Signature> {
        let key_pair = Ed25519PrivateKey::from_encoded_string(private_key_str).unwrap();
        let signature = key_pair
            .sign_intent(IntentSignatureBody {
                payload: payload.clone(),
                nonce,
            })
            .map_err(|e| EkidenError::auth(format!("Failed to sign intent: {}", e)))?;
        Ok(signature)
    }

    /// Send an intent (execute actions)
    pub async fn send_intent(&self, params: SendIntentParams) -> Result<SendIntentResponse> {
        let config =
            RequestConfig::post(&params)?.with_auth(self.trading_token().await.unwrap_or_default());
        self.request("user/intent/commit", config).await
    }

    // ===== Deposit/Withdrawal Endpoints =====

    /// Get deposits
    pub async fn get_deposits(&self, params: ListDepositsParams) -> Result<Vec<DepositResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("deposits", config).await
    }

    /// Get user deposits
    pub async fn get_user_deposits(&self, user_addr: &str) -> Result<Vec<DepositResponse>> {
        let params = ListDepositsParams {
            user_addr: Some(user_addr.to_string()),
            vault_addr: None,
            asset_addr: None,
            start_version: None,
            end_version: None,
            pagination: Pagination::default(),
        };
        self.get_deposits(params).await
    }

    /// Get withdrawals
    pub async fn get_withdrawals(
        &self,
        params: ListWithdrawsParams,
    ) -> Result<Vec<WithdrawResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("withdraws", config).await
    }

    /// Get user withdrawals
    pub async fn get_user_withdrawals(&self, user_addr: &str) -> Result<Vec<WithdrawResponse>> {
        let params = ListWithdrawsParams {
            user_addr: Some(user_addr.to_string()),
            vault_addr: None,
            asset_addr: None,
            start_version: None,
            end_version: None,
            pagination: Pagination::default(),
        };
        self.get_withdrawals(params).await
    }

    // ===== Candle Endpoints =====

    /// Get candlestick data
    pub async fn get_candles(&self, params: ListCandlesParams) -> Result<Vec<CandleResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("candles", config).await
    }

    /// Get recent candles for a market
    pub async fn get_recent_candles(
        &self,
        market_addr: &str,
        interval: &str,
        limit: Option<u32>,
    ) -> Result<Vec<CandleResponse>> {
        let params = ListCandlesParams {
            market_addr: market_addr.to_string(),
            interval: interval.to_string(),
            start_time: None,
            end_time: None,
            pagination: Pagination {
                limit,
                offset: Some(0),
                page: None,
                page_size: None,
            },
        };
        self.get_candles(params).await
    }

    // ===== Funding Rate Endpoints =====

    /// Get funding rates
    pub async fn get_funding_rates(
        &self,
        params: ListFundingRatesParams,
    ) -> Result<Vec<FundingRateResponse>> {
        let config = RequestConfig::get().with_query(params.to_query_params());
        self.request("funding_rate", config).await
    }

    /// Get current funding rate for a market
    pub async fn get_current_funding_rate(
        &self,
        market_addr: &str,
    ) -> Result<Option<FundingRateResponse>> {
        let params = ListFundingRatesParams {
            market_addr: market_addr.to_string(),
            start_time: None,
            end_time: None,
            pagination: Pagination {
                limit: Some(1),
                offset: Some(0),
                page: None,
                page_size: None,
            },
        };
        let rates = self.get_funding_rates(params).await?;
        Ok(rates.into_iter().next())
    }

    // ===== WebSocket Methods =====

    /// Connect to WebSocket
    pub async fn connect_websocket(&self) -> Result<()> {
        if let Some(ws_client) = &self.ws_client {
            let mut client = ws_client.write().await;
            client.connect().await?;
            info!("WebSocket connected");
        }
        Ok(())
    }

    /// Disconnect from WebSocket
    pub async fn disconnect_websocket(&self) -> Result<()> {
        if let Some(ws_client) = &self.ws_client {
            let mut client = ws_client.write().await;
            client.disconnect().await?;
            info!("WebSocket disconnected");
        }
        Ok(())
    }

    /// Check if WebSocket is connected
    pub async fn is_websocket_connected(&self) -> bool {
        if let Some(ws_client) = &self.ws_client {
            let client = ws_client.read().await;
            client.is_connected().await
        } else {
            false
        }
    }

    /// Subscribe to orderbook updates
    pub async fn subscribe_orderbook(
        &self,
        market_addr: &str,
    ) -> Result<tokio::sync::broadcast::Receiver<WsEvent>> {
        if let Some(ws_client) = &self.ws_client {
            let client = ws_client.read().await;
            client.subscribe_orderbook(market_addr).await
        } else {
            Err(EkidenError::config("WebSocket client not available"))
        }
    }

    /// Subscribe to trade updates
    pub async fn subscribe_trades(
        &self,
        market_addr: &str,
    ) -> Result<tokio::sync::broadcast::Receiver<WsEvent>> {
        if let Some(ws_client) = &self.ws_client {
            let client = ws_client.read().await;
            client.subscribe_trades(market_addr).await
        } else {
            Err(EkidenError::config("WebSocket client not available"))
        }
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, channel: &str) -> Result<()> {
        if let Some(ws_client) = &self.ws_client {
            let client = ws_client.read().await;
            client.unsubscribe(channel).await
        } else {
            Err(EkidenError::config("WebSocket client not available"))
        }
    }

    // ===== Private Helper Methods =====

    /// Make an HTTP request to the API
    async fn request<T>(&self, path: &str, config: RequestConfig) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = self.config.api_url(path);
        let auth_already_present = config.has_auth_header();
        let mut request = self.http_client.request(config.method, &url);

        // Add query parameters
        if let Some(query) = &config.query {
            request = request.query(query);
        }

        // Add headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        // Add authentication headers if required
        if config.auth_required && !auth_already_present {
            let auth = self.auth.read().await;
            auth.ensure_authenticated()?;
            let auth_headers = auth.auth_headers();
            for (key, value) in auth_headers {
                request = request.header(key, value);
            }
        }

        // Add body for POST/PUT requests
        if let Some(body) = &config.body {
            request = request.json(body);
        }

        // Execute the request
        let response = request.send().await?;
        self.handle_response(response).await
    }

    /// Handle HTTP response and convert to the desired type
    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();

        if status.is_success() {
            let text = response.text().await?;
            debug!("API response: {}", text);
            serde_json::from_str(&text).map_err(EkidenError::Json)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("API error {}: {}", status, error_text);
            Err(EkidenError::api(status.as_u16(), error_text))
        }
    }
}

/// Builder for creating configured Ekiden clients
#[derive(Debug)]
pub struct EkidenClientBuilder {
    config: EkidenConfig,
    private_key: Option<String>,
    funding_private_key: Option<String>,
    trading_private_key: Option<String>,
    token: Option<String>,
}

impl EkidenClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self {
            config: EkidenConfig::default(),
            private_key: None,
            funding_private_key: None,
            trading_private_key: None,
            token: None,
        }
    }

    /// Set the configuration
    pub fn config(mut self, config: EkidenConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the base URL
    pub fn base_url<S: AsRef<str>>(mut self, base_url: S) -> Result<Self> {
        self.config = EkidenConfig::new(base_url)?;
        Ok(self)
    }

    /// Use production environment
    pub fn production(mut self) -> Result<Self> {
        self.config = EkidenConfig::production()?;
        Ok(self)
    }

    /// Use staging environment
    pub fn staging(mut self) -> Result<Self> {
        self.config = EkidenConfig::staging()?;
        Ok(self)
    }

    /// Use local development environment
    pub fn local(mut self) -> Result<Self> {
        self.config = EkidenConfig::local()?;
        Ok(self)
    }

    /// Set the private key
    pub fn private_key<S: Into<String>>(mut self, private_key: S) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    /// Set the trading account private key
    pub fn trading_private_key<S: Into<String>>(mut self, private_key: S) -> Self {
        self.trading_private_key = Some(private_key.into());
        self
    }

    /// Set the funding account private key
    pub fn funding_private_key<S: Into<String>>(mut self, private_key: S) -> Self {
        self.funding_private_key = Some(private_key.into());
        self
    }

    /// Set the authentication token
    pub fn token<S: Into<String>>(mut self, token: S) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config = self.config.with_timeout(timeout);
        self
    }

    /// Set user agent
    pub fn user_agent<S: Into<String>>(mut self, user_agent: S) -> Self {
        self.config = self.config.with_user_agent(user_agent);
        self
    }

    /// Enable logging
    pub fn with_logging(mut self, enable: bool) -> Self {
        self.config = self.config.with_logging(enable);
        self
    }

    /// Build the client
    pub async fn build(self) -> Result<EkidenClient> {
        let client = EkidenClient::new(self.config)?;

        // Set private key if provided
        if let Some(private_key) = self.private_key {
            client.set_private_key(&private_key).await?;
        }

        // Set private key if provided
        if let Some(private_key) = self.funding_private_key {
            client.set_funding_private_key(&private_key).await?;
        }

        // Set private key if provided
        if let Some(private_key) = self.trading_private_key {
            client.set_trading_private_key(&private_key).await?;
        }

        // Set token if provided
        if let Some(token) = self.token {
            client.set_token(&token).await;
        }

        Ok(client)
    }

    /// Build and authenticate the client
    pub async fn build_and_auth(self) -> Result<EkidenClient> {
        let client = self.build().await?;
        client.authorize().await?;
        client.authorize_funding().await?;
        client.authorize_trading().await?;
        Ok(client)
    }
}

impl Default for EkidenClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = EkidenClient::default_config().unwrap();
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_client_builder() {
        let client = EkidenClientBuilder::new()
            .local()
            .unwrap()
            .timeout(Duration::from_secs(10))
            .build()
            .await
            .unwrap();

        assert!(!client.is_authenticated().await);
    }
}
