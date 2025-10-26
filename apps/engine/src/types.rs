use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
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
#[serde(rename_all = "camelCase")]
pub struct CreateTradeRequest {
    pub user_id: String,
    pub order_id: String,
    pub asset: String,
    pub side: Side,                       // "buy" | "sell"
    pub margin: i64,                      // smallest unit (cents)
    pub leverage: i32,
    pub slippage: Option<i32>,
    pub order_type: Option<OrderType>,    // "market" | "limit"
    pub limit_price: Option<i64>,         // smallest unit
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub trade_term: Option<String>,
    pub time_in_force: Option<String>,
    pub expiry_timestamp: Option<i64>,    // ms since epoch
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdate {
    pub asset: String,
    pub price: i64,      // smallest unit
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: String,
    pub user_id: String,
    pub asset: String,
    pub side: Side,
    pub margin: i64,
    pub leverage: i32,
    pub entry_price: Option<i64>,
    pub close_price: Option<i64>,
    pub pnl: Option<i64>,
    pub status: Option<String>,
    pub created_at: Option<i64>,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOutcome {
    pub order_id: String,
    pub user_id: String,
    pub success: bool,
    pub reason: Option<String>,
    pub pnl: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseTradeRequest {
    pub user_id: String,
    pub order_id: String,
    pub close_price: Option<i64>, // engine provides when closing
    pub reason: Option<String>,
    pub timestamp: i64,
}
