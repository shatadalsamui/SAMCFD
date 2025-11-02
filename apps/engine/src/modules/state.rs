use crate::modules::types::{Order, Trade};
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;
pub struct OrderBook {
    pub buy: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    pub sell: BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
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
    pub balances: HashMap<String, f64>,          // user_id -> balance
    pub open_trades: HashMap<String, Trade>,     // order_id -> Trade
    pub order_books: HashMap<String, OrderBook>, // asset -> order book
    pub prices: HashMap<String, f64>,            // asset -> price
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
