import { db as prisma } from "@repo/db";
import { TradeStatus } from "@repo/db/generated/prisma";

export const tradeOutcomeHandler = async (message: any) => {
    try {
        const {
            tradeId,
            userId,
            asset,
            side,
            quantity,
            entryPrice,
            closePrice,
            pnl,
            status,
            timestamp,
            margin,
            leverage,
            slippage,
            user
        } = message;

        // Map side to Prisma enum
        const prismaSide = typeof side === "string"
            ? side.toUpperCase() === "BUY" ? "BUY"
            : side.toUpperCase() === "SELL" ? "SELL"
            : undefined
            : undefined;

        // Map status to Prisma enum
        const prismaStatus = typeof status === "string" && TradeStatus[status.toUpperCase() as keyof typeof TradeStatus]
            ? TradeStatus[status.toUpperCase() as keyof typeof TradeStatus]
            : undefined;

        if (!tradeId || !userId || !asset || !prismaSide || !quantity || !entryPrice || !prismaStatus) {
            console.error("Invalid trade outcome message: missing required fields or invalid side/status value");
            return;
        }

        const updatePayload: any = {
            asset,
            side: prismaSide,
            quantity,
            entryPrice,
            closePrice,
            pnl,
            status: prismaStatus,
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: margin ?? 0,
            leverage: leverage ?? 0,
            slippage: slippage ?? 0,
        };
        if (userId) {
            updatePayload.user = { connect: { id: userId } };
        }

        const createPayload: any = {
            id: tradeId,
            asset,
            side: prismaSide,
            quantity,
            entryPrice,
            closePrice,
            pnl,
            status: prismaStatus,
            createdAt: timestamp ? new Date(timestamp) : new Date(),
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: margin ?? 0,
            leverage: leverage ?? 0,
            slippage: slippage ?? 0,
        };
        if (userId) {
            createPayload.user = { connect: { id: userId } };
        }

        await prisma.trade.upsert({
            where: { id: tradeId },
            update: updatePayload,
            create: createPayload,
        });

        console.log(`Trade outcome processed for tradeId: ${tradeId}`);
    } catch (error: any) {
        console.error("Error in trade outcome handler:", error);
    }
};