import type { Request, Response } from "express";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";

export const getBalanceController = async (req: Request, res: Response) => {
    try {
        const userId = (req as any).user.id;

        const balanceResponse = await kafkaRequestResponse(
            "balance-query-request",
            "balance-query-response",
            { userId }
        );
        if (!balanceResponse.success) {
            return res.status(404).json({ message: balanceResponse.message || "Balance not found" });
        }
        res.json({ balance: balanceResponse.balance });
    } catch (error) {
        console.error("Error fetching balance:", error);
        res.status(500).json({ message: "Internal server error" });
    }
}