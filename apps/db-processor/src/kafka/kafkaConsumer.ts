// filepath: /home/shatadal/SUPER30/WEEK1/CFD-Broker/apps/db-processor/src/kafka/kafkaConsumer.ts
import { Kafka } from "kafkajs";

// Create a separate Kafka consumer for the DB processor
const kafka = new Kafka({
  clientId: 'db-processor',
  brokers: [process.env.KAFKA_BROKER || 'localhost:9092'],
});

const dbConsumer = kafka.consumer({ groupId: 'db-processor-group' });

export const setupKafkaConsumer = async (
    topics: string[],
    messageHandler: (topic: string, message: any) => Promise<void>
) => {
    try {
        // Connect the consumer
        await dbConsumer.connect();

        // Subscribe to all provided topics
        for (const topic of topics) {
            await dbConsumer.subscribe({ topic, fromBeginning: true });
            console.log(`Subscribed to topic: ${topic}`);
        }

        // Run the consumer with the provided message handler
        await dbConsumer.run({
            eachMessage: async ({ topic, message }) => {
                await messageHandler(topic, message);
            },
        });

        console.log("Kafka consumer is running.");
    } catch (error) {
        console.error("Error setting up Kafka consumer:", error);
        throw error;
    }
};