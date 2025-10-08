import type { Request, Response } from "express";
import { SigninSchema } from "@repo/schemas";
import { kafkaRequestResponse } from "../../kafka/kafkaRequestResponse";
import jwt from "jsonwebtoken";
import bcrypt from "bcrypt";
import { redisClient } from "@repo/redis";

export const signIn = async (req: Request, res: Response) => {
    try {
        // Step 1: Validate input
        const { success, data, error } = SigninSchema.safeParse(req.body);

        if (!success) {
            return res.status(411).json({
                message: "Validation failed",
                errors: error.issues,
            });
        }

        const { email, password } = data;

        // Step 2: Publish Kafka topic for user authentication
        let authResponse;
        try {
            authResponse = await kafkaRequestResponse(
                "user-authentication-request", // Request topic
                "user-authentication-response", // Response topic
                { email } // Message payload
            );
        } catch (kafkaError) {
            console.error("Kafka Error during user authentication:", kafkaError);
            return res.status(500).json({ message: "Failed to authenticate user." });
        }

        // Step 3: Handle authentication response
        if (!authResponse.hashedPassword) {
            return res.status(401).json({ message: authResponse.message || "Authentication failed" });
        }

        // Step 3.1: Compare with bcrypt.compare
        const isPasswordValid = await bcrypt.compare(password, authResponse.hashedPassword);
        if (!isPasswordValid) {
            return res.status(401).json({ message: "Invalid password" });
        }

        // Step 4: Generate JWT
        if (!process.env.JWT_SECRET) {
            console.error("JWT_SECRET is not defined in environment variables.");
            return res.status(500).json({ message: "Internal server error." });
        }

        const token = jwt.sign(
            { email },
            process.env.JWT_SECRET!,
            { expiresIn: "1h" }
        );

        // Step 5: Set JWT as HTTP-only cookie
        res.cookie("auth_token", token, {
            httpOnly: true,
            secure: process.env.NODE_ENV === "production", // Secure in production
            sameSite: "strict",
        });

        //step 5.1 : store the email and token in redis for session management
        await redisClient.set(`jwt:${token}`, JSON.stringify({ email }), { EX: 3600 });
        // Step 6: Respond with success
        return res.status(200).json({
            message: "Signed in successfully.",
        });
    } catch (err) {
        console.error("Signin Error:", err);
        return res.status(500).json({ message: "Internal server error." });
    }
};