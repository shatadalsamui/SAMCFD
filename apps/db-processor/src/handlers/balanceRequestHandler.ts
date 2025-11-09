import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const balanceRequestHandler = async (parsedMessage: any) => {
    try {
        console.log("Received balance-request message:", parsedMessage);
        const user_id = parsedMessage.user_id || parsedMessage.userId;
        const balanceRecord = await prisma.balance.findUnique({ where: { userId: user_id } });
        console.log("Balance record from DB for user:", user_id, balanceRecord);
        const toBigIntString = (val: any): string => {
            if (val === undefined || val === null) {
                return "0";
            }
            if (typeof val === "bigint") {
                return val.toString();
            }
            if (typeof val === "number") {
                if (!Number.isFinite(val)) {
                    return "0";
                }
                return Math.trunc(val).toString();
            }
            try {
                return BigInt(val).toString();
            } catch (error) {
                console.error("Failed to normalize balance value to BigInt string:", val, error);
                return "0";
            }
        };

        const balance = balanceRecord ? toBigIntString(balanceRecord.amount) : "0";

        await producer.send({
            topic: "balance-response",
            messages: [
                {
                    key: user_id,
                    value: JSON.stringify({ user_id, balance }),
                },
            ],
        });
        console.log(`Balance response sent for user: ${user_id}, balance: ${balance}`);
    } catch (error) {
        console.error("Error in balanceRequestHandler:", error);
    }
};