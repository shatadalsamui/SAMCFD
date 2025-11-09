import { db as prisma } from '@repo/db';

async function main() {
    // Find both users by email
    const emails = ['shatadalsamui82@gmail.com', 'shatadalsamuimain@gmail.com'];
    for (const email of emails) {
        const user = await prisma.user.findUnique({ where: { email } });
        if (!user) {
            console.error(`User not found: ${email}`);
            continue;
        }
        // Upsert holdings for BTC_USDC
        await prisma.holdings.upsert({
            where: { userId_asset: { userId: user.id, asset: 'BTC_USDC' } },
            update: { quantity: BigInt(10) },
            create: { userId: user.id, asset: 'BTC_USDC', quantity: BigInt(10) }
        });
        console.log(`Holdings set for user (${user.email}) on BTC_USDC: 1`);
    }
}

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(async () => {
        await prisma.$disconnect();
    });