// Import standard library types for hash maps and atomic reference counting.
use std::collections::HashMap;
use std::sync::Arc;

// Import Tokio's async-aware Mutex for safe concurrent access.
use tokio::sync::Mutex;

// Import your Trade struct from types.rs (must declare `mod types;` in main.rs).
use crate::types::Trade;

/// EngineState holds all balances and open trades in memory.
/// - balances: maps user_id (String) to amount (i64)
/// - open_trades: maps trade_id (String) to Trade struct
pub struct EngineState {
    pub balances: HashMap<String, i64>,
    pub open_trades: HashMap<String, Trade>,
}

impl EngineState {
    /// Create a new, empty EngineState.
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            open_trades: HashMap::new(),
        }
    }
}

/// SharedEngineState is a thread-safe, async-safe pointer to EngineState.
/// Use this type to share state between async tasks.
pub type SharedEngineState = Arc<Mutex<EngineState>>;

/// Helper to create a new shared state instance.
pub fn shared_state() -> SharedEngineState {
    Arc::new(Mutex::new(EngineState::new()))
}