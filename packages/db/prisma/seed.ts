import {db as prisma} from '@repo/db';

async function main() {
    await prisma.user.create({
        data: {
            email: 'test@example.com',
            password: 'hashedpassword',
            name: 'test',
        },
    });
}

main()
    .catch((e) => {
        console.error(e);
        process.exit(1);
    })
    .finally(async () => {
        await prisma.$disconnect();
    });