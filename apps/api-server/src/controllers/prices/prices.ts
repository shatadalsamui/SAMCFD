import WebSocket, { WebSocketServer } from "ws";
import dotenv from "dotenv";
import { redisClient } from "@repo/redis";
import { setPriceUpdateCallback } from "../../kafka/kafkaFireAndForget";
import type { Request, Response } from "express";

dotenv.config();

const WS_PORT = parseInt(process.env.WS_PORT as string, 10);
const wss = new WebSocketServer({ port: WS_PORT });

console.log(`WebSocket server running on port ${WS_PORT}`);

const clients = new Set<WebSocket>();

wss.on('connection', (ws: WebSocket) => {
    console.log('New WebSocket client connected');
    clients.add(ws);

    ws.on('close', () => {
        console.log('WebSocket client disconnected');
        clients.delete(ws);
    });

    ws.on('error', (err) => {
        console.error('WebSocket error:', err);
        clients.delete(ws);
    });
});

const broadcast = (data: any) => {
    const message = JSON.stringify(data);
    clients.forEach((client) => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(message);
        }
    });
};

// Set the callback to broadcast and cache
setPriceUpdateCallback((data) => {
    broadcast(data);
    redisClient.set(data.asset, JSON.stringify(data)).catch(console.error);
});

export const getLatestPrices = async (req: Request, res: Response) => {
    try {
        const assets = ["SOL_USDC", "BTC_USDC", "ETH_USDC", "DOGE_USDC", "BNB_USDC"];
        const prices: Record<string, any> = {};
        for (const asset of assets) {
            const value = await redisClient.get(asset);
            if (value) {
                prices[asset] = JSON.parse(value);
            }
        }
        res.json(prices);
    } catch (error) {
        console.error('Error fetching prices:', error);
        res.status(500).json({ error: 'Failed to fetch prices' });
    }
};

export { wss };