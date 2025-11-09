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
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 3000000,
    "slippage": 0
  }
  ```

  **Result:**
  - When the sell limit order is placed first, and then the buy market order is sent, the engine matches and fills both orders at the limit price (3000000).
  - Market order is fully filled, limit order is fully filled.
  - PnL: 0 (no profit/loss at entry, as expected for opening trade).
  - This matches standard exchange behavior.
  - ✅ Scenario tested and passed.

  **Result:**
  - Market order did NOT match immediately (no liquidity).
  - When the limit order arrived, it did NOT match the resting market order (engine does not match new limit orders against resting market orders, which is standard exchange behavior).

- [x] Market order vs. market order (should NOT match)
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
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  **Result:**
  - No match occurred between the two market orders, as expected.
  - Both market orders remained unfilled (partially filled), which is standard exchange behavior (market orders do not match each other).

- [x] Limit order vs. limit order (should match only if prices cross)
  **Payloads:**

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 3100000,
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 3000000,
    "slippage": 0
  }
  ```

  **Result:**
  - Buy limit order at 3100000 and sell limit order at 3000000 matched at price 3100000.
  - Both orders were fully filled as expected when prices crossed.

- [x] Buy market order vs. sell limit order (should match and fill)
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
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 3000000,
    "slippage": 0
  }
  ```

  **Result:**

- Scenario tested by sending the sell limit order first, then the buy market order.
- Orders matched and filled at the limit price (3000000) as expected.
- Market order fully filled, limit order fully filled.
- PnL: 0 (no profit/loss at entry).
- This matches standard exchange behavior.
- ✅ Scenario tested and passed.

- [x] Sell market order vs. buy limit order (should match and fill)
  **Payloads:**

  ```json
  {
    "margin": 5000,
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
    "margin": 5000,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 3000000,
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
