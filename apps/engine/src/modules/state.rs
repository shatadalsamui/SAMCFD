use crate::modules::types::{CreateTradeRequest, Order, Trade};
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
pub struct OrderBook {
    pub buy: BTreeMap<i64, VecDeque<Order>>,
    pub sell: BTreeMap<i64, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            buy: BTreeMap::new(),
            sell: BTreeMap::new(),
        }
    }
}

pub struct EngineState {
    pub balances: HashMap<String, i64>,          // user_id -> balance (scaled integer)
    pub open_trades: HashMap<String, Trade>,     // order_id -> Trade
    pub order_books: HashMap<String, OrderBook>, // asset -> order book
    pub prices: HashMap<String, i64>,            // asset -> price (scaled integer)
    pub pending_trades: HashMap<String, Vec<CreateTradeRequest>>, // user_id -> trades
    pub holdings: HashMap<(String, String), i64>, // user_id , asset -> quantity
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            open_trades: HashMap::new(),
            order_books: HashMap::new(),
            prices: HashMap::new(),
            pending_trades: HashMap::new(),
            holdings: HashMap::new(),
        }
    }
}

// Shared state type for concurrent access
pub type SharedEngineState = Arc<Mutex<EngineState>>;
