import { producer, consumer } from "@repo/kafka";
import { v4 as uuidv4 } from "uuid";

// A map to store pending requests
const pendingRequests = new Map<string, (response: any) => void>();

// Set up the singleton consumer to listen for responses
const setupConsumer = async () => {
    await consumer.connect();

    // Subscribe to all response topics
    await consumer.subscribe({ topic: "user-existence-response", fromBeginning: false });
    await consumer.subscribe({ topic: "user-creation-response", fromBeginning: false });

    consumer.run({
        eachMessage: async ({ topic, message }) => {
            if (!message.value) {
                console.warn(`Received a message with null value on topic: ${topic}`);
                return;
            }

            const parsedMessage = JSON.parse(message.value.toString());
            const { correlationId } = parsedMessage;

            // Resolve the pending request if the correlation ID matches
            if (pendingRequests.has(correlationId)) {
                const resolve = pendingRequests.get(correlationId);
                resolve!(parsedMessage);
                pendingRequests.delete(correlationId);
            }
        },
    });
};

// Ensure the consumer is set up once
setupConsumer().catch((err) => {
    console.error("Error setting up Kafka consumer:", err);
});

export const kafkaRequestResponse = async (
    requestTopic: string,
    responseTopic: string,
    message: any,
    timeout = 10000 // Timeout in milliseconds
): Promise<any> => {
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