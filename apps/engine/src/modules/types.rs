use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Liquidated,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            OrderStatus::Open => "Open",
            OrderStatus::PartiallyFilled => "PartiallyFilled",
            OrderStatus::Filled => "Filled",
            OrderStatus::Cancelled => "Cancelled",
            OrderStatus::Liquidated => "Liquidated",
        };
        write!(f, "{}", status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTradeRequest {
    pub user_id: String,
    pub correlation_id: Option<String>, // For request-response matching
    pub asset: String,
    pub side: Side,  // "buy" | "sell"
    pub margin: f64, // smallest unit (cents)
    pub leverage: f64,
    pub slippage: Option<f64>,
    pub order_type: Option<OrderType>, // "market" | "limit"
    pub limit_price: Option<f64>,      // smallest unit
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub trade_term: Option<String>,
    pub time_in_force: Option<String>,
    pub expiry_timestamp: Option<i64>, // ms since epoch
    pub timestamp: i64,
    pub quantity: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub asset: String,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Option<f64>, // limit price (None for market)
    pub quantity: f64,
    pub filled: f64,
    pub status: OrderStatus,
    pub margin: f64,
    pub leverage: f64,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub created_at: i64,
    pub expiry: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdate {
    pub asset: String,
    pub price: f64, // smallest unit
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    pub id: String,
    pub user_id: String,
    pub asset: String,
    pub side: Side,
    pub margin: f64,
    pub leverage: f64,
    pub quantity: f64,
    pub entry_price: Option<f64>,
    pub close_price: Option<f64>,
    pub pnl: Option<f64>,
    pub status: Option<String>,
    pub created_at: Option<i64>,
    pub closed_at: Option<i64>,
    pub take_profit_percent: Option<f64>,
    pub stop_loss_percent: Option<f64>,
    pub price: Option<f64>,
    
}

pub fn order_to_trade(order: &Order) -> Trade {
    Trade {
        id: order.id.clone(),
        user_id: order.user_id.clone(),
        asset: order.asset.clone(),
        side: order.side.clone(),
        margin: order.margin,
        leverage: order.leverage,
        quantity: order.quantity,
        entry_price: order.price,
        close_price: None,
        pnl: None,
        status: Some(order.status.to_string()),
        created_at: Some(order.created_at),
        closed_at: None,
        take_profit_percent: order.take_profit_percent,
        stop_loss_percent: order.stop_loss_percent,
        price: order.price,
    }
}

pub fn trade_to_order(trade: &Trade) -> Order {
    Order {
        id: trade.id.clone(),
        user_id: trade.user_id.clone(),
        asset: trade.asset.clone(),
        side: trade.side.clone(),
        order_type: OrderType::Market, // Defaulting to Market
        price: trade.price,
        quantity: trade.quantity,
        filled: 0.0, // Assuming not relevant when converting back for PnL
        status: OrderStatus::Open, // Assuming a default status
        margin: trade.margin,
        leverage: trade.leverage,
        stop_loss_percent: trade.stop_loss_percent,
        take_profit_percent: trade.take_profit_percent,
        created_at: trade.created_at.unwrap_or(0),
        expiry: None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeOutcome {
    pub trade_id: String,
    pub user_id: String,
    pub asset: String,
    pub side: Side,
    pub quantity: f64,
    pub entry_price: Option<f64>,
    pub close_price: Option<f64>,
    pub pnl: Option<f64>,
    pub status: Option<String>, // "opened", "matched", "liquidated", "closed"
    pub timestamp: Option<i64>,
    pub margin: Option<f64>,
    pub leverage: Option<f64>,
    pub slippage: Option<f64>,
    pub reason: Option<String>,
    pub success: Option<bool>,
    pub order_type: Option<OrderType>,
    pub limit_price: Option<f64>,
    pub updated_balance:Option<f64>,
    pub updated_holdings:Option<f64>
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

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub buy: std::collections::BTreeMap<i64, std::collections::VecDeque<Order>>, // Buy orders sorted by price
    pub sell: std::collections::BTreeMap<i64, std::collections::VecDeque<Order>>, // Sell orders sorted by price
}

#[derive(Debug, Clone)]
pub struct EngineState {
    pub balances: std::collections::HashMap<String, i64>, // User balances
    pub order_books: std::collections::HashMap<String, OrderBook>, // Asset -> OrderBook
    pub open_orders: std::collections::HashMap<String, Order>, // Order ID -> Order
}
