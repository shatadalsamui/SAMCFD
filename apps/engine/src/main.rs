mod kafka;
mod modules;

use kafka::consumer;
use kafka::producer;
use modules::state::EngineState;
use std::sync::Arc;
use tokio::sync::Mutex;
use modules::price_updater::spawn_price_logger;

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

    spawn_price_logger(state.clone());

    println!("Engine started successfully.");
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl_c");
    println!("Shutting down engine.");
}
