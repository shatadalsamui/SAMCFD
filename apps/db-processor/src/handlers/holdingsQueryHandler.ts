import { db as prisma } from "@repo/db";
import { producer } from "@repo/kafka";

export const holdingsQueryHandler = async (message: any) => {
    try {
        const { userId, asset, correlationId } = message;

        if (!userId || !asset || !correlationId) {
            console.error("Invalid message: Missing userId, asset, or correlationId");
            await producer.send({
                topic: "holdings-query-response",
                messages: [
                    {
                        key: correlationId,
                        value: JSON.stringify({
                            success: false,
                            message: "Invalid message: Missing userId, asset, or correlationId.",
                            correlationId,
                        }),
                    },
                ],
            });
            return;
        }

        const holdings = await prisma.holdings.findUnique({
            where: { userId_asset: { userId, asset } },
        });
        const heldQuantity = holdings?.quantity ?? 0;

        console.log(`Holdings check for user ${userId}, asset ${asset}: held=${heldQuantity}`);
        await producer.send({
            topic: "holdings-query-response",
            messages: [
                {
                    key: correlationId,
                    value: JSON.stringify({
                        sufficient: Number(heldQuantity) > 0,  // check if user holds any of the asset
                        heldQuantity: Number(heldQuantity),
                        correlationId,
                    }),
                },
            ],
        });
    } catch (error: any) {
        console.error("Error in holdings query handler:", error);
        await producer.send({
            topic: "holdings-query-response",
            messages: [
                {
                    key: message.correlationId || "unknown",
                    value: JSON.stringify({
                        sufficient: false,
                        reason: `Error checking holdings: ${error.message}`,
                        correlationId: message.correlationId,
                    }),
                },
            ],
        });
    }
};