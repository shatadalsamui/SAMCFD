// OTP Verification Controller
// This file handles the verification of OTPs during the signup process. It validates user input, verifies OTPs from Redis, communicates with Kafka for user creation, and generates JWT tokens for authentication.

import type { Request, Response } from "express";
import { VerifyOtpSchema } from "@repo/schemas";
import { getOtp, deleteOtp } from "../../cache/otpCache";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import bcrypt from "bcrypt";
import jwt from "jsonwebtoken";

export const verifyOtp = async (req: Request, res: Response) => {
    try {
        //step 1 input validation 
        const { success, data, error } = VerifyOtpSchema.safeParse(req.body);
        if (!success) {
            return res.status(411).json({ message: "Validation failed", errors: error.issues });
        }

        const { email, otp, ...userData } = data;

        //step 2 retrieve and verify otp from redis in memory cache 
        const storedHashedOtp = await getOtp(email);

        if (!storedHashedOtp) {
            return res.status(401).json({ message: "Invalid or expired Otp" });
        }

        const isMatch = await bcrypt.compare(otp, storedHashedOtp);
        if (!isMatch) {
            return res.status(401).json({ message: "Wrong otp Entered." });
        }

        // Only delete OTP after successful verification
        await deleteOtp(email);

        // Step 2.5: Hash the password before sending to Kafka (SECURITY)
        const hashedPassword = await bcrypt.hash(userData.password, 12);

        //step 3 publish user creation request to kafka and wait for the response 
        let userCreationResponse;
        try {
            userCreationResponse = await kafkaRequestResponse(
                "user-creation-request",//request topic (fixed: removed 's')
                "user-creation-response",//response topic 
                { email, ...userData, password: hashedPassword }
            );
        } catch (kafkaError) {
            console.error("Kafka Error during user creation: ", kafkaError)
            return res.status(500).json({ message: "Failed to process user creation." });
        }

        //step 4 handle the response from db-processor
        if (!userCreationResponse.success) {
            return res.status(500).json({ message: userCreationResponse.message });
        }

        // Step 5: Generate JWT
        if (!process.env.JWT_SECRET) {
            console.error("JWT_SECRET is not defined in environment variables.");
            return res.status(500).json({ message: "Internal server error." });
        }
        const token = jwt.sign(
            { email },
            process.env.JWT_SECRET!,
            { expiresIn: "1h" }
        );

        // Step 6: Set JWT as HTTP-only cookie
        res.cookie("auth_token", token, {
            httpOnly: true,
            secure: process.env.NODE_ENV === "production", // Secure in production
            sameSite: "strict",
        });

        // Step 7: Respond with success
        return res.status(200).json({
            message: "User created successfully.",
        });
    } catch (err) {
        console.error("OTP Verification Error:", err);
        return res.status(500).json({ message: "Internal server error." });
    }
}