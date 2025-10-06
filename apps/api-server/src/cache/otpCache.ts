import { redisClient } from "@repo/redis";
import bcrypt from "bcrypt";

async function ensureRedisConnection() {

    if (!redisClient.isOpen) {
        
        await redisClient.connect();
    }
}

export async function storeOtp(email: string, otp: string): Promise<void> {

    await ensureRedisConnection();

    const hashedotp = await bcrypt.hash(otp, 10);

    await redisClient.set(`otp:${email}`, hashedotp, { EX: 600 });
}

export async function getOtp(email: string): Promise<string | null> {

    await ensureRedisConnection();

    return await redisClient.get(`otp:${email}`);

}

export async function deleteOtp(email: string): Promise<void> {

    await ensureRedisConnection();

    await redisClient.del(`otp:${email}`);
}
