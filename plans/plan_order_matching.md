# Order Matching Test Plan

## ✅ To-Do List for Order Matching Engine

- [x] Limit order placed first, then market order (should match and fill)
- [ ] Market order placed first, then limit order (should NOT match immediately)
- [ ] Market order vs. market order (should NOT match)
- [ ] Limit order vs. limit order (should match only if prices cross)
- [ ] Buy market order vs. sell limit order (should match and fill)
- [ ] Sell market order vs. buy limit order (should match and fill)
- [ ] Partial fills (large limit order matched by multiple market orders)
- [ ] Check if PnL is generated and logged correctly after a match

---

**Goal:**  
Test all combinations of order types and verify that PnL is generated and logged as expected for each successful match.