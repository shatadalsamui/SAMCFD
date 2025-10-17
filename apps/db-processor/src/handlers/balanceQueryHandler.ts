import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const balanceQueryHandler = async (message: any) => {
    try {

        const { userId, correlationId } = message;

        if (!userId || !correlationId) {
            console.error("Invalid message: Missing userId or correlationId");
            await producer.send({
                topic: "balance-query-response",
                messages: [
                    {
                        key: correlationId || "unknown",
                        value: JSON.stringify({
                            success: false,
                            message: "Invalid message: Missing userId or correlationId.",
                            correlationId,
                        })
                    }
                ]
            });
            return;
        }
        const balanceRecord = await prisma.balance.findUnique({
            where: { userId },
        });

        if (!balanceRecord) {
            await producer.send({
                topic: "balance-query-response",
                messages: [
                    {
                        key: correlationId,
                        value: JSON.stringify({
                            success: false,
                            message: "Balance not found for user.",
                            correlationId,
                        }),
                    },
                ],
            });
            return;
        }

        console.log(`Balance fetched for user ${userId}: ${balanceRecord.amount}`);
        await producer.send({
            topic: "balance-query-response",
            messages: [
                {
                    key: correlationId,
                    value: JSON.stringify({
                        success: true,
                        balance: balanceRecord.amount,
                        correlationId,
                    }),
                },
            ],
        });
    } catch (error: any) {
        console.error("Error in balance query handler:", error);
        await producer.send({
            topic: "balance-query-response",
            messages: [
                {
                    key: message.correlationId || "unknown",
                    value: JSON.stringify({
                        success: false,
                        message: `Error fetching balance: ${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
}