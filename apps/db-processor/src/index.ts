// DB Processor Entry Point
// Initializes the DB Processor service, setting up Kafka producer and consumer to handle user-related messages.

import { producer } from "@repo/kafka";
import { setupKafkaConsumer } from "./kafka/kafkaConsumer";
import { userExistenceHandler } from "./handlers/userExistenceHandler";
import { userCreationHandler } from "./handlers/userCreationHandler";
import { userAuthenticationHandler } from "./handlers/userAuthenticationHandler";
import { balanceQueryHandler } from "./handlers/balanceQueryHandler";
import { holdingsQueryHandler } from "./handlers/holdingsQueryHandler";
import { balanceRequestHandler } from "./handlers/balanceRequestHandler";
import { tradeOutcomeHandler } from "./handlers/tradeOutcomeHandler";

const messageHandler = async (topic: string, message: any) => {
    // Handles incoming Kafka messages based on their topic.
    try {
        if (!message.value) {
            console.warn(`Received a message with null value on topic: ${topic}`);
            return;
        }

        const parsedMessage = JSON.parse(message.value.toString());

        // Log message without sensitive data
        const logMessage = { ...parsedMessage };
        if (logMessage.password) {
            logMessage.password = "[REDACTED]";
        }
        console.log(`Processing message on topic ${topic}:`, logMessage);

        switch (topic) {
            case "user-existence-check":
                await userExistenceHandler(parsedMessage);
                break;
            case "user-creation-request":
                await userCreationHandler(parsedMessage);
                break;
            case "user-authentication-request":
                await userAuthenticationHandler(parsedMessage);
                break;
            case "balance-query-request":
                await balanceQueryHandler(parsedMessage);
                break;
            case "holdings-request":
                await holdingsQueryHandler(parsedMessage);
                break;
            case "balance-request":
                await balanceRequestHandler(parsedMessage);
                break;
            case "trade-outcome":
                await tradeOutcomeHandler(parsedMessage);
                break;
            default:
                console.warn(`Unknown topic: ${topic}`);
        }
    } catch (error) {
        console.error(`Error processing message on topic ${topic}:`, error);
    }
};

const startDbProcessor = async () => {
    // Starts the DB Processor service by connecting the Kafka producer and setting up the consumer.
    try {
        console.log("Starting DB Processor...");

        // Connect producer
        await producer.connect();
        console.log("Producer connected");

        // Setup consumer for the topics we want to listen to
        const topics =
            [
                "user-existence-check",
                "user-creation-request",
                "user-authentication-request",
                "balance-query-request",
                "holdings-request",
                "balance-request",
                "trade-outcome"
            ];

        await setupKafkaConsumer(topics, messageHandler);

        console.log("DB Processor started successfully");
    } catch (error) {
        console.error("Failed to start DB Processor:", error);
        process.exit(1);
    }
};

startDbProcessor();