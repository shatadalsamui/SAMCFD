/*
  Warnings:

  - Added the required column `slippage` to the `Trade` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "Trade" ADD COLUMN     "slippage" INTEGER NOT NULL;
