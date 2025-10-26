-- DropIndex
DROP INDEX "public"."Balance_userId_idx";

-- CreateTable
CREATE TABLE "Holdings" (
    "id" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "asset" TEXT NOT NULL,
    "quantity" DECIMAL(65,30) NOT NULL,

    CONSTRAINT "Holdings_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE INDEX "Holdings_userId_idx" ON "Holdings"("userId");

-- CreateIndex
CREATE UNIQUE INDEX "Holdings_userId_asset_key" ON "Holdings"("userId", "asset");

-- CreateIndex
CREATE INDEX "Trade_asset_status_idx" ON "Trade"("asset", "status");

-- AddForeignKey
ALTER TABLE "Holdings" ADD CONSTRAINT "Holdings_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
