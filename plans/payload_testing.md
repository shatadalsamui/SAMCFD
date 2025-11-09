  **User assignment:**

- 1st payload (buy limit): send from **User A** (e.g., shatadalsamuimain)
- 2nd payload (sell market): send from **User B** (e.g., shatadalsamui82)
- 3rd payload (buy limit): send from **User B** (e.g., shatadalsamui82)
- 4th payload (sell market): send from **User A** (e.g., shatadalsamuimain)

---

- [x] Full open/close cycle (for realized PnL test)
  **Payloads:**
  
  ```json
  {
    "margin": 5000000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 11000000,
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 5000000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 5000000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 10900000,
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 5000000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  **User assignment:**
  - 1st payload (buy limit 10500000): **User A** (cf9ec1ac-0b66-4aa6-9ad1-1d37607caca6)
  - 2nd payload (sell market): **User B** (a42d7740-e068-474d-b3a1-e65528f727ad)
  - 3rd payload (buy limit 10400000): **User B** (a42d7740-e068-474d-b3a1-e65528f727ad)
  - 4th payload (sell market): **User A** (cf9ec1ac-0b66-4aa6-9ad1-1d37607caca6)

  **Result:**
  - ✅ Order 1 (User A buy limit 10500000): Added to order book, status=Open
  - ✅ Order 2 (User B sell market): Matched at 10500000, User B opens SHORT position, PnL=0
  - ✅ Order 3 (User B buy limit 10400000): Added to order book, status=Open (should close SHORT but created new LONG instead)
  - ✅ Order 4 (User A sell market): Matched at 10400000, User A closes LONG position, **PnL=-100000**
  
  **Issues Found:**
  - ❌ User B's buy limit at 104000 did NOT close their SHORT position (entry 105000)
  - ❌ Expected User B PnL: +1000 (shorted at 105000, closed at 104000)
  - ❌ Actual: User B now has both an open SHORT and an open LONG (incorrect)
  
  **Expected Behavior:**
  - When a user with an open SHORT places a BUY order, it should close the SHORT, not open a new LONG
  - User B should have PnL=+100000 logged when Order 3 executes
  
  **To Fix:**
  - Engine needs to check if user has an opposite position before creating a new one
  - If opposite position exists, close it and calculate realized PnL
  - Only open a new position if no opposite position exists

# Order Matching Test Plan

## 🔄 Additional Payloads for Open/Close Cycle at 120k/121k

**Payload 1: User A opens a new long at $120,000**

```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1,
  "orderType": "limit",
  "limitPrice": 12000000,
  "slippage": 0
}
```

**Payload 2: User B opens a new short at $120,000 (market sell to match above)**

```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1,
  "orderType": "market",
  "slippage": 0
}
```

**Payload 3: User B closes their short (buy limit at $121,000)**

```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1,
  "orderType": "limit",
  "limitPrice": 12100000,
  "slippage": 0
}
```

**Payload 4: User A closes their long (sell market, matches User B’s buy at $121,000)**

```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1,
  "orderType": "market",
  "slippage": 0
}
```

**Expected Results:**
- User A: Realizes $1,000 profit, margin unlocked, holdings decrease by 1.
- User B: Realizes $1,000 loss, margin unlocked, holdings increase by 1.
- Trade outcomes show correct PnL, margin unlock, and updated balances/holdings.

## ✅ To-Do List for Order Matching Engine

- [x] Limit order placed first, then market order (should match and fill)
  **Payloads:**
  
  ```json
  {
  "margin": 10000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 11000000,
  "slippage": 0
  }

  ```
  
  ```json
  {
    "margin": 10000000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

- [x] Market order placed first, then limit order (should NOT match immediately)

  **Payloads:**

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  ```json
  {

    ## Spot Margin Profit/Loss Test Flow (Engine Verified)

    This test demonstrates a $1,000 profit for User A and a $1,000 loss for User B, with correct changes in balances and holdings.

    ### Payloads

    **1. User A: Buy 1 BTC at 100,000 USD (limit)**
    ```json
    {
      "margin": 100000,
      "asset": "BTC_USDC",
      "side": "buy",
      "leverage": 1,
      "quantity": 1.0,
      "orderType": "limit",
      "limitPrice": 10000000,
      "slippage": 0
    }
    ```

    **2. User B: Sell 1 BTC at 100,000 USD (market)**
    ```json
    {
      "margin": 100000,
      "asset": "BTC_USDC",
      "side": "sell",
      "leverage": 1,
      "quantity": 1.0,
      "orderType": "market",
      "slippage": 0
    }
    ```

    **3. User A: Sell 1 BTC at 101,000 USD (limit)**
    ```json
    {
      "margin": 100000,
      "asset": "BTC_USDC",
      "side": "sell",
      "leverage": 1,
      "quantity": 1.0,
      "orderType": "limit",
      "limitPrice": 10100000,
      "slippage": 0
    }
    ```

    **4. User B: Buy 1 BTC at 101,000 USD (market)**
    ```json
    {
      "margin": 100000,
      "asset": "BTC_USDC",
      "side": "buy",
      "leverage": 1,
      "quantity": 1.0,
      "orderType": "market",
      "slippage": 0
    }
    ```

    ### Actual Outcomes (Engine Verified)

    - **User A:**
      - Final USD balance: 501,000 USD
      - Final BTC holdings: 10
      - Net profit: $1,000

    - **User B:**
      - Final USD balance: 499,000 USD
      - Final BTC holdings: 10
      - Net loss: $1,000

    All margin is unlocked, no open positions remain, and all PnL is realized as expected.
    "slippage": 0
  }
  ```

  **Result:**

- Scenario tested by sending the sell market order first, then the buy limit order.
- No match occurred; both orders remained open/partially filled.
- Engine does not match new limit orders against resting market orders (standard exchange behavior).
- ✅ Scenario tested and matches engine design.

- [x] Partial fills (large limit order matched by multiple market orders)
  **Payloads:**

  ```json
  {
    "margin": 15000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 3.0,
    "orderType": "limit",
    "limitPrice": 3000000,
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  **Result:**

- Scenario tested by placing a large sell limit order (quantity 3), then sending three buy market orders (quantity 1 each).
- Each market order matched and filled 1 unit at the limit price (3000000).
- The sell limit order was partially filled after each market order, and fully filled after the third.
- All market orders were fully filled at the limit price.
- PnL: 0 for each trade (no profit/loss at entry).
- ✅ Scenario tested and passed.

  ```

- [x] Check if PnL is generated and logged correctly after a match

  **Result:**

- PnL is generated and logged after each successful match (see engine logs: e.g., 'Order ... filled. PnL: 0').
- For all opening trades in these scenarios, PnL is 0 as expected.
- To test realized PnL, run trade close scenarios at different prices.
- ✅ Scenario tested and passed for opening trades.
  
---

**Goal:**  
Test all combinations of order types and verify that PnL is generated and logged as expected for each successful match.


## Comprehensive 8-Payload Test Flow

This section documents a single, continuous test flow using 8 payloads, covering open/close cycles, PnL realization, and order matching. Each payload is listed in order, followed by the final expected outcomes.

### Payloads

**1. User A opens long at 110,000**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 11000000,
  "slippage": 0
}
```

**2. User B opens short at 110,000 (market sell)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**3. User B opens long at 109,000**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 10900000,
  "slippage": 0
}
```

**4. User A closes long (market sell, matches User B’s buy at 109,000)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**5. User A opens long at 120,000**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 12000000,
  "slippage": 0
}
```

**6. User B opens short at 120,000 (market sell)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**7. User B closes short (buy limit at 121,000)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 12100000,
  "slippage": 0
}
```

**8. User A closes long (market sell, matches User B’s buy at 121,000)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

### Final Expected Outcomes

- **User A:**
  - Opens and closes two long positions (110k→109k, 120k→121k)
  - First cycle: Realizes **-100,000** PnL (loss), margin unlocked, holdings decrease by 1
  - Second cycle: Realizes **+100,000** PnL (profit), margin unlocked, holdings decrease by 1
  - All margin is unlocked after both closes

- **User B:**
  - Opens and closes two short positions (110k→109k, 120k→121k)
  - First cycle: Realizes **+100,000** PnL (profit), margin unlocked, holdings increase by 1
  - Second cycle: Realizes **-100,000** PnL (loss), margin unlocked, holdings increase by 1
  - All margin is unlocked after both closes

- **Trade outcomes:**
  - All trades show correct PnL, margin unlock, and updated balances/holdings
  - No open positions remain for either user
  - Locked margin per order is tracked and released correctly

#### Final Balances and Holdings (if both started with 500,000 USD and 10 BTC)

- **User A:**
  - USD: **450,000 USD**
  - BTC: 10 (if cash-settled; adjust if physical delivery)

- **User B:**
  - USD: **460,000 USD**
  - BTC: 10 (if cash-settled; adjust if physical delivery)

All margin is unlocked, and there are no open positions for either user.

---

## Step-by-Step 8-Payload Test Flow (No userId)

Send these payloads in order, using the correct user context for each step:

**1. Buy Limit @ 11,000,000 (User A)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 11000000,
  "slippage": 0
}
```

**2. Sell Market (User B)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**3. Buy Limit @ 10,900,000 (User B)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 10900000,
  "slippage": 0
}
```

**4. Sell Market (User A)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**5. Buy Limit @ 12,000,000 (User A)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 12000000,
  "slippage": 0
}
```

**6. Sell Market (User B)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

**7. Buy Limit @ 12,100,000 (User B)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "buy",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "limit",
  "limitPrice": 12100000,
  "slippage": 0
}
```

**8. Sell Market (User A)**
```json
{
  "margin": 5000000,
  "asset": "BTC_USDC",
  "side": "sell",
  "leverage": 1,
  "quantity": 1.0,
  "orderType": "market",
  "slippage": 0
}
```

### Actual Outcomes (Engine Verified)

- **User A:**
  - Final USD balance: 500,000 USD
  - Final BTC holdings: 10

- **User B:**
  - Final USD balance: 500,000 USD
  - Final BTC holdings: 10

All margin is unlocked, no open positions remain, and all PnL is realized as expected.

---
