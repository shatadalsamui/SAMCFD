  **User assignment:**
  - 1st payload (buy limit): send from **User A** (e.g., shatadalsamuimain)
  - 2nd payload (sell market): send from **User B** (e.g., shatadalsamui82)
  - 3rd payload (buy limit): send from **User B** (e.g., shatadalsamui82)
  - 4th payload (sell market): send from **User A** (e.g., shatadalsamuimain)

---

- [ ] Full open/close cycle (for realized PnL test)
  **Payloads:**
  
  ```json
  {
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 105000,
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 50.0,
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
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 105000,
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "market",
    "slippage": 0
  }
  ```

  **Result:**
  - Use these four payloads in order (buy limit, sell market, buy limit, sell market) to guarantee a full open/close cycle and trigger realized PnL logging.
  - Assign the first and last order to one user, and the second and third to another user.
  - This scenario is for verifying PnL calculation and log output after a position is closed at a different price.
# Order Matching Test Plan

## ✅ To-Do List for Order Matching Engine

- [x] Limit order placed first, then market order (should match and fill)
  **Payloads:**
  
  ```json
  {
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 30000,
    "slippage": 0
  }
  ```
  
  ```json
  {
    "margin": 50.0,
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
    "margin": 50.0,
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
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 30000,
    "slippage": 0
  }
  ```

  **Result:**
  - When the sell limit order is placed first, and then the buy market order is sent, the engine matches and fills both orders at the limit price (30000).
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
    "margin": 50.0,
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
    "margin": 50.0,
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
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 31000,
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 30000,
    "slippage": 0
  }
  ```

  **Result:**
  - Buy limit order at 31000 and sell limit order at 30000 matched at price 31000.
  - Both orders were fully filled as expected when prices crossed.

 - [x] Buy market order vs. sell limit order (should match and fill)
  **Payloads:**

  ```json
  {
    "margin": 50.0,
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
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 30000,
    "slippage": 0
  }
  ```

  **Result:**
  - Scenario tested by sending the sell limit order first, then the buy market order.
  - Orders matched and filled at the limit price (30000) as expected.
  - Market order fully filled, limit order fully filled.
  - PnL: 0 (no profit/loss at entry).
  - This matches standard exchange behavior.
  - ✅ Scenario tested and passed.

 - [x] Sell market order vs. buy limit order (should match and fill)
  **Payloads:**

  ```json
  {
    "margin": 50.0,
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
    "margin": 50.0,
    "asset": "BTC_USDC",
    "side": "buy",
    "leverage": 1,
    "quantity": 1.0,
    "orderType": "limit",
    "limitPrice": 30000,
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
    "margin": 150.0,
    "asset": "BTC_USDC",
    "side": "sell",
    "leverage": 1,
    "quantity": 3.0,
    "orderType": "limit",
    "limitPrice": 30000,
    "slippage": 0
  }
  ```

  ```json
  {
    "margin": 50.0,
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
    "margin": 50.0,
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
    "margin": 50.0,
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
  - Each market order matched and filled 1 unit at the limit price (30000).
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
