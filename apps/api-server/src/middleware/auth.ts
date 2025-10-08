import type { Request, Response, NextFunction } from "express";
import jwt from "jsonwebtoken";
import { redisClient } from "@repo/redis";

export const requireAuth = async (req: Request, res: Response, next: NextFunction) => {
    const token = req.cookies.auth_token;
    if (!token) return res.status(401).json({ message: "No token" });

    try {
        // Verify JWT
        const decoded = jwt.verify(token, process.env.JWT_SECRET!) as { id: string, email: string };

        // Check Redis
        const stored = await redisClient.get(`jwt:${token}`);
        if (!stored) return res.status(401).json({ message: "Invalid token" });

        (req as any).user = decoded;
        next();
    } catch (err) {
        res.status(401).json({ message: "Invalid token" });
    }
};