
# Balance & Holdings Logic Issues

## 1. Incorrect Margin Deduction for Spot Sells

Currently, margin is deducted for all sell orders, even when the user already has sufficient holdings (spot sell). In a real exchange, margin should only be deducted for leveraged/short positions, not for spot sells. For spot sells, the user's holdings should decrease and their balance should increase by the sale proceeds, with no margin deduction.

### Impact

- User's balance is reduced unnecessarily for spot sells.
- This does not match real exchange logic and may confuse users.

### Solution

- Only deduct margin for leveraged/short trades (when user does not have enough holdings).
- For spot sells (user has enough holdings), do not deduct margin; just decrease holdings and increase balance by sale proceeds.

---

## 2. Holdings Not Updated After Trade

After a SELL, the seller’s holdings remain unchanged (should decrease by the sold quantity). After a BUY, the buyer’s holdings remain unchanged (should increase by the bought quantity).

### Impact

- Users see incorrect asset balances after trades.
- Trade outcome messages may be misleading.

### Solution

- Decrease seller’s holdings by sold quantity after SELL.
- Increase buyer’s holdings by bought quantity after BUY.

---

## 3. Balance Not Updated by Trade Proceeds

After a SELL, the seller’s balance only reflects margin deduction, not the sale proceeds (should increase by quantity × price for spot sells). After a BUY, the buyer’s balance only reflects margin deduction, not the purchase cost (should decrease by quantity × price for spot buys).

### Impact

- Users see incorrect balances after trades.
- Trade outcome messages may be misleading.

### Solution

- For SELL: increase balance by sale proceeds (quantity × price).
- For BUY: decrease balance by purchase cost (quantity × price).

---

## 4. Trade Outcome Message Accuracy

The `updatedHoldings` and `updatedBalance` fields in the trade outcome message do not reflect the correct post-trade state if holdings and balances are not updated as above.

### Impact

- Downstream systems and users may see misleading trade results.

### Solution

- Ensure trade outcome messages reflect the true post-trade state, including correct balances and holdings.
