import { db as prisma } from '@repo/db';

async function main() {
    // Find the user by email
    const user = await prisma.user.findUnique({
        where: { email: 'shatadalsamui82@gmail.com' },
    });

    if (!user) {
        console.error('User not found!');
        process.exit(1);
    }

    // Upsert holdings for BTC_USDC
    await prisma.holdings.upsert({
        where: { userId_asset: { userId: user.id, asset: 'BTC_USDC' } },
        update: { quantity: 1.0 },
        create: { userId: user.id, asset: 'BTC_USDC', quantity: 1.0 }
    });

    console.log(`Holdings set for user (${user.email}) on BTC_USDC: 1.0`);
}

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(async () => {
        await prisma.$disconnect();
    });