use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::time::Duration;
use crate::types::CreateTradeRequest;
use serde_json;
use crate::state::SharedEngineState;

pub async fn start_consumer(_state: SharedEngineState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Kafka consumer...");

    // Configure the Kafka consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "engine-group")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["trade-create-request"])
        .expect("Can't subscribe");

    println!("Kafka consumer started, waiting for messages...");

    loop {
        match consumer.recv().await {
            Ok(m) => {
                match m.payload_view::<str>() {
                    Some(Ok(payload)) => {
                        handle_trade_create_request(payload);
                    }
                    Some(Err(e)) => {
                        println!("Failed to decode message payload: {}", e);
                    }
                    None => {
                        println!("Received message with empty payload");
                    }
                }
            }
            Err(e) => {
                println!("Kafka error: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// Separate handler function for trade create requests
fn handle_trade_create_request(payload: &str) {
    match serde_json::from_str::<CreateTradeRequest>(payload) {
        Ok(req) => println!("Parsed CreateTradeRequest: {:?}", req),
        Err(e) => println!("Failed to parse CreateTradeRequest: {}", e),
    }
}