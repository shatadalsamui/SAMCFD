import dotenv from 'dotenv';
dotenv.config();

export const BACKPACK_WS_URL = process.env.BACKPACK_WS_URL || 'wss://ws.backpack.exchange/';
export const ASSETS = ["SOL_USDC", "BTC_USDC", "ETH_USDC", "DOGE_USDC", "BNB_USDC"];