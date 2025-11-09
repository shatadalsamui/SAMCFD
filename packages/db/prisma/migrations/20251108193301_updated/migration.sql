/*
  Warnings:

  - You are about to alter the column `amount` on the `Balance` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `quantity` on the `Holdings` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `margin` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `entryPrice` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `closePrice` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `limitPrice` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `pnl` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `stopLossPercent` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `DoublePrecision` to `Integer`.
  - You are about to alter the column `stopLossPrice` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `takeProfitPercent` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `DoublePrecision` to `Integer`.
  - You are about to alter the column `takeProfitPrice` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.
  - You are about to alter the column `quantity` on the `Trade` table. The data in that column could be lost. The data in that column will be cast from `Decimal(65,30)` to `BigInt`.

*/
-- AlterTable
ALTER TABLE "Balance" ALTER COLUMN "amount" SET DATA TYPE BIGINT;

-- AlterTable
ALTER TABLE "Holdings" ALTER COLUMN "quantity" SET DATA TYPE BIGINT;

-- AlterTable
ALTER TABLE "Trade" ALTER COLUMN "margin" SET DATA TYPE BIGINT,
ALTER COLUMN "entryPrice" SET DATA TYPE BIGINT,
ALTER COLUMN "closePrice" SET DATA TYPE BIGINT,
ALTER COLUMN "limitPrice" SET DATA TYPE BIGINT,
ALTER COLUMN "pnl" SET DATA TYPE BIGINT,
ALTER COLUMN "stopLossPercent" SET DATA TYPE INTEGER,
ALTER COLUMN "stopLossPrice" SET DATA TYPE BIGINT,
ALTER COLUMN "takeProfitPercent" SET DATA TYPE INTEGER,
ALTER COLUMN "takeProfitPrice" SET DATA TYPE BIGINT,
ALTER COLUMN "quantity" SET DATA TYPE BIGINT;
