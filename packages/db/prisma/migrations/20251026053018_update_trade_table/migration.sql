/*
  Warnings:

  - You are about to drop the column `type` on the `Trade` table. All the data in the column will be lost.
  - The `status` column on the `Trade` table would be dropped and recreated. This will lead to data loss if there is data in the column.
  - Added the required column `side` to the `Trade` table without a default value. This is not possible if the table is not empty.

*/
-- CreateEnum
CREATE TYPE "Side" AS ENUM ('BUY', 'SELL');

-- CreateEnum
CREATE TYPE "OrderType" AS ENUM ('MARKET', 'LIMIT');

-- CreateEnum
CREATE TYPE "TradeTerm" AS ENUM ('INTRAHOUR', 'INTRADAY', 'WEEK', 'MONTH', 'YEAR');

-- CreateEnum
CREATE TYPE "TimeInForce" AS ENUM ('IOC', 'FOK', 'DAY', 'GTC', 'EXPIRE_AT');

-- CreateEnum
CREATE TYPE "TradeStatus" AS ENUM ('OPEN', 'MATCHED', 'FILLED', 'CLOSED', 'LIQUIDATED', 'CANCELLED');

-- AlterTable
ALTER TABLE "Trade" DROP COLUMN "type",
ADD COLUMN     "closePrice" INTEGER,
ADD COLUMN     "expiryAt" TIMESTAMP(3),
ADD COLUMN     "limitPrice" INTEGER,
ADD COLUMN     "orderType" "OrderType" NOT NULL DEFAULT 'MARKET',
ADD COLUMN     "pnl" INTEGER,
ADD COLUMN     "side" "Side" NOT NULL,
ADD COLUMN     "stopLossPercent" DOUBLE PRECISION,
ADD COLUMN     "stopLossPrice" INTEGER,
ADD COLUMN     "takeProfitPercent" DOUBLE PRECISION,
ADD COLUMN     "takeProfitPrice" INTEGER,
ADD COLUMN     "timeInForce" "TimeInForce",
ADD COLUMN     "tradeTerm" "TradeTerm",
DROP COLUMN "status",
ADD COLUMN     "status" "TradeStatus" NOT NULL DEFAULT 'OPEN';
