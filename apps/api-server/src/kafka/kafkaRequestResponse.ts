import { producer, consumer } from "@repo/kafka";
import { v4 as uuidv4 } from "uuid";

// A map to store pending requests
const pendingRequests = new Map<string, (response: any) => void>();

let consumerReady = false;

// Set up the singleton consumer to listen for responses
const setupConsumer = async () => {
    console.log("[Kafka Consumer] Connecting...");
    await consumer.connect();
    console.log("[Kafka Consumer] Connected, subscribing to topics...");

    // Subscribe to all response topics
    await consumer.subscribe({ topic: "user-existence-response", fromBeginning: false });
    await consumer.subscribe({ topic: "user-creation-response", fromBeginning: false });
    await consumer.subscribe({ topic: "user-authentication-response", fromBeginning: false });
    await consumer.subscribe({ topic: "balance-query-response", fromBeginning: false });
    await consumer.subscribe({ topic: "trade-create-response", fromBeginning: false }); 
    await consumer.subscribe({ topic: "trade-close-response", fromBeginning: false }); 
    await consumer.subscribe({ topic: "holdings-query-response", fromBeginning: false });

    console.log("[Kafka Consumer] Subscribed, starting consumer...");
    await consumer.run({
        eachMessage: async ({ topic, message }) => {
            if (!message.value) {
                console.warn(`Received a message with null value on topic: ${topic}`);
                return;
            }

            const parsedMessage = JSON.parse(message.value.toString());
            console.log(`[Kafka Consumer] Received message on topic ${topic}:`, parsedMessage);
            const { correlationId } = parsedMessage;

            // Resolve the pending request if the correlation ID matches
            if (correlationId && pendingRequests.has(correlationId)) {
                console.log(`[Kafka Consumer] Resolving pending request for correlationId: ${correlationId}`);
                const resolve = pendingRequests.get(correlationId);
                resolve!(parsedMessage);
                pendingRequests.delete(correlationId);
            } else {
                console.warn(`[Kafka Consumer] No pending request found for correlationId: ${correlationId}`);
            }
        },
    });

    consumerReady = true;
    console.log("[Kafka Consumer] Ready to receive messages");
};

// Ensure the consumer is set up once
setupConsumer().catch((err) => {
    console.error("Error setting up Kafka consumer:", err);
});

export const kafkaRequestResponse = async (
    requestTopic: string,
    responseTopic: string,
    message: any,
    timeout = 30000 // Increased timeout to 30 seconds
): Promise<any> => {
    // Wait for consumer to be ready (with timeout)
    const maxWait = 30000; // 30 seconds
    const startWait = Date.now();
    while (!consumerReady && (Date.now() - startWait) < maxWait) {
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    if (!consumerReady) {
        throw new Error("Kafka consumer not ready after 30 seconds");
    }

    const correlationId = uuidv4(); // Generate a unique correlation ID

    // Add correlationId to the message
    const messageWithCorrelationId = { ...message, correlationId };

    // Publish the message to the request topic
    await producer.connect();
    await producer.send({
        topic: requestTopic,
        messages: [{ key: correlationId, value: JSON.stringify(messageWithCorrelationId) }],
    });

    console.log(`Message published to ${requestTopic} with correlationId: ${correlationId}`);

    // Return a promise that resolves when the response is received
    return new Promise((resolve, reject) => {
        // Store the resolver in the pendingRequests map
        pendingRequests.set(correlationId, resolve);

        // Set a timeout to reject the promise if no response is received
        setTimeout(() => {
            if (pendingRequests.has(correlationId)) {
                pendingRequests.delete(correlationId);
                reject(new Error("Timeout waiting for Kafka response."));
            }
        }, timeout);
    });
};