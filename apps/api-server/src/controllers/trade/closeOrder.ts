import { closeOrderSchema } from "@repo/schemas";
import { producer } from "@repo/kafka";
import type { Request, Response } from "express";

export const closeOrderController = async (req: Request, res: Response) => {
    try {
        const userId = (req as any).user.id;
        const result = closeOrderSchema.safeParse(req.body);

        if (!result.success) {
            return res.status(400).json({ message: "Invalid input", errors: result.error.issues });
        }

        await producer.send({
            topic: "trade-close-request",
            messages: [
                {
                    key: result.data.orderId,
                    value: JSON.stringify({
                        userId,
                        orderId: result.data.orderId,
                        timestamp: Date.now(),
                    }),
                },
            ],
        });

        return res.status(200).json({ message: "Close order submitted" });
    } catch (error: any) {
        console.error("Error in closeOrderController:", error);
        return res.status(500).json({ message: "Internal server error" });
    }
};