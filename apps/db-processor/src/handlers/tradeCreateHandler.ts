import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const tradeCreateHandler = async (message: any) => {
    try {
        const { userId, asset, type, margin, leverage, slippage, correlationId } = message;

        if (!userId || !asset || !type || !margin || !leverage || !slippage || !correlationId) {
            await producer.send({
                topic: "trade-create-response",
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
            // Deduct margin from balance
            const balance = await tx.balance.findUnique({ where: { userId } });
            if (!balance || balance.amount < margin) {
                throw new Error("Insufficient balance");
            }
            await tx.balance.update({
                where: { userId },
                data: { amount: balance.amount - margin },
            });

            // Create trade
            const trade = await tx.trade.create({
                data: {
                    userId,
                    asset,
                    type,
                    margin,
                    leverage,
                    slippage,
                    status: "open",
                },
            });

            await producer.send({
                topic: "trade-create-response",
                messages: [
                    {
                        key: correlationId,
                        value: JSON.stringify({
                            success: true,
                            orderId: trade.id,
                            correlationId,
                        }),
                    },
                ],
            });
        });
    } catch (error: any) {
        console.error("Error creating trade:", error);
        await producer.send({
            topic: "trade-create-response",
            messages: [
                {
                    key: message.correlationId,
                    value: JSON.stringify({
                        success: false,
                        message: `Error creating trade: ${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
};