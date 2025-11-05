import { createOrderSchema } from "@repo/schemas";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import type { Request, Response } from "express";

export const createOrderController = async (req: Request, res: Response) => {
    try {
        // 1. Get userId from auth middleware
        const userId = (req as any).user.id;

        // 2. Validate request body
        const result = createOrderSchema.safeParse(req.body);
        if (!result.success) {
            return res.status(400).json({ message: "Invalid input", errors: result.error.issues });
        }
        const {
            margin,
            asset,
            side,
            leverage,
            quantity,
            slippage,
            orderType,
            limitPrice,
            stopLossPercent,
            takeProfitPercent,
            tradeTerm,
            timeInForce,
            expiryTimestamp,
        } = result.data;


        // 4. Send request and wait for engine response
        const engineResponse = await kafkaRequestResponse(
            "trade-create-request",
            "trade-create-response",
            {
                userId,
                asset,
                side,
                margin,
                leverage,
                quantity,
                slippage,
                orderType,
                limitPrice,
                stopLossPercent,
                takeProfitPercent,
                tradeTerm,
                timeInForce,
                expiryTimestamp: expiryTimestamp ?? null,
                timestamp: Date.now(),
            }
        );

        // 5. Ensure orderId is always present in response
        const responseWithOrderId = {
            ...engineResponse,
            orderId: engineResponse.orderId ?? null,
        };
        return res.status(200).json(responseWithOrderId);
    } catch (error: any) {
        console.error("Error in createOrderController:", error);
        return res.status(500).json({ message: "Internal server error" });
    }
};