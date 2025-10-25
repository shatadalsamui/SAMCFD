// Import serde traits for (de)serialization.
use serde::{Deserialize, Serialize};

/// Represents a trade (position) in the engine.
/// Field names use Rust's snake_case convention.
/// The #[serde(rename = "...")] attribute ensures JSON keys match camelCase used by other services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub asset: String,
    #[serde(rename = "type")]
    pub trade_type: String, // "long" or "short"
    pub margin: i64,
    pub leverage: i32,
    #[serde(rename = "entryPrice")]
    pub entry_price: Option<i64>,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "closedAt")]
    pub closed_at: Option<String>,
}

/// Represents a user's balance in the engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub amount: i64,
}

/// Message sent from API server to engine to request trade creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTradeRequest {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub asset: String,
    #[serde(rename = "type")]
    pub trade_type: String,
    pub margin: i64,
    pub leverage: i32,
    pub slippage: i32,
    pub timestamp: i64,
}

/// Price update message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdate {
    pub asset: String,
    pub price: i64,
    pub timestamp: i64,
}

/// Outcome produced by the engine after processing a trade request or liquidation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOutcome {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub success: bool,
    pub reason: Option<String>,
    pub pnl: Option<i64>,
    pub status: String,
}

/// Message sent from API server to engine to request trade closure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloseTradeRequest {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "orderId")]
    pub order_id: String,
    pub timestamp: i64,
}
