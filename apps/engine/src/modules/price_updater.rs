use crate::modules::state::SharedEngineState;
use serde_json::Value;
use tokio::time::{sleep, Duration};

/// Call this once at startup to periodically print all prices.
pub fn spawn_price_logger(state: SharedEngineState) {
    tokio::spawn(async move {
        loop {
            {
                let engine_state = state.lock().await;
                // Print all prices in a single line
                let mut prices: Vec<String> = Vec::new();
                for asset in ["BTC_USDC", "ETH_USDC", "SOL_USDC", "BNB_USDC", "DOGE_USDC"] {
                    let price = engine_state.prices.get(asset).cloned().unwrap_or(-1.0);
                    prices.push(format!("{}: {}", asset, price));
                }
                println!("Prices => {}", prices.join(" | "));
            }
            sleep(Duration::from_millis(100)).await;
        }
    });
}

/// Handles price updates and updates the `prices` field in `EngineState`.
pub async fn handle_price_update(payload: &str, state: SharedEngineState) {
    match serde_json::from_str::<Value>(payload) {
        Ok(price_update) => {
            if let Some(asset) = price_update["asset"].as_str() {
                // Always parse as f64 for accuracy
                let price_opt = price_update["price"].as_f64()
                    .or_else(|| price_update["price"].as_i64().map(|v| v as f64));

                if let Some(price) = price_opt {
                    let mut engine_state = state.lock().await;
                    engine_state.prices.insert(asset.to_string(), price);
                }
            }
        }
        Err(_) => {}
    }
}