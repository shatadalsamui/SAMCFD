use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::modules::types::{Order, Trade};

pub struct OrderBook {
    pub buy: BTreeMap<i64, VecDeque<Order>>,  // price -> FIFO queue (highest price first)
    pub sell: BTreeMap<i64, VecDeque<Order>>, // price -> FIFO queue (lowest price first)
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
    pub balances: HashMap<String, i64>, // user_id -> balance
    pub open_trades: HashMap<String, Trade>, // order_id -> Trade
    pub order_books: HashMap<String, OrderBook>, // asset -> order book
    pub prices: HashMap<String, i64>, // asset -> price
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            open_trades: HashMap::new(),
            order_books: HashMap::new(),
            prices: HashMap::new(),
        }
    }
}

// Shared state type for concurrent access
pub type SharedEngineState = Arc<Mutex<EngineState>>;