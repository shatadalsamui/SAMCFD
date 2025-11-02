use crate::modules::price_updater::handle_price_update;
use crate::modules::processor::process_trade_create;
use crate::modules::state::SharedEngineState;
use crate::modules::types::CreateTradeRequest;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use serde_json;

/// Consumer for fast trade requests (subscribed only to "trade-create-request")
pub async fn consume_trade_requests(state: SharedEngineState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Trade Request Consumer...");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "engine-trade-group")
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Trade Consumer creation failed");

    consumer.subscribe(&["trade-create-request"]).expect("Can't subscribe to trade-create-request");

    println!("Trade Request Consumer started, waiting for messages...");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(Ok(payload)) = message.payload_view::<str>() {  // Unwrap the Result
                    match serde_json::from_str::<CreateTradeRequest>(payload) {
                        Ok(req) => {
                            println!("Received trade create request: {:?}", req);
                            let state_clone = state.clone();
                            tokio::spawn(async move {
                                process_trade_create(state_clone, req).await;  // Remove if let Err, as it returns ()
                            });
                        }
                        Err(e) => {
                            println!("Failed to parse trade create request: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                println!("Error receiving trade message: {}", e);
            }
        }
    }
}

/// Consumer for slow price updates (subscribed only to "price-updates")
pub async fn consume_price_updates(state: SharedEngineState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Price Update Consumer...");

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "engine-price-group")
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Price Consumer creation failed");

    consumer.subscribe(&["price-updates"]).expect("Can't subscribe to price-updates");

    println!("Price Update Consumer started, waiting for messages...");

    loop {
        match consumer.recv().await {
            Ok(message) => {
                if let Some(Ok(payload)) = message.payload_view::<str>() {  // Unwrap the Result
                    handle_price_update(payload, state.clone()).await;
                }
            }
            Err(e) => {
                println!("Error receiving price message: {}", e);
            }
        }
    }
}