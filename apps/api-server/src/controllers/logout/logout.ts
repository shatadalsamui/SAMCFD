import type { Request, Response } from "express";
import { redisClient } from "@repo/redis";

export const logout = async (req: Request, res: Response) => {
    const token = req.cookies.auth_token;
    if (token) await redisClient.del(`jwt:${token}`);
    res.clearCookie("auth_token");
    res.json({ message: "Logged out" });
};