import { db as prisma } from "@repo/db";
import { Prisma, TradeStatus } from "@repo/db/generated/prisma";

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

        const parseBigIntField = (value: unknown, field: string, required = false): bigint | undefined => {
            if (value === undefined || value === null || value === "") {
                if (required) {
                    console.error(`Missing required BigInt field: ${field}`);
                }
                return undefined;
            }
            try {
                return BigInt(value as any);
            } catch (error) {
                console.error(`Invalid BigInt for ${field}:`, value, error);
                return undefined;
            }
        };

        const parseIntField = (value: unknown): number | undefined => {
            if (value === undefined || value === null || value === "") {
                return undefined;
            }
            const parsed = Number.parseInt(String(value), 10);
            return Number.isNaN(parsed) ? undefined : parsed;
        };

        const parsedQuantity = parseBigIntField(quantity, "quantity", true);
        const parsedEntryPrice = parseBigIntField(entryPrice, "entryPrice", true);
        const parsedClosePrice = parseBigIntField(closePrice, "closePrice");
        const parsedPnl = parseBigIntField(pnl, "pnl");
        const parsedMargin = parseBigIntField(margin, "margin") ?? 0n;
        const parsedLimitPrice = parseBigIntField(limitPrice, "limitPrice");
        const parsedLockedMargin = parseBigIntField(
            message.lockedMargin ?? message.locked_margin,
            "lockedMargin"
        );
        const parsedStopLossPercent = parseIntField(message.stopLossPercent ?? message.stop_loss_percent);
        const parsedTakeProfitPercent = parseIntField(message.takeProfitPercent ?? message.take_profit_percent);

        if (parsedQuantity === undefined || parsedEntryPrice === undefined) {
            console.error("Unable to process trade outcome due to missing required numeric fields.");
            return;
        }

        const parsedLeverage = parseIntField(leverage) ?? 0;
        const parsedSlippage = parseIntField(slippage) ?? 0;

        const updatePayload: any = {
            asset,
            side: prismaSide,
            quantity: parsedQuantity,
            entryPrice: parsedEntryPrice,
            closePrice: parsedClosePrice,
            pnl: parsedPnl,
            status: prismaStatus,
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: parsedMargin,
            leverage: parsedLeverage,
            slippage: parsedSlippage,
        };
        if (parsedLockedMargin !== undefined) {
            updatePayload.lockedMargin = parsedLockedMargin;
        }
        if (prismaOrderType) {
            updatePayload.orderType = prismaOrderType;
        }
        if (parsedLimitPrice !== undefined) {
            updatePayload.limitPrice = parsedLimitPrice;
        }
        if (parsedStopLossPercent !== undefined) {
            updatePayload.stopLossPercent = parsedStopLossPercent;
        }
        if (parsedTakeProfitPercent !== undefined) {
            updatePayload.takeProfitPercent = parsedTakeProfitPercent;
        }
        if (userId) {
            updatePayload.user = { connect: { id: userId } };
        }

        const createPayload: any = {
            id: tradeId,
            asset,
            side: prismaSide,
            quantity: parsedQuantity,
            entryPrice: parsedEntryPrice,
            closePrice: parsedClosePrice,
            pnl: parsedPnl,
            status: prismaStatus,
            createdAt: timestamp ? new Date(timestamp) : new Date(),
            closedAt: timestamp ? new Date(timestamp) : undefined,
            margin: parsedMargin,
            leverage: parsedLeverage,
            slippage: parsedSlippage,
        };
        if (parsedLockedMargin !== undefined) {
            createPayload.lockedMargin = parsedLockedMargin;
        }
        if (prismaOrderType) {
            createPayload.orderType = prismaOrderType;
        }
        if (parsedLimitPrice !== undefined) {
            createPayload.limitPrice = parsedLimitPrice;
        }
        if (parsedStopLossPercent !== undefined) {
            createPayload.stopLossPercent = parsedStopLossPercent;
        }
        if (parsedTakeProfitPercent !== undefined) {
            createPayload.takeProfitPercent = parsedTakeProfitPercent;
        }
        if (userId) {
            createPayload.user = { connect: { id: userId } };
        }

        const prismaOperations: Prisma.PrismaPromise<unknown>[] = [
            prisma.trade.upsert({
                where: { id: tradeId },
                update: updatePayload,
                create: createPayload,
            }),
        ];

        if (userId && message.updatedBalance !== undefined && message.updatedBalance !== null) {
            const updatedBalance = parseBigIntField(message.updatedBalance, "updatedBalance");
            if (updatedBalance !== undefined) {
                prismaOperations.push(
                    prisma.balance.upsert({
                        where: { userId },
                        update: { amount: updatedBalance },
                        create: { userId, amount: updatedBalance },
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
            const updatedHoldings = parseBigIntField(message.updatedHoldings, "updatedHoldings");
            if (updatedHoldings !== undefined) {
                
                prismaOperations.push(
                    prisma.holdings.upsert({
                        where: { userId_asset: { userId, asset } },
                        update: { quantity: updatedHoldings },
                        create: {
                            userId,
                            asset,
                            quantity: updatedHoldings,
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