import { db as prisma } from "@repo/db";
import { producer } from "@repo/kafka";

export const holdingsQueryHandler = async (message: any) => {
    try {
        const { userId, asset } = message;

        if (!userId || !asset) {
            console.error("Invalid message: Missing userId or asset");
            await producer.send({
                topic: "holdings-query-response",
                messages: [
                    {
                        key: userId || "unknown",
                        value: JSON.stringify({
                            success: false,
                            message: "Invalid message: Missing userId or asset.",
                        }),
                    },
                ],
            });
            return;
        }

        const holdings = await prisma.holdings.findUnique({
            where: { userId_asset: { userId, asset } },
        });
        const toBigIntValue = (val: any): bigint => {
            if (val === undefined || val === null) {
                return 0n;
            }
            if (typeof val === "bigint") {
                return val;
            }
            if (typeof val === "number") {
                if (!Number.isFinite(val)) {
                    return 0n;
                }
                return BigInt(Math.trunc(val));
            }
            try {
                return BigInt(val);
            } catch (error) {
                console.error("Failed to normalize holdings value to BigInt:", val, error);
                return 0n;
            }
        };

        const heldQuantityBigInt = toBigIntValue(holdings?.quantity);
        const heldQuantity = heldQuantityBigInt.toString();

        console.log(`Holdings check for user ${userId}, asset ${asset}: held=${heldQuantity}`);
        await producer.send({
            topic: "holdings-query-response",
            messages: [
                {
                    key: userId,
                    value: JSON.stringify({
                        sufficient: heldQuantityBigInt > 0n,
                        heldQuantity,
                        userId,
                        asset,
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
                    key: message.userId || "unknown",
                    value: JSON.stringify({
                        sufficient: false,
                        reason: `Error checking holdings: ${error.message}`,
                        userId: message.userId,
                        asset: message.asset,
                    }),
                },
            ],
        });
    }
};