import { createConsumer } from "@repo/kafka";
import { PriceUpdateSchema } from "@repo/schemas";
import dotenv from "dotenv";

dotenv.config();


const fireForgetConsumer = createConsumer('samcfd-fire-forget');

// Callback for handling received data (set by controller)
let onPriceUpdate: (data: any) => void = () => { };

// Function to set the callback
export const setPriceUpdateCallback = (callback: (data: any) => void) => {
    onPriceUpdate = callback;
};


// Set up the consumer for fire-and-forget topics
const setupFireForgetConsumer = async () => {
    await fireForgetConsumer.connect();
    await fireForgetConsumer.subscribe({ topic: 'price-updates', fromBeginning: false });

    fireForgetConsumer.run({
        eachMessage: async ({ message }) => {
            if (!message.value) return;

            try {
                const parsed = JSON.parse(message.value.toString());
                const validation = PriceUpdateSchema.safeParse(parsed);

                if (validation.success) {
                    //console.log('Received price update:', validation.data);
                    onPriceUpdate(validation.data);  // Call the callback
                } else {
                    console.error('Invalid price update schema:', validation.error);
                }
            } catch (err) {
                console.error('Error parsing Kafka message:', err);
            }
        },
    });
};

setupFireForgetConsumer().catch(console.error);