// API Server Entry Point
// Initializes the Express server, sets up middleware, and defines API routes.

import express from "express";
import v1Router from "./routes/v1"
import cors from "cors";
import cookieParser from "cookie-parser";
import { redisClient } from "@repo/redis";


const app = express();

app.use(cors());
app.use(cookieParser());
app.use(express.json());
app.use("/api/v1", v1Router);

const PORT = process.env.PORT || 3001;

(async () => {
  try {
    // Connect to Redis
    await redisClient.connect();
    console.log("Redis connected");

    app.listen(PORT, () => {
      console.log(`API Server running on port ${PORT}`);
    });
  } catch (err) {
    console.error("Failed to start server:", err);
    process.exit(1);
  }
})();

// Graceful shutdown
process.on('SIGINT', async () => {
  console.log('Shutting down server...');
  await redisClient.disconnect();
  process.exit(0);
});

