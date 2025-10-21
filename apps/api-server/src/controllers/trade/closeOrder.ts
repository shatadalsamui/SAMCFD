import type { Request, Response } from "express";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import { closeOrderSchema } from "@repo/schemas";

export const closeOrderController = async (req: Request, res: Response) => {
    try {
        const userId = (req as any).user.id;
        const validatedData = closeOrderSchema.safeParse(req.body);

        if (!validatedData.success) {
            return res.status(400).json({ message: "Invalid input", errors: validatedData.error.issues });
        }

        const tradeResponse = await kafkaRequestResponse(
            "trade-close-request",
            "trade-close-response",
            { userId, orderId: validatedData.data.orderId }
        );

        if (!tradeResponse.success) {
            return res.status(400).json({ message: tradeResponse.message });
        }

        res.json({ message: "Order closed successfully" });
    } catch (error: any) {
        console.error("Error closing order:", error);
        res.status(500).json({ message: "Internal server error" });
    }
};