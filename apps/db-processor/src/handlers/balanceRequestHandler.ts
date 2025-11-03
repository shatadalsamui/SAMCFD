import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const balanceRequestHandler = async (parsedMessage: any) => {
    try {
        console.log("Received balance-request message:", parsedMessage);
        const user_id = parsedMessage.user_id || parsedMessage.userId;
        const balanceRecord = await prisma.balance.findUnique({ where: { userId: user_id } });
        console.log("Balance record from DB for user:", user_id, balanceRecord);
        const balance = balanceRecord ? Number(balanceRecord.amount) : 0;

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