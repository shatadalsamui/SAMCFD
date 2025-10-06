import { producer } from "@repo/kafka"; // Singleton Kafka producer
import { db as prisma } from "@repo/db"; // Prisma client for database operations

export const userExistenceHandler = async (message: any) => {
    try {
        const { email, correlationId } = message;

        // Validate the incoming message
        if (!email || !correlationId) {
            console.error("Invalid message: Missing email or correlationId.");
            await producer.send({
                topic: "user-existence-response",
                messages: [
                    {
                        key: correlationId || "unknown",
                        value: JSON.stringify({
                            success: false,
                            message: "Invalid message: Missing email or correlationId.",
                            correlationId,
                        }),
                    },
                ],
            });
            return;
        }

        //step 1 check if the user exists in the database 
        const existingUser = await prisma.user.findUnique({
            where: { email },
        });

        const exists = !!existingUser;

        console.log(`User existence check for email ${email}: ${exists}`);
        await producer.send({
            topic: "user-existence-response",
            messages: [
                {
                    key: correlationId,
                    value: JSON.stringify({
                        success: true,
                        exists,
                        correlationId,
                    }),
                },
            ],
        });

    } catch (error: any) {
        console.error("Error processing user existence check:", error);
        await producer.send({
            topic: "user-existence-response",
            messages: [
                {
                    key: message.correlationId,
                    value: JSON.stringify({
                        success: false,
                        message: `Failed to check user existence.${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
};