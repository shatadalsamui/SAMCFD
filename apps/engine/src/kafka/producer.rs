use crate::modules::state::SharedEngineState;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde_json::json;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::sync::Arc;

// Create a global, shared producer instance
pub static PRODUCER: Lazy<Arc<FutureProducer>> = Lazy::new(|| {
    let producer: FutureProducer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation failed");
    Arc::new(producer)
});

/// Send a balance request for a user to the "balance-request" topic.
pub async fn send_balance_request(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "user_id": user_id
    })
    .to_string();

    let record = FutureRecord::to("balance-request")
        .key(user_id)
        .payload(&payload);

    match PRODUCER.send(record, Duration::from_secs(0)).await {
        Ok(_) => println!("Balance request sent for user: {}", user_id),
        Err((e, _)) => println!("Failed to produce balance request: {}", e),
    }

    Ok(())
}