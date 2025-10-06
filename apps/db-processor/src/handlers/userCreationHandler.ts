import { producer } from "@repo/kafka"; // Singleton Kafka producer
import { db as prisma } from "@repo/db"; // Prisma client for database operations

export const userCreationHandler = async (message: any) => {
    try {
        const { email, name, password, correlationId } = message;

        // Validate the incoming message
        if (!email || !name || !password || !correlationId) {
            console.error("Invalid message: Missing required fields.");
            await producer.send({
                topic: "user-creation-response",
                messages: [
                    {
                        key: correlationId || "unknown",
                        value: JSON.stringify({
                            success: false,
                            message: "Invalid message: Missing required fields.",
                            correlationId,
                        }),
                    },
                ],
            });
            return;
        }

        // Step 1: Create the user in the database (password already hashed by API server)
        // Note: Password is already hashed, so we store it directly
        await prisma.user.create({
            data: { email, name, password, verfied: true },
        });

        console.log(`User with email ${email} created successfully.`);
        await producer.send({
            topic: "user-creation-response",
            messages: [
                {
                    key: correlationId,
                    value: JSON.stringify({
                        success: true,
                        message: "User created successfully.",
                        correlationId,
                    }),
                },
            ],
        });
    } catch (error:any) {
        console.error("Error processing user creation request:", error);
        await producer.send({
            topic: "user-creation-response",
            messages: [
                {
                    key: message.correlationId || "unknown",
                    value: JSON.stringify({
                        success: false,
                        message: `Failed to create user: ${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
};