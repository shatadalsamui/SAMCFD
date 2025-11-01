mod kafka;
mod state;
mod types;
mod processor;
mod pnl;
mod liquidations;
mod stop_loss_take_profit;
mod order_matching;

use kafka::consumer;
use kafka::producer;
use state::EngineState;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(EngineState::new()));

    // Start the Kafka consumer
    let consumer_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = consumer::start_consumer(consumer_state).await {
            eprintln!("Error in Kafka consumer: {:?}", e);
        }
    });

    // Start the Kafka producer
    let producer_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = producer::start_producer(producer_state).await {
            eprintln!("Error in Kafka producer: {:?}", e);
        }
    });

    println!("Engine started successfully.");
}