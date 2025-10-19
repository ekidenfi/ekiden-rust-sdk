use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use aptos_crypto::{signing_message, CryptoMaterialError};
use aptos_crypto::ed25519::{Ed25519PrivateKey, Ed25519Signature};
use aptos_crypto_derive::{BCSCryptoHash, CryptoHasher};
use serde_with::{serde_as, DisplayFromStr};
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
    pub timestamp_ms: i64,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizeResponse {
    pub token: String,
}

// ===== Market Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketResponse {
    pub symbol: String,
    pub addr: String,
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
#[derive(Default)]
pub struct ListMarketsParams {
    pub market_addr: Option<String>,
    pub symbol: Option<String>,
    #[serde(flatten)]
    pub pagination: Pagination,
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
    pub addr: String,
    pub user_addr: String,
    pub asset_addr: String,
    pub amount: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVaultsParams {
    #[serde(flatten)]
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionResponse {
    pub sid: String,
    pub market_addr: String,
    pub user_addr: String,
    pub size: i64,
    pub price: u64,
    pub entry_price: u64,
    pub margin: u64,
    pub funding_index: u64,
    pub is_cross: bool,
    pub initial_margin: Option<u64>,
    pub initial_margin_mark: Option<u64>,
    pub maintenance_margin: Option<u64>,
    pub leverage: Option<u64>,
    pub mark_price: u64,
    pub side: String,
    pub unrealized_pnl: i64,
    pub liq_price: Option<u64>,
    pub timestamp: u64,
    pub timestamp_ms: u64,
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
    pub leverage: u64,
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
    pub vault_balances: Vec<PortfolioVault>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub total_value: Option<u64>,
    pub available_balance: Option<u64>,
    pub locked_balance: Option<u64>,
    pub unrealized_pnl: Option<i64>,
    pub margin_used: Option<u64>,
    pub margin_available: Option<u64>,
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
    pub id: u64,
    pub asset_addr: String,
    pub balance: u64,
}

// ===== Intent Types =====

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum TimeInForce {
    #[default]
    #[serde(rename = "GTC")]
    // Regardless of what porion of the order is filled, remainder is placed into OrderBook
    GTC,
    #[serde(rename = "IOC")]
    // Fills as much is possible, unfilled part is cancelled and not placed in OrderBook
    IOC,
    #[serde(rename = "FOK")]
    // The order is either filled in its entirety, otherwise it is cancelled
    FOK,
    #[serde(rename = "PostOnly")]
    // If any part of the order is matched, it is cancelled.
    // The intent is to place the order in OrderBook, and not fill it.
    PostOnly,
}

impl Display for TimeInForce {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::GTC => f.write_str("GTC"),
            TimeInForce::IOC => f.write_str("IOC"),
            TimeInForce::FOK => f.write_str("FOK"),
            TimeInForce::PostOnly => f.write_str("PostOnly"),
        }
    }
}

impl FromStr for TimeInForce {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GTC" => Ok(TimeInForce::GTC),
            "IOC" => Ok(TimeInForce::IOC),
            "FOK" => Ok(TimeInForce::FOK),
            "PostOnly" => Ok(TimeInForce::PostOnly),
            _ => Err(format!("Invalid TimeInForce: {}", s)),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreate {
    pub side: String,
    pub size: u64,
    pub price: u64,
    pub leverage: u64,

    /// The type of the order (limit, market, etc.)
    pub r#type: String,

    /// The address of the market
    pub market_addr: String,

    pub is_cross: bool,

    /// Time in force strategy. Defaults to GTC if not provided by the client.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub time_in_force: Option<TimeInForce>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateAction {
    pub orders: Vec<OrderCreate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancel {
    pub sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelAction {
    pub cancels: Vec<OrderCancel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelAllAction {
    /// If provided, cancels all active orders for this market. If None, cancels all active orders for the user.
    pub market_addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionPayload {
    OrderCreate(OrderCreateAction),
    OrderCancel(OrderCancelAction),
    OrderCancelAll(OrderCancelAllAction),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendIntentParams {
    pub payload: ActionPayload,
    pub nonce: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CryptoHasher, BCSCryptoHash)]
pub struct IntentSignatureBody {
    pub payload: ActionPayload,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateOutput {
    pub sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreateIntentOutput {
    pub outputs: Vec<OrderCreateOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelOutput {
    pub sid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelIntentOutput {
    pub outputs: Vec<OrderCancelOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelAllIntentOutput {
    pub outputs: Vec<OrderCancelOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntentOutput {
    OrderCreate(OrderCreateIntentOutput),
    OrderCancel(OrderCancelIntentOutput),
    OrderCancelAll(OrderCancelAllIntentOutput),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendIntentResponse {
    pub output: IntentOutput,
    pub seq: u64,
    pub version: u64,
    pub timestamp: u64,
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
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCandlesParams {
    pub market_addr: String,
    pub timeframe: String, // "1m", "5m", "15m", "1h", "4h", "1d"
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

    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_required = true;
        self.headers
            .insert("Authorization".to_string(), format!("Bearer {}", token));

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

    pub fn has_auth_header(&self) -> bool {
        self.headers.contains_key("Authorization") || self.headers.contains_key("authorization")
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
        params.insert("timeframe".to_string(), self.timeframe.clone());

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

pub trait SigningIntent {
    fn sign_intent(
        &self,
        intent: IntentSignatureBody,
    ) -> Result<Ed25519Signature, CryptoMaterialError>;
}

impl SigningIntent for Ed25519PrivateKey {
    fn sign_intent(
        &self,
        intent: IntentSignatureBody,
    ) -> Result<Ed25519Signature, CryptoMaterialError> {
        let signature = self.sign_message(&signing_message(&intent)?);

        Ok(signature)
    }
}
