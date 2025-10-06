import redisClient, { checkRedisHealth } from "./client";

const testRedis = async () => {
    try {
        await redisClient.connect();
        console.log("Redis connection successful");

        // Perform a health check
        await checkRedisHealth();

        // Test setting and getting a key
        await redisClient.set("test_key", "test_value");
        const value = await redisClient.get("test_key");
        console.log("Value from Redis:", value);
    } catch (err) {
        console.error("Redis test error:", err);
    } finally {
        await redisClient.disconnect();
    }
};

testRedis();