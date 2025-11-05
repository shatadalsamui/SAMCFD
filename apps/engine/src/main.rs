mod kafka;
mod modules;

use kafka::consumer::{consume_balance_responses, consume_price_updates, consume_trade_requests, consume_holdings_responses};
use kafka::producer;
use modules::price_updater::spawn_price_logger;
use modules::state::EngineState;
use modules::stop_loss_take_profit::monitor_stop_loss_take_profit;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(EngineState::new()));

    // Spawn Trade Request Consumer (fast jobs)
    let trade_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_trade_requests(trade_state).await {
            eprintln!("Error in Trade Request Consumer: {:?}", e);
        }
    });

    // Spawn Price Update Consumer (slow jobs)
    let price_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_price_updates(price_state).await {
            eprintln!("Error in Price Update Consumer: {:?}", e);
        }
    });

    // Spawn Balance Response Consumer
    let balance_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_balance_responses(balance_state).await {
            eprintln!("Error in Balance Response Consumer: {:?}", e);
        }
    });

    // Spawn Holdings Response Consumer
    let holdings_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = consume_holdings_responses(holdings_state).await {
            eprintln!("Error in Holdings Response Consumer: {:?}", e);
        }
    });

    spawn_price_logger(state.clone());

    // Start stop-loss and take-profit monitoring
    let stop_loss_state = state.clone();
    tokio::spawn(async move {
        loop {
            monitor_stop_loss_take_profit(stop_loss_state.clone()).await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    println!("Concurrent Engine started successfully with separate consumers.");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl_c");
    println!("Shutting down engine.");
}
