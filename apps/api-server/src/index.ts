// API Server Entry Point
// Initializes the Express server, sets up middleware, and defines API routes.

import express from "express";
import v1Router from "./routes/v1"
import cors from "cors";
import { redisClient } from "@repo/redis";

const app = express();

app.use(cors());
app.use(express.json());
app.use("/api/v1", v1Router);

// Test Redis Route
app.get("/api/v1/test-redis", async (req, res) => {
  try {

    if (!redisClient.isOpen) {
      await redisClient.connect(); // Add this check
    }
    // Set a test key-value pair in Redis
    await redisClient.set("testKey", "testValue");

    // Retrieve the value from Redis
    const value = await redisClient.get("testKey");

    // Delete the test key
    await redisClient.del("testKey");

    // Respond with the retrieved value
    res.status(200).json({ message: "Redis Test Successful", value });
  } catch (err: any) {
    console.error("Redis Test Error:", err);
    res.status(500).json({ message: "Redis Test Failed", error: err.message });
  }
});

const PORT = process.env.PORT || 3001;

app.listen(PORT, () => {
  console.log(`API Server running on port ${PORT}`); // Add this for logging
});

