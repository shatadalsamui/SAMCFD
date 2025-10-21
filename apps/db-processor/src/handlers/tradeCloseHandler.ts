import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const tradeCloseHandler = async (message: any) => {
    try {
        const { userId, orderId, correlationId } = message;

        if (!userId || !orderId || !correlationId) {
            await producer.send({
                topic: "trade-close-response",
                messages: [
                    {
                        key: correlationId,
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

        await prisma.$transaction(async (tx) => {
            // Find trade
            const trade = await tx.trade.findUnique({ where: { id: orderId, userId } });
            if (!trade || trade.status !== "open") {
                throw new Error("Trade not found or not open");
            }

            // Update trade to closed
            await tx.trade.update({
                where: { id: orderId },
                data: { status: "closed", closedAt: new Date() },
            });

            await producer.send({
                topic: "trade-close-response",
                messages: [
                    {
                        key: correlationId,
                        value: JSON.stringify({
                            success: true,
                            message: "Trade closed successfully",
                            correlationId,
                        }),
                    },
                ],
            });
        });
    } catch (error: any) {
        console.error("Error closing trade:", error);
        await producer.send({
            topic: "trade-close-response",
            messages: [
                {
                    key: message.correlationId,
                    value: JSON.stringify({
                        success: false,
                        message: `Error closing trade: ${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
};