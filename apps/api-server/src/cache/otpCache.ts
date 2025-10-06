// OTP Cache Utility
// Provides functions to store, retrieve, and delete OTPs in Redis with hashing for security.

import { redisClient } from "@repo/redis";
import bcrypt from "bcrypt";

//check redis if redis is working 
async function ensureRedisConnection() {

    if (!redisClient.isOpen) {

        await redisClient.connect();
    }
}

//store the otp sent to user
export async function storeOtp(email: string, otp: string): Promise<void> {

    await ensureRedisConnection();

    const hashedotp = await bcrypt.hash(otp, 10);

    await redisClient.set(`otp:${email}`, hashedotp, { EX: 600 });
}

//get the otp from user 
export async function getOtp(email: string): Promise<string | null> {

    await ensureRedisConnection();

    return await redisClient.get(`otp:${email}`);

}

//delete the otp 
export async function deleteOtp(email: string): Promise<void> {

    await ensureRedisConnection();

    await redisClient.del(`otp:${email}`);
}
