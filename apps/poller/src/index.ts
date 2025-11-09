import WebSocket from 'ws';
import { BACKPACK_WS_URL, ASSETS } from './config';
import Big from 'big.js';
import { producer } from '@repo/kafka';
import { PriceUpdateSchema } from '@repo/schemas';

let reconnectDelay = 1000;

const publishPrice = (symbol: string, midPriceCents: number) => {
    const payload = { asset: symbol, price: midPriceCents, timestamp: Date.now() };
    const validationResult = PriceUpdateSchema.safeParse(payload);
    if (validationResult.success) {
        producer.send({
            topic: 'price-updates',
            messages: [{ value: JSON.stringify(payload) }],
        }).catch(console.error);
    } else {
        console.error("Invalid schema:", validationResult.error);
    }
};

// A simple function to manage the connection
const connect = () => {
    // Create a new WebSocket client instance
    const ws = new WebSocket(BACKPACK_WS_URL);
    producer.connect().catch(console.error);

    // 1. Handle the 'open' event: This runs when the connection is successful
    ws.on('open', () => {
        console.log('✅ Connected to Backpack WebSocket.');
        reconnectDelay = 1000;

        // Define the subscription message for our assets
        const subscriptionMessage = {
            method: 'SUBSCRIBE',
            params: ASSETS.map(asset => `bookTicker.${asset}`),
        };

        // Send the message to the server to start receiving price data
        ws.send(JSON.stringify(subscriptionMessage));
    });

    // 2. Handle the 'message' event: This runs for every new piece of data received
    ws.on('message', (data: string) => {
        const parsedMsg = JSON.parse(data);
        if (parsedMsg?.data?.e === 'bookTicker') {
            const symbol = parsedMsg.data.s; // e.g., "SOL_USDC"
            const ask = new Big(parsedMsg.data.a);
            const bid = new Big(parsedMsg.data.b);
            const midPriceCents = ask.plus(bid).div(2).times(100).round(0, Big.roundHalfUp).toNumber();

            // Update the price variables and log
            switch (symbol) {
                case "SOL_USDC":
                    //console.log(`UPDATE ==> SOL Midpoint Price: $${midPrice}`);
                    publishPrice(symbol, midPriceCents);
                    break;
                case "BTC_USDC":
                    //console.log(`UPDATE ==> BTC Midpoint Price: $${midPrice}`);
                    publishPrice(symbol, midPriceCents);
                    break;
                case "ETH_USDC":
                    //console.log(`UPDATE ==> ETH Midpoint Price: $${midPrice}`);
                    publishPrice(symbol, midPriceCents);
                    break;
                case "DOGE_USDC":
                    //console.log(`UPDATE ==> DOGE Midpoint Price: $${midPrice}`);
                    publishPrice(symbol, midPriceCents);
                    break;
                case "BNB_USDC":
                    //console.log(`UPDATE ==> BNB Midpoint Price: $${midPrice}`);
                    publishPrice(symbol, midPriceCents);
                    break;
            }
        }
    });

    // 3. Handle the 'close' event: This runs if the connection is lost
    ws.on('close', () => {
        console.error(`❌ Disconnected from Backpack. Reconnecting in ${reconnectDelay / 1000} seconds...`);
        setTimeout(connect, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, 60000); // Double delay, max 1 minute
    });

    // 4. Handle errors
    ws.on('error', (err) => {
        console.error('WebSocket Error:', err.message);
    });
};

// Start the connection process
connect();