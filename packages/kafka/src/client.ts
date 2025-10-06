import { Kafka, Producer, Consumer, Partitioners } from 'kafkajs';

// Create a single Kafka instance
const kafka = new Kafka({
  clientId: 'samcfd-app',
  brokers: [process.env.KAFKA_BROKER || 'localhost:9092'], // Use environment variable for flexibility
});

// Create a Kafka producer
const producer: Producer = kafka.producer({
  createPartitioner: Partitioners.LegacyPartitioner,
});

// Create a Kafka consumer
const consumer: Consumer = kafka.consumer({ groupId: 'samcfd-group' });

// Export the Kafka instance, producer, and consumer
export { producer, consumer };
export default kafka;