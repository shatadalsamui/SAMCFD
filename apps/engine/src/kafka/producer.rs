use crate::modules::state::SharedEngineState; // Updated import for state
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;

pub async fn start_producer(_state: SharedEngineState) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Kafka producer...");

    let producer: FutureProducer = rdkafka::config::ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Producer creation failed");

    loop {
        let record = FutureRecord::to("trade-results")
            .key("key")
            .payload("payload");

        match producer.send(record, Duration::from_secs(0)).await {
            Ok(_delivery) => {}
            Err((e, _)) => println!("Failed to produce message: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
