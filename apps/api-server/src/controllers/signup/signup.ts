import type { Request, Response } from "express";
import { SignupSchema } from "@repo/schemas";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import { storeOtp } from "../../cache/otpCache.ts";
import { sendOtpMail } from "../../services/mail/otpMail";
import crypto from "crypto";

export const signUp = async (req: Request, res: Response) => {
    try {
        const { success, data, error } = SignupSchema.safeParse(req.body);

        if (!success) {
            return res.status(411).json({
                message: "Validation failed",
                errors: error.issues,
            });
        }
        const { email } = data;

        //step 1 check if user already exists 
        let userExists = false;
        try {
            const response = await kafkaRequestResponse(
                "user-existence-check", // Request topic
                "user-existence-response", // Response topic
                { email } // Message payload
            );
            userExists = response.exists;
        } catch (kafkaError) {
            console.error("Kafka Error during user existence check:", kafkaError);
            return res.status(500).json({ message: "Failed to verify user existence." });
        }

        if (userExists) {
            return res.status(409).json({ message: "User already exists." });
        }

        //step 2 generate otp
        const otp = crypto.randomInt(100000, 999999).toString();

        //step 3 store otp in redis
        await storeOtp(email, otp);

        //step 4 send otp to the users mail 
        await sendOtpMail(email, otp);

        res.status(200).json({ message: "OTP sent to your email." });

    } catch (err: any) {

        console.error("Signup Error:", err);
        res.status(500).json({ message: "Internal server error." });
    }
}