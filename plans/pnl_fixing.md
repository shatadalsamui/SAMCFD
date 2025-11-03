# PnL Calculation Fix Plan

## Step 1: Store Entry and Close Price Properly
- When a position is opened, set and store `entry_price` in the `Trade` struct.
- When the position is closed, set `close_price` in the `Trade` struct.

## Step 2: Update PnL Calculation
- Refactor the PnL calculation to use `entry_price` and `close_price` from the `Trade` struct, not the `Order`'s `price`.

## Step 3: Update Trade Closing Logic
- On trade close (market close, stop-loss, take-profit, or liquidation), set `close_price` and call the new PnL calculation.
- Log and credit/debit the calculated PnL to the user's balance.

## Step 4: Test the Workflow
- Open a position (buy/sell).
- Close the position at a different price.
- Verify that PnL is non-zero and correctly logged/credited.