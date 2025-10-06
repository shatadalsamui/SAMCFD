import dotenv from "dotenv";
dotenv.config();

import { createClient } from "redis";

const redisClient = createClient({
    url: process.env.REDIS_URL, // Use the full Redis URL
});

// Add event listeners
redisClient.on("connect", () => {
    console.log("Redis client connected successfully");
});

redisClient.on("ready", () => {
    console.log("Redis client is ready to accept commands");
});

redisClient.on("error", (err) => {
    console.error("Redis Client Error:", err);
});

redisClient.on("end", () => {
    console.log("Redis client disconnected");
});

// Health check function
export const checkRedisHealth = async () => {
    try {
        await redisClient.ping();
        console.log("Redis is healthy");
    } catch (err) {
        console.error("Redis health check failed:", err);
    }
};

export default redisClient;