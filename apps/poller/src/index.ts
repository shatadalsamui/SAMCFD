import WebSocket from 'ws';

// The URL you provided
const BACKPACK_WS_URL = 'wss://ws.backpack.exchange/';

let solPrice: string | null = null;
let btcPrice: string | null = null;
let ethPrice: string | null = null;
let dogePrice: string | null = null;
let bnbPrice: string | null = null;
// A simple function to manage the connection
const connect = () => {
    // Create a new WebSocket client instance
    const ws = new WebSocket(BACKPACK_WS_URL);

    // 1. Handle the 'open' event: This runs when the connection is successful
    ws.on('open', () => {
        console.log('✅ Connected to Backpack WebSocket.');

        // Define the subscription message for our assets
        const subscriptionMessage = {
            method: 'SUBSCRIBE',
            params: ["ticker.SOL_USDC", "ticker.BTC_USDC", "ticker.ETH_USDC", "ticker.DOGE_USDC", "ticker.BNB_USDC"],
        };

        // Send the message to the server to start receiving price data
        ws.send(JSON.stringify(subscriptionMessage));
    });

    // 2. Handle the 'message' event: This runs for every new piece of data received
    ws.on('message', (data: string) => {
        const message = JSON.parse(data);

        // The ticker data is usually inside a 'data' object
        switch (message.stream) {
            case "ticker.SOL_USDC":
                solPrice = message.data.c;
                console.log(`UPDATE ==> SOL Price: $${solPrice}`);
                break;
            case "ticker.BTC_USDC":
                btcPrice = message.data.c;
                console.log(`UPDATE ==> BTC Price: $${btcPrice}`);
                break;
            case "ticker.ETH_USDC":
                ethPrice = message.data.c;
                console.log(`UPDATE ==> ETH Price: $${ethPrice}`);
                break;
            case "ticker.DOGE_USDC":
                dogePrice = message.data.c;
                console.log(`UPDATE ==> DOGE Price: $${dogePrice}`);
                break;
            case "ticker.BNB_USDC":
                bnbPrice = message.data.c;
                console.log(`UPDATE ==> BNB Price: $${bnbPrice}`);
                console.log("<------------------------------------->")
                break;
            default:
                // Ignore other streams
                break;
        }
    });

    // 3. Handle the 'close' event: This runs if the connection is lost
    ws.on('close', () => {
        console.error('❌ Disconnected from Backpack. Reconnecting in 5 seconds...');
        // Automatically try to reconnect after a delay
        setTimeout(connect, 5000);
    });

    // 4. Handle errors
    ws.on('error', (err) => {
        console.error('WebSocket Error:', err.message);
    });
};

// Start the connection process
connect();