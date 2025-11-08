import { db as prisma } from "@repo/db";
import { TradeStatus } from "@repo/db/generated/prisma";
import Decimal from "decimal.js";

export const tradeOutcomeHandler = async (message: any) => {
    try {

        console.log("RAW Trade Outcome Message Received:");
        console.log(JSON.stringify(message, null, 2));
        console.log("Updated Balance:", message.updatedBalance);
        console.log("Updated Holdings:", message.updatedHoldings);
        console.log("-------------------------------------------------------------------");

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
            orderType,
            limitPrice,
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

        // Map orderType to Prisma enum
        const prismaOrderType = typeof orderType === "string"
            ? orderType.toUpperCase() === "MARKET" ? "MARKET"
                : orderType.toUpperCase() === "LIMIT" ? "LIMIT"
                    : undefined
            : undefined;

        if (!tradeId || !userId || !asset || !prismaSide || !quantity || !entryPrice || !prismaStatus) {
            console.error("Invalid trade outcome message: missing required fields or invalid side/status value");
            return;
        }

        // Convert numeric fields to correct types
        const toDecimal = (val: any) => val !== undefined && val !== null ? new Decimal(val).toNumber() : undefined;
        const toInt = (val: any) => val !== undefined && val !== null ? Number(val) : undefined;

        const updatePayload: any = {
            asset,
            side: prismaSide,
            quantity: toDecimal(quantity),
            entryPrice: toDecimal(entryPrice),
            closePrice: toDecimal(closePrice),
            pnl: toDecimal(pnl),
            status: prismaStatus,
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: toDecimal(margin) ?? 0,
            leverage: toInt(leverage) ?? 0,
            slippage: toInt(slippage) ?? 0,
        };
        if (prismaOrderType) {
            updatePayload.orderType = prismaOrderType;
        }
        if (limitPrice !== undefined && limitPrice !== null) {
            updatePayload.limitPrice = toDecimal(limitPrice);
        }
        if (userId) {
            updatePayload.user = { connect: { id: userId } };
        }

        const createPayload: any = {
            id: tradeId,
            asset,
            side: prismaSide,
            quantity: toDecimal(quantity),
            entryPrice: toDecimal(entryPrice),
            closePrice: toDecimal(closePrice),
            pnl: toDecimal(pnl),
            status: prismaStatus,
            createdAt: timestamp ? new Date(timestamp) : new Date(),
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: toDecimal(margin) ?? 0,
            leverage: toInt(leverage) ?? 0,
            slippage: toInt(slippage) ?? 0,
        };
        if (prismaOrderType) {
            createPayload.orderType = prismaOrderType;
        }
        if (limitPrice !== undefined && limitPrice !== null) {
            createPayload.limitPrice = toDecimal(limitPrice);
        }
        if (userId) {
            createPayload.user = { connect: { id: userId } };
        }

        const prismaOperations: Parameters<typeof prisma.$transaction>[0] = [
            prisma.trade.upsert({
                where: { id: tradeId },
                update: updatePayload,
                create: createPayload,
            }),
        ];

        if (userId && message.updatedBalance !== undefined && message.updatedBalance !== null) {
            const updatedBalance = new Decimal(message.updatedBalance);
            if (!updatedBalance.isNaN()) {
                prismaOperations.push(
                    prisma.balance.upsert({
                        where: { userId },
                        update: { amount: updatedBalance.toNumber() },
                        create: { userId, amount: updatedBalance.toNumber() },
                    })
                );
                console.log(`Queued balance update for user ${userId}: ${updatedBalance.toString()}`);
            }
        }

        if (
            userId &&
            asset &&
            message.updatedHoldings !== undefined &&
            message.updatedHoldings !== null
        ) {
            const updatedHoldings = new Decimal(message.updatedHoldings);
            if (!updatedHoldings.isNaN()) {
                prismaOperations.push(
                    prisma.holdings.upsert({
                        where: { userId_asset: { userId, asset } },
                        update: { quantity: updatedHoldings.toNumber() },
                        create: {
                            userId,
                            asset,
                            quantity: updatedHoldings.toNumber(),
                        },
                    })
                );
                console.log(
                    `Queued holdings update for user ${userId} asset ${asset}: ${updatedHoldings.toString()}`
                );
            }
        }

        await prisma.$transaction(prismaOperations);

        console.log(`Trade outcome processed for tradeId: ${tradeId}`);
    } catch (error: any) {
        console.error("Error in trade outcome handler:", error);
    }
};