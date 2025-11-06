use once_cell::sync::Lazy;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::de::Error as SerdeDeError;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

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
        "userId": user_id
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

/// Send a holdings request for a user and asset to the "holdings-request" topic.
pub async fn send_holdings_request(
    user_id: &str,
    asset: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = json!({
        "userId": user_id,
        "asset": asset
    })
    .to_string();

    let record = FutureRecord::to("holdings-request")
        .key(user_id)
        .payload(&payload);

    match PRODUCER.send(record, Duration::from_secs(0)).await {
        Ok(_) => println!(
            "Holdings request sent for user: {}, asset: {}",
            user_id, asset
        ),
        Err((e, _)) => println!("Failed to produce holdings request: {}", e),
    }

    Ok(())
}

/// Send a trade-create-response event to Kafka.
pub async fn send_trade_create_response(key: &str, response: &str) {
    let record = FutureRecord::to("trade-create-response")
        .key(key)
        .payload(response);
    match PRODUCER.send(record, Duration::from_secs(0)).await {
        Ok(_) => println!("trade-create-response sent for key: {}", key),
        Err((e, _)) => println!("Failed to produce trade-create-response: {}", e),
    }
}

pub async fn publish_trade_outcome(msg: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the trade_id from the JSON message
    let trade_id = serde_json::from_str::<serde_json::Value>(msg)
        .and_then(|v| {
            v.get("trade_id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| SerdeDeError::custom("Missing trade_id"))
        })
        .unwrap_or_default();

    let record = FutureRecord::to("trade-outcome")
        .key(&trade_id)
        .payload(msg);

    match PRODUCER.send(record, Duration::from_secs(0)).await {
        Ok(_) => println!("Trade outcome published: {}", msg),
        Err((e, _)) => println!("Failed to publish trade outcome: {}", e),
    }

    Ok(())
}
