import { createOrderSchema } from "@repo/schemas";
import { producer } from "@repo/kafka";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import { v4 as uuidv4 } from "uuid";
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
        const { margin, asset, type, leverage, slippage } = result.data;

        // 3. Check balance via Kafka request-response
        const balanceResponse = await kafkaRequestResponse(
            "balance-query-request",
            "balance-query-response",
            { userId }
        );

        const balance = balanceResponse?.balance ?? 0;
        const marginUnits = Math.round(margin * 100);

        if (balance < marginUnits) {
            return res.status(400).json({ message: "Insufficient balance" });
        }

        // 4. Generate orderId
        const orderId = uuidv4();

        // 5. Fire-and-forget: publish to Kafka
        await producer.send({
            topic: "trade-create-request",
            messages: [
                {
                    key: orderId,
                    value: JSON.stringify({
                        userId,
                        orderId,
                        asset,
                        type,
                        margin: marginUnits,
                        leverage,
                        slippage,
                        timestamp: Date.now(),
                    }),
                },
            ],
        });

        // 6. Respond immediately
        return res.status(200).json({ orderId });
    } catch (error: any) {
        console.error("Error in createOrderController:", error);
        return res.status(500).json({ message: "Internal server error" });
    }
};