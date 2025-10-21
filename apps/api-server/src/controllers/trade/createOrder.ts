import type { Request, Response } from "express";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import { createOrderSchema } from "@repo/schemas";

export const createOrderController = async (req: Request, res: Response) => {
    try {
        const userId = (req as any).user.id;
        const validatedData = createOrderSchema.safeParse(req.body);

        if (!validatedData.success) {
            return res.status(400).json({ message: "Invalid input", errors: validatedData.error.issues });
        }

        // Need to check balance first
        const balanceResponse = await kafkaRequestResponse(
            "balance-query-request",
            "balance-query-response",
            { userId }
        );

        if (!balanceResponse.success || balanceResponse.balance < validatedData.data.margin) {
            return res.status(400).json({ message: "Insufficient balance" });
        }

        const tradeResponse = await kafkaRequestResponse(
            "trade-create-request",
            "trade-create-response",
            { userId, ...validatedData.data }
        );

        if (!tradeResponse.success) {
            return res.status(400).json({ message: tradeResponse.message });
        }

        res.json({ orderId: tradeResponse.orderId });
    } catch (error: any) {
        console.error("Error creating order:", error);
        res.status(500).json({ message: "Internal server error" });
    }
};