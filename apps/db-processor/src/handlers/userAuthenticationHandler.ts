import { producer } from "@repo/kafka";
import { db as prisma } from "@repo/db";

export const userAuthenticationHandler = async (message: any) => {
    const { email, correlationId } = message;

    try {

        const user = await prisma.user.findUnique({
            where: { email },
        });

        let response;
        if (!user) {
            response = {
                success: false,
                message: "user not found",
                correlationId,
            };
        } else {
            response = {
                id: user.id,
                hashedPassword: user.password,
                correlationId,
            };
        }

        await producer.send({
            topic: "user-authentication-response",
            messages: [{ key: correlationId, value: JSON.stringify(response) }],
        })
    } catch (error) {
        console.error("Error in userAuthenticationHandler:", error);
        // Send error response
        const errorResponse = {
            success: false,
            message: "Internal server error",
            correlationId,
        };
        await producer.send({
            topic: "user-authentication-response",
            messages: [{ key: correlationId, value: JSON.stringify(errorResponse) }],
        });
    }
}