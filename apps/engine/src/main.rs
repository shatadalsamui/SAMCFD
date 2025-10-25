// Import the state module and types module.
mod state;
mod types;

use state::{shared_state, SharedEngineState};

#[tokio::main] // Start the Tokio async runtime.
async fn main() {
    println!("Engine starting...");

    // Create the shared, thread-safe engine state.
    let state: SharedEngineState = shared_state();

    // Here you will later spawn Kafka consumer tasks and pass `state.clone()` to each.
    // Example:
    // tokio::spawn(consume_trade_requests(state.clone()));
    // tokio::spawn(consume_price_updates(state.clone()));

    // For now, just keep the process alive (so the async runtime doesn't exit).
    // Remove this later when you add real tasks.
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}