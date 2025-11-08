use crate::kafka::producer;
use crate::modules::execution::{apply_execution, publish_trade_outcome_for_market_order};
use crate::modules::netting::apply_netting;
use crate::modules::order_matching::{add_limit_order, match_market_order};
use crate::modules::state::OrderBook;
use crate::modules::state::SharedEngineState;
use crate::modules::types::{
    order_to_trade, CreateTradeRequest, Order, OrderStatus, OrderType, Side,
};
use std::collections::VecDeque;
use uuid::Uuid;

pub async fn process_trade_create(
    state: SharedEngineState,
    req: CreateTradeRequest,
    tx: tokio::sync::mpsc::Sender<String>,
) {
    println!(
        "Processing trade request - correlationId: {:?}",
        req.correlation_id
    );
    let mut engine_state = state.lock().await;

    // Validate user balance - NO deduction yet
    let current_balance = match engine_state.balances.get(&req.user_id) {
        Some(balance) => *balance,
        None => {
            println!(
                "No balance found for user: {}. Requesting balance...",
                req.user_id
            );
            if let Err(e) = producer::send_balance_request(&req.user_id).await {
                eprintln!("Failed to send balance request: {:?}", e);
            }
            engine_state
                .pending_trades
                .entry(req.user_id.clone())
                .or_default()
                .push(req);
            return;
        }
    };

    // Ensure holdings data is loaded before proceeding
    if !engine_state
        .holdings
        .contains_key(&(req.user_id.clone(), req.asset.clone()))
    {
        if let Err(e) = producer::send_holdings_request(&req.user_id, &req.asset).await {
            eprintln!("Failed to send holdings request: {:?}", e);
        }
        engine_state
            .pending_trades
            .entry(req.user_id.clone())
            .or_default()
            .push(req);
        return;
    }

    let req_qty = req.quantity.map(|q| q as f64).unwrap_or(0.0);

    // Determine overlap with existing opposite positions (closing) and net new exposure (opening)
    let opposite_side = match req.side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    };
    let total_opposite_qty: f64 = engine_state
        .open_trades
        .values()
        .filter(|trade| {
            trade.user_id == req.user_id && trade.asset == req.asset && trade.side == opposite_side
        })
        .map(|trade| trade.quantity)
        .sum();

    let closing_qty = req_qty.min(total_opposite_qty);
    let opening_qty = (req_qty - closing_qty).max(0.0);

    let opening_margin_total = if req_qty > 0.0 {
        req.margin * (opening_qty / req_qty)
    } else {
        0.0
    };

    let mut remaining_closing_qty = closing_qty;
    let mut remaining_opening_qty = opening_qty;
    let margin_per_new_unit = if opening_qty > 0.0 {
        opening_margin_total / opening_qty
    } else {
        0.0
    };

    let mut allocate_margin_for_execution = |exec_qty: f64| -> f64 {
        let closing_used = remaining_closing_qty.min(exec_qty);
        remaining_closing_qty -= closing_used;
        let opening_used = ((exec_qty - closing_used).max(0.0)).min(remaining_opening_qty);
        remaining_opening_qty -= opening_used;
        opening_used * margin_per_new_unit
    };

    // Calculate required funds (margin for new exposure only)
    let required_funds = opening_margin_total;

    // Validate balance
    if current_balance < required_funds {
        println!("Insufficient balance for user: {}", req.user_id);
        let mut response_json = serde_json::json!({
            "userId": req.user_id,
            "status": "rejected",
            "reason": "Insufficient balance"
        });
        if let Some(ref corr_id) = req.correlation_id {
            response_json["correlationId"] = serde_json::json!(corr_id);
        }
        let response = response_json.to_string();
        producer::send_trade_create_response(&req.user_id, &response).await;
        return;
    }

    // Deduct only the margin required for net-new exposure
    if let Some(balance) = engine_state.balances.get_mut(&req.user_id) {
        *balance -= required_funds;
    }

    // Create the order and assign orderId only after all checks pass
    let order_id = Uuid::new_v4().to_string();
    let mut order = Order {
        id: order_id.clone(),
        user_id: req.user_id.clone(),
        asset: req.asset.clone(),
        side: req.side.clone(),
        order_type: req.order_type.unwrap_or(OrderType::Market),
        price: req.limit_price,
        quantity: req_qty,
        filled: 0.0,
        status: OrderStatus::Open,
        margin: opening_margin_total,
        leverage: req.leverage,
        stop_loss_percent: req.stop_loss_percent,
        take_profit_percent: req.take_profit_percent,
        created_at: req.timestamp,
        expiry: req.expiry_timestamp,
    };
    // After adding to order book, publish accepted response
    let mut response_json = serde_json::json!({
        "orderId": order_id,
        "userId": order.user_id,
        "status": "accepted",
        "details": {
            "asset": order.asset,
            "side": order.side,
            "quantity": order.quantity,
            "margin": order.margin,
            "leverage": order.leverage,
            "orderType": order.order_type,
            "price": order.price
        }
    });
    if let Some(ref corr_id) = req.correlation_id {
        response_json["correlationId"] = serde_json::json!(corr_id);
    }
    let response = response_json.to_string();
    producer::send_trade_create_response(&order_id, &response).await;
    let asset_key = order.asset.clone();
    // Temporarily take ownership of the asset book to avoid overlapping borrows.
    let mut order_book = engine_state
        .order_books
        .remove(&asset_key)
        .unwrap_or_else(OrderBook::new);
    let _prices_snapshot = engine_state.prices.clone();

    match order.side {
        Side::Buy => match order.order_type {
            OrderType::Market => {
                let matched_trades = match_market_order(order.clone(), &mut order_book.sell);
                for trade in &matched_trades {
                    order.filled += trade.quantity;
                    println!(
                        "Matched Buy order {} with Sell order {} for {} units at price {}",
                        order.id,
                        trade.id,
                        trade.quantity,
                        trade.price.unwrap()
                    );
                    if order.filled >= order.quantity {
                        order.status = OrderStatus::Filled;
                        break;
                    }
                }
                if order.status == OrderStatus::Filled {
                    let close_price = matched_trades.last().and_then(|t| t.price).unwrap_or(0.0);
                    order.price = Some(close_price);

                    // Netting logic for Buy order
                    let existing_position_data = engine_state
                        .open_trades
                        .iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id
                                && trade.asset == order.asset
                                && matches!(trade.side, Side::Sell)
                        })
                        .map(|(id, trade)| {
                            (
                                id.clone(),
                                trade.entry_price.unwrap_or(0.0),
                                trade.quantity,
                                trade.margin,
                                trade.side.clone(),
                            )
                        });

                    if let Some((
                        existing_id,
                        entry_price,
                        existing_qty,
                        existing_margin,
                        existing_side,
                    )) = existing_position_data
                    {
                        let close_qty = order.quantity.min(existing_qty);
                        let margin_return_ratio = if existing_qty > 0.0 {
                            close_qty / existing_qty
                        } else {
                            0.0
                        };
                        let pnl = (entry_price - close_price) * close_qty * order.leverage;
                        if matches!(existing_side, Side::Sell) {
                            *engine_state
                                .holdings
                                .entry((order.user_id.clone(), order.asset.clone()))
                                .or_insert(0.0) += close_qty;
                        }
                        let returned_margin = existing_margin * margin_return_ratio;

                        // Update balance with PnL and returned margin
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                            *balance += returned_margin;
                        }

                        println!("Order {} filled. Closing short position. Entry: {}, Close: {}, PnL: {}", 
                            order.id, entry_price, close_price, pnl);

                        // Update existing trade (quantity + margin)
                        if let Some(trade) = engine_state.open_trades.get_mut(&existing_id) {
                            trade.quantity -= close_qty;
                            let remaining_ratio = if existing_qty > 0.0 {
                                (existing_qty - close_qty).max(0.0) / existing_qty
                            } else {
                                0.0
                            };
                            trade.margin = existing_margin * remaining_ratio;
                            if trade.quantity <= 0.0 {
                                engine_state.open_trades.remove(&existing_id);
                            }
                        }

                        let remaining_qty = order.quantity - close_qty;
                        let remaining_margin = if remaining_qty > 0.0 {
                            order.margin
                        } else {
                            0.0
                        };
                        if remaining_qty > 0.0 {
                            // Open new long position
                            let mut trade = order_to_trade(&order);
                            trade.quantity = remaining_qty;
                            trade.entry_price = Some(close_price);
                            trade.close_price = Some(close_price);
                            trade.margin = remaining_margin;
                            let holdings_key = (order.user_id.clone(), order.asset.clone());
                            *engine_state.holdings.entry(holdings_key).or_insert(0.0) +=
                                remaining_qty;
                            println!(
                                "Order {} filled. Opening new long position at {}. PnL: 0",
                                order.id, close_price
                            );
                            engine_state.open_trades.insert(order.id.clone(), trade);
                            publish_trade_outcome_for_market_order(
                                &engine_state,
                                &order,
                                Some(close_price),
                                close_price,
                                0.0,
                                "filled",
                                &tx,
                            )
                            .await;
                        } else {
                            publish_trade_outcome_for_market_order(
                                &engine_state,
                                &order,
                                Some(entry_price),
                                close_price,
                                pnl,
                                "closed",
                                &tx,
                            )
                            .await;
                        }
                    } else {
                        // Opening a new BUY position
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);
                        // Update holdings for new long position
                        let holdings_key = (order.user_id.clone(), order.asset.clone());
                        *engine_state.holdings.entry(holdings_key).or_insert(0.0) += order.quantity;
                        println!(
                            "Order {} filled. Opening new long position at {}. PnL: 0",
                            order.id, close_price
                        );
                        engine_state.open_trades.insert(order.id.clone(), trade);
                        publish_trade_outcome_for_market_order(
                            &engine_state,
                            &order,
                            Some(close_price),
                            close_price,
                            0.0,
                            "filled",
                            &tx,
                        )
                        .await;
                    }

                    // Also apply executions for each matched counterparty (they sold)
                    for ct in matched_trades {
                        let exec_price = ct.price.unwrap_or(close_price);
                        let exec_qty = ct.quantity;
                        apply_execution(
                            &mut engine_state,
                            &ct.user_id,
                            &ct.asset,
                            &Side::Sell,
                            exec_qty,
                            exec_price,
                            ct.leverage,
                            &ct.id,
                            &ct.order_type,
                            if matches!(ct.order_type, OrderType::Limit) {
                                ct.price
                            } else {
                                None
                            },
                            ct.margin,
                            ct.created_at,
                            &tx,
                        )
                        .await;
                    }
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price, matched_trades) =
                        add_limit_order(&mut order, &mut order_book.sell, &tx, &engine_state).await;
                    if filled > 0.0 {
                        for ct in matched_trades {
                            let exec_price = ct.price.unwrap_or(close_price);
                            let exec_qty = ct.quantity;
                            apply_execution(
                                &mut engine_state,
                                &ct.user_id,
                                &ct.asset,
                                &Side::Sell,
                                exec_qty,
                                exec_price,
                                ct.leverage,
                                &ct.id,
                                &ct.order_type,
                                if matches!(ct.order_type, OrderType::Limit) {
                                    ct.price
                                } else {
                                    None
                                },
                                ct.margin,
                                ct.created_at,
                                &tx,
                            )
                            .await;
                        }

                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;

                            // Apply netting
                            apply_netting(&mut engine_state, &order, close_price, &tx).await;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                            let executed_margin = allocate_margin_for_execution(filled);
                            // Apply execution for filled portion
                            apply_execution(
                                &mut engine_state,
                                &order.user_id,
                                &order.asset,
                                &Side::Buy,
                                filled,
                                close_price,
                                order.leverage,
                                &order.id,
                                &order.order_type,
                                if matches!(order.order_type, OrderType::Limit) {
                                    order.price
                                } else {
                                    None
                                },
                                executed_margin,
                                order.created_at,
                                &tx,
                            )
                            .await;
                            order.margin = (remaining_opening_qty * margin_per_new_unit).max(0.0);
                        }
                    }
                    if order.filled < order.quantity {
                        let mut remaining_order = order.clone();
                        remaining_order.quantity = order.quantity - order.filled;
                        remaining_order.filled = 0.0;
                        let price_level = order_book
                            .buy
                            .entry(ordered_float::OrderedFloat(order.price.unwrap()))
                            .or_insert(VecDeque::new());
                        price_level.push_back(remaining_order);
                        println!("Added Buy limit order to book: {:?}", order.id);
                    }
                }
            }
        },
        Side::Sell => match order.order_type {
            OrderType::Market => {
                let matched_trades = match_market_order(order.clone(), &mut order_book.buy);
                for trade in &matched_trades {
                    order.filled += trade.quantity;
                    println!(
                        "Matched Sell order {} with Buy order {} for {} units at price {}",
                        order.id,
                        trade.id,
                        trade.quantity,
                        trade.price.unwrap()
                    );
                    if order.filled >= order.quantity {
                        order.status = OrderStatus::Filled;
                        break;
                    }
                }
                if order.status == OrderStatus::Filled {
                    let close_price = matched_trades.last().and_then(|t| t.price).unwrap_or(0.0);
                    order.price = Some(close_price);

                    // Netting logic for Sell order
                    let existing_position_data = engine_state
                        .open_trades
                        .iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id
                                && trade.asset == order.asset
                                && matches!(trade.side, Side::Buy)
                        })
                        .map(|(id, trade)| {
                            (
                                id.clone(),
                                trade.entry_price.unwrap_or(0.0),
                                trade.quantity,
                                trade.margin,
                            )
                        });

                    if let Some((existing_id, entry_price, existing_qty, existing_margin)) =
                        existing_position_data
                    {
                        let close_qty = order.quantity.min(existing_qty);
                        let margin_return_ratio = if existing_qty > 0.0 {
                            close_qty / existing_qty
                        } else {
                            0.0
                        };
                        let pnl = (close_price - entry_price) * close_qty * order.leverage;
                        let returned_margin = existing_margin * margin_return_ratio;

                        // Update balance with PnL and returned margin
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                            *balance += returned_margin;
                        }

                        // Decrease holdings when closing long
                        let holdings_key = (order.user_id.clone(), order.asset.clone());
                        if let Some(holdings) = engine_state.holdings.get_mut(&holdings_key) {
                            *holdings -= close_qty;
                        }

                        println!(
                            "Order {} filled. Closing long position. Entry: {}, Close: {}, PnL: {}",
                            order.id, entry_price, close_price, pnl
                        );

                        // Update existing trade
                        if let Some(trade) = engine_state.open_trades.get_mut(&existing_id) {
                            trade.quantity -= close_qty;
                            let remaining_ratio = if existing_qty > 0.0 {
                                (existing_qty - close_qty).max(0.0) / existing_qty
                            } else {
                                0.0
                            };
                            trade.margin = existing_margin * remaining_ratio;
                            if trade.quantity <= 0.0 {
                                engine_state.open_trades.remove(&existing_id);
                            }
                        }

                        let remaining_qty = order.quantity - close_qty;
                        let remaining_margin = if remaining_qty > 0.0 {
                            order.margin
                        } else {
                            0.0
                        };
                        if remaining_qty > 0.0 {
                            // Open new short position for the net new exposure
                            let mut trade = order_to_trade(&order);
                            trade.quantity = remaining_qty;
                            trade.entry_price = Some(close_price);
                            trade.close_price = Some(close_price);
                            trade.margin = remaining_margin;
                            let holdings_key = (order.user_id.clone(), order.asset.clone());
                            *engine_state.holdings.entry(holdings_key).or_insert(0.0) -=
                                remaining_qty;
                            println!(
                                "Order {} filled. Opening new short position at {}. PnL: 0",
                                order.id, close_price
                            );
                            engine_state.open_trades.insert(order.id.clone(), trade);
                            publish_trade_outcome_for_market_order(
                                &engine_state,
                                &order,
                                Some(close_price),
                                close_price,
                                0.0,
                                "filled",
                                &tx,
                            )
                            .await;
                        } else {
                            publish_trade_outcome_for_market_order(
                                &engine_state,
                                &order,
                                Some(entry_price),
                                close_price,
                                pnl,
                                "closed",
                                &tx,
                            )
                            .await;
                        }
                    } else {
                        // Opening a new SELL position (short)
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);
                        let holdings_key = (order.user_id.clone(), order.asset.clone());
                        *engine_state.holdings.entry(holdings_key).or_insert(0.0) -= order.quantity;
                        println!(
                            "Order {} filled. Opening new short position at {}. PnL: 0",
                            order.id, close_price
                        );
                        engine_state.open_trades.insert(order.id.clone(), trade);

                        publish_trade_outcome_for_market_order(
                            &engine_state,
                            &order,
                            Some(close_price),
                            close_price,
                            0.0,
                            "filled",
                            &tx,
                        )
                        .await;
                    }

                    // Also apply executions for each matched counterparty (they bought)
                    for ct in matched_trades {
                        let exec_price = ct.price.unwrap_or(close_price);
                        let exec_qty = ct.quantity;
                        apply_execution(
                            &mut engine_state,
                            &ct.user_id,
                            &ct.asset,
                            &Side::Buy,
                            exec_qty,
                            exec_price,
                            ct.leverage,
                            &ct.id,
                            &ct.order_type,
                            if matches!(ct.order_type, OrderType::Limit) {
                                ct.price
                            } else {
                                None
                            },
                            ct.margin,
                            ct.created_at,
                            &tx,
                        )
                        .await;
                    }
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price, matched_trades) =
                        add_limit_order(&mut order, &mut order_book.buy, &tx, &engine_state).await;
                    if filled > 0.0 {
                        for ct in matched_trades {
                            let exec_price = ct.price.unwrap_or(close_price);
                            let exec_qty = ct.quantity;
                            apply_execution(
                                &mut engine_state,
                                &ct.user_id,
                                &ct.asset,
                                &Side::Buy,
                                exec_qty,
                                exec_price,
                                ct.leverage,
                                &ct.id,
                                &ct.order_type,
                                if matches!(ct.order_type, OrderType::Limit) {
                                    ct.price
                                } else {
                                    None
                                },
                                ct.margin,
                                ct.created_at,
                                &tx,
                            )
                            .await;
                        }

                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;

                            // Apply netting
                            apply_netting(&mut engine_state, &order, close_price, &tx).await;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                            let executed_margin = allocate_margin_for_execution(filled);
                            // Apply execution for filled portion
                            apply_execution(
                                &mut engine_state,
                                &order.user_id,
                                &order.asset,
                                &Side::Sell,
                                filled,
                                close_price,
                                order.leverage,
                                &order.id,
                                &order.order_type,
                                if matches!(order.order_type, OrderType::Limit) {
                                    order.price
                                } else {
                                    None
                                },
                                executed_margin,
                                order.created_at,
                                &tx,
                            )
                            .await;
                            order.margin = (remaining_opening_qty * margin_per_new_unit).max(0.0);
                        }
                    }
                    if order.filled < order.quantity {
                        let mut remaining_order = order.clone();
                        remaining_order.quantity = order.quantity - order.filled;
                        remaining_order.filled = 0.0;
                        let price_level = order_book
                            .sell
                            .entry(ordered_float::OrderedFloat(order.price.unwrap()))
                            .or_insert(VecDeque::new());
                        price_level.push_back(remaining_order);
                        println!("Added Sell limit order to book: {:?}", order.id);
                    }
                }
            }
        },
    }

    engine_state.order_books.insert(asset_key, order_book);

    // Do not insert trades for unfilled orders; trades are recorded upon execution via apply_execution or fill branches.

    println!(
        "Trade created and added to open trades: user={}, order={}, status={:?}",
        order.user_id, order.id, order.status
    );
}
