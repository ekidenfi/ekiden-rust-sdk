use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ===== Common Pagination =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: Some(100),
            offset: Some(0),
            page: None,
            page_size: None,
        }
    }
}

impl Pagination {
    pub fn new(limit: u32, offset: u32) -> Self {
        Self {
            limit: Some(limit),
            offset: Some(offset),
            page: None,
            page_size: None,
        }
    }

    pub fn with_page(page: u32, page_size: u32) -> Self {
        Self {
            limit: None,
            offset: None,
            page: Some(page),
            page_size: Some(page_size),
        }
    }
}

// ===== Authentication Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeParams {
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    pub token: String,
}

// ===== Market Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub symbol: String,
    pub base_addr: String,
    pub base_decimals: u8,
    pub quote_addr: String,
    pub quote_decimals: u8,
    pub min_order_size: u64,
    pub max_leverage: u32,
    pub initial_margin_ratio: f64,
    pub maintenance_margin_ratio: f64,
    pub mark_price: u64,
    pub oracle_price: u64,
    pub open_interest: u64,
    pub funding_index: u64,
    pub funding_epoch: u64,
    pub root: String,
    pub epoch: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMarketsParams {
    pub market_addr: Option<String>,
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

impl Default for ListMarketsParams {
    fn default() -> Self {
        Self {
            market_addr: None,
            symbol: None,
            pagination: Pagination::default(),
        }
    }
}

// ===== Order Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub sid: String,
    pub side: String,
    pub size: u64,
    pub price: u64,
    pub leverage: u64,
    #[serde(rename = "type")]
    pub order_type: String,
    pub status: String,
    pub user_addr: String,
    pub market_addr: String,
    pub seq: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersParams {
    pub market_addr: String,
    pub side: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub status: String,
}

// ===== Fill Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillResponse {
    pub sid: String,
    pub price: u64,
    pub size: u64,
    pub side: String,
    pub taker_addr: String,
    pub maker_addr: String,
    pub market_addr: String,
    pub seq: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFillsParams {
    pub market_addr: String,
    #[serde(flatten)]
    pub pagination: Pagination,
}

// ===== User Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultResponse {
    pub vault_addr: String,
    pub user_addr: String,
    pub asset_addr: String,
    pub balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVaultsParams {
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResponse {
    pub market_addr: String,
    pub user_addr: String,
    pub side: String,
    pub size: u64,
    pub entry_price: u64,
    pub mark_price: u64,
    pub unrealized_pnl: i64,
    pub margin: u64,
    pub leverage: u64,
    pub liquidation_price: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPositionsParams {
    pub market_addr: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeverageResponse {
    pub market_addr: String,
    pub user_addr: String,
    pub leverage: u64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserLeverageParams {
    pub market_addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetUserLeverageParams {
    pub market_addr: String,
    pub leverage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioResponse {
    pub summary: PortfolioSummary,
    pub positions: Vec<PortfolioPosition>,
    pub vaults: Vec<PortfolioVault>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value: u64,
    pub available_balance: u64,
    pub locked_balance: u64,
    pub unrealized_pnl: i64,
    pub margin_used: u64,
    pub margin_available: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPosition {
    pub market_addr: String,
    pub symbol: String,
    pub side: String,
    pub size: u64,
    pub entry_price: u64,
    pub mark_price: u64,
    pub unrealized_pnl: i64,
    pub margin: u64,
    pub leverage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioVault {
    pub vault_addr: String,
    pub asset_addr: String,
    pub symbol: String,
    pub balance: u64,
    pub locked_balance: u64,
    pub available_balance: u64,
    pub usd_value: u64,
}

// ===== Intent Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendIntentParams {
    pub actions: Vec<ActionPayload>,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendIntentResponse {
    pub seq: u64,
    pub status: String,
    pub outputs: Vec<IntentOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPayload {
    #[serde(rename = "type")]
    pub action_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentOutput {
    pub action_type: String,
    pub result: serde_json::Value,
}

// ===== Deposit/Withdrawal Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    pub user_addr: String,
    pub vault_addr: String,
    pub asset_addr: String,
    pub amount: u64,
    pub tx_hash: String,
    pub version: u64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDepositsParams {
    pub user_addr: Option<String>,
    pub vault_addr: Option<String>,
    pub asset_addr: Option<String>,
    pub start_version: Option<u64>,
    pub end_version: Option<u64>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    pub user_addr: String,
    pub vault_addr: String,
    pub asset_addr: String,
    pub amount: u64,
    pub tx_hash: String,
    pub version: u64,
    pub timestamp: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWithdrawsParams {
    pub user_addr: Option<String>,
    pub vault_addr: Option<String>,
    pub asset_addr: Option<String>,
    pub start_version: Option<u64>,
    pub end_version: Option<u64>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

// ===== Candle Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleResponse {
    pub market_addr: String,
    pub timestamp: u64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub volume: u64,
    pub interval: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCandlesParams {
    pub market_addr: String,
    pub interval: String, // "1m", "5m", "15m", "1h", "4h", "1d"
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

// ===== Funding Rate Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRateResponse {
    pub market_addr: String,
    pub funding_rate: f64,
    pub funding_index: u64,
    pub funding_epoch: u64,
    pub next_funding_time: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFundingRatesParams {
    pub market_addr: String,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    #[serde(flatten)]
    pub pagination: Pagination,
}

// ===== WebSocket Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsRequest {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "subscribe")]
    Subscribe { channel: String },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { channel: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsResponse {
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "subscribed")]
    Subscribed { channel: String },
    #[serde(rename = "unsubscribed")]
    Unsubscribed { channel: String },
    #[serde(rename = "event")]
    Event { channel: String, data: WsEvent },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsEvent {
    #[serde(rename = "orderbook_snapshot")]
    OrderbookSnapshot {
        market_addr: String,
        bids: Vec<OrderbookLevel>,
        asks: Vec<OrderbookLevel>,
        timestamp: u64,
    },
    #[serde(rename = "orderbook_update")]
    OrderbookUpdate {
        market_addr: String,
        bids: Vec<OrderbookLevel>,
        asks: Vec<OrderbookLevel>,
        timestamp: u64,
    },
    #[serde(rename = "trade")]
    Trade {
        market_addr: String,
        price: u64,
        size: u64,
        side: String,
        timestamp: u64,
    },
    #[serde(rename = "order_update")]
    OrderUpdate { order: OrderResponse },
    #[serde(rename = "position_update")]
    PositionUpdate { position: PositionResponse },
    #[serde(rename = "balance_update")]
    BalanceUpdate { vault: VaultResponse },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookLevel {
    pub price: u64,
    pub size: u64,
}

// ===== Request Configuration =====

#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub method: reqwest::Method,
    pub headers: HashMap<String, String>,
    pub query: Option<HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub auth_required: bool,
}

impl Default for RequestConfig {
    fn default() -> Self {
        Self {
            method: reqwest::Method::GET,
            headers: HashMap::new(),
            query: None,
            body: None,
            auth_required: false,
        }
    }
}

impl RequestConfig {
    pub fn get() -> Self {
        Self {
            method: reqwest::Method::GET,
            ..Default::default()
        }
    }

    pub fn post<T: Serialize>(body: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            method: reqwest::Method::POST,
            body: Some(serde_json::to_value(body)?),
            ..Default::default()
        })
    }

    pub fn put<T: Serialize>(body: &T) -> Result<Self, serde_json::Error> {
        Ok(Self {
            method: reqwest::Method::PUT,
            body: Some(serde_json::to_value(body)?),
            ..Default::default()
        })
    }

    pub fn delete() -> Self {
        Self {
            method: reqwest::Method::DELETE,
            ..Default::default()
        }
    }

    pub fn with_auth(mut self) -> Self {
        self.auth_required = true;
        self
    }

    pub fn with_query(mut self, query: HashMap<String, String>) -> Self {
        self.query = Some(query);
        self
    }

    pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

// ===== Utility Functions =====

impl Pagination {
    pub fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        if let Some(limit) = self.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if let Some(offset) = self.offset {
            params.insert("offset".to_string(), offset.to_string());
        }

        if let Some(page) = self.page {
            params.insert("page".to_string(), page.to_string());
        }

        if let Some(page_size) = self.page_size {
            params.insert("page_size".to_string(), page_size.to_string());
        }

        params
    }
}

// Helper trait for converting structs to query parameters
pub trait ToQueryParams {
    fn to_query_params(&self) -> HashMap<String, String>;
}

impl ToQueryParams for ListMarketsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();

        if let Some(market_addr) = &self.market_addr {
            params.insert("market_addr".to_string(), market_addr.clone());
        }

        if let Some(symbol) = &self.symbol {
            params.insert("symbol".to_string(), symbol.clone());
        }

        params
    }
}

impl ToQueryParams for ListOrdersParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();
        params.insert("market_addr".to_string(), self.market_addr.clone());

        if let Some(side) = &self.side {
            params.insert("side".to_string(), side.clone());
        }

        params
    }
}

impl ToQueryParams for ListFillsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();
        params.insert("market_addr".to_string(), self.market_addr.clone());
        params
    }
}

impl ToQueryParams for ListVaultsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        self.pagination.to_query_params()
    }
}

impl ToQueryParams for ListPositionsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();

        if let Some(market_addr) = &self.market_addr {
            params.insert("market_addr".to_string(), market_addr.clone());
        }

        params
    }
}

impl ToQueryParams for GetUserLeverageParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert("market_addr".to_string(), self.market_addr.clone());
        params
    }
}

impl ToQueryParams for ListCandlesParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();
        params.insert("market_addr".to_string(), self.market_addr.clone());
        params.insert("interval".to_string(), self.interval.clone());

        if let Some(start_time) = self.start_time {
            params.insert("start_time".to_string(), start_time.to_string());
        }

        if let Some(end_time) = self.end_time {
            params.insert("end_time".to_string(), end_time.to_string());
        }

        params
    }
}

impl ToQueryParams for ListFundingRatesParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();
        params.insert("market_addr".to_string(), self.market_addr.clone());

        if let Some(start_time) = self.start_time {
            params.insert("start_time".to_string(), start_time.to_string());
        }

        if let Some(end_time) = self.end_time {
            params.insert("end_time".to_string(), end_time.to_string());
        }

        params
    }
}

impl ToQueryParams for ListDepositsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();

        if let Some(user_addr) = &self.user_addr {
            params.insert("user_addr".to_string(), user_addr.clone());
        }

        if let Some(vault_addr) = &self.vault_addr {
            params.insert("vault_addr".to_string(), vault_addr.clone());
        }

        if let Some(asset_addr) = &self.asset_addr {
            params.insert("asset_addr".to_string(), asset_addr.clone());
        }

        if let Some(start_version) = self.start_version {
            params.insert("start_version".to_string(), start_version.to_string());
        }

        if let Some(end_version) = self.end_version {
            params.insert("end_version".to_string(), end_version.to_string());
        }

        params
    }
}

impl ToQueryParams for ListWithdrawsParams {
    fn to_query_params(&self) -> HashMap<String, String> {
        let mut params = self.pagination.to_query_params();

        if let Some(user_addr) = &self.user_addr {
            params.insert("user_addr".to_string(), user_addr.clone());
        }

        if let Some(vault_addr) = &self.vault_addr {
            params.insert("vault_addr".to_string(), vault_addr.clone());
        }

        if let Some(asset_addr) = &self.asset_addr {
            params.insert("asset_addr".to_string(), asset_addr.clone());
        }

        if let Some(start_version) = self.start_version {
            params.insert("start_version".to_string(), start_version.to_string());
        }

        if let Some(end_version) = self.end_version {
            params.insert("end_version".to_string(), end_version.to_string());
        }

        params
    }
}
