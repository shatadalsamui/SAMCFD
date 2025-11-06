use crate::kafka::producer;
use crate::modules::execution::apply_execution;
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

    // Validate user balance and deduct margin in its own scope
    let balance = engine_state.balances.get_mut(&req.user_id);
    if let Some(balance) = balance {
        if *balance < req.margin {
            println!("Insufficient balance for user: {}", req.user_id);
            // Publish rejection response
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
        *balance -= req.margin;
    } else {
        println!(
            "No balance found for user: {}. Requesting balance...",
            req.user_id
        );
        if let Err(e) = producer::send_balance_request(&req.user_id).await {
            eprintln!("Failed to send balance request: {:?}", e);
        }
        // Store the pending trade
        engine_state
            .pending_trades
            .entry(req.user_id.clone())
            .or_default()
            .push(req);
        return;
    }

    // Holdings check for SELL orders
    if matches!(req.side, Side::Sell) {
        let key = (req.user_id.clone(), req.asset.clone());
        let req_qty = req.quantity.map(|q| q as f64).unwrap_or(0.0);
        match engine_state.holdings.get(&key) {
            Some(quantity) if *quantity >= req_qty => {
                // Sufficient holdings, proceed
            }
            Some(_) => {
                println!(
                    "Insufficient holdings for user: {} asset: {}",
                    req.user_id, req.asset
                );
                // Publish rejection response
                let mut response_json = serde_json::json!({
                    "userId": req.user_id,
                    "status": "rejected",
                    "reason": "Insufficient holdings"
                });
                if let Some(ref corr_id) = req.correlation_id {
                    response_json["correlationId"] = serde_json::json!(corr_id);
                }
                let response = response_json.to_string();
                producer::send_trade_create_response(&req.user_id, &response).await;
                return;
            }
            None => {
                // Send holdings request and store trade as pending
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
        }
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
        quantity: req.quantity.map(|q| q as f64).unwrap_or(0.0),
        filled: 0.0,
        status: OrderStatus::Open,
        margin: req.margin,
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

                    // Check if user has an open SELL position to close
                    let existing_position_data = engine_state
                        .open_trades
                        .iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id
                                && trade.asset == order.asset
                                && matches!(trade.side, Side::Sell)
                        })
                        .map(|(id, trade)| (id.clone(), trade.entry_price.unwrap_or(0.0)));

                    if let Some((existing_id, entry_price)) = existing_position_data {
                        // Closing an existing SELL position
                        let pnl = (entry_price - close_price) * order.quantity * order.leverage;

                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!("Order {} filled. Closing short position. Entry: {}, Close: {}, PnL: {}", 
                            order.id, entry_price, close_price, pnl);

                        // Remove the closed position - DO NOT re-insert
                        engine_state.open_trades.remove(&existing_id);

                        // Send TradeOutcome to mpsc channel for the CLOSED position
                        let trade_outcome = crate::modules::types::TradeOutcome {
                            trade_id: order.id.clone(),
                            user_id: order.user_id.clone(),
                            asset: order.asset.clone(),
                            side: order.side.clone(),
                            quantity: order.quantity,
                            entry_price: Some(entry_price),
                            close_price: Some(close_price),
                            pnl: Some(pnl),
                            status: Some("closed".to_string()),
                            timestamp: Some(order.created_at as i64),
                            margin: Some(order.margin),
                            leverage: Some(order.leverage),
                            slippage: Some(0.0),
                            reason: None,
                            success: Some(true),
                            order_type: Some(order.order_type.clone()),
                            limit_price: if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                        };
                        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
                            let _ = tx.send(json_string).await;
                        }
                    } else {
                        // Opening a new BUY position
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);

                        let pnl = 0.0;
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!(
                            "Order {} filled. Opening new long position at {}. PnL: {}",
                            order.id, close_price, pnl
                        );

                        engine_state.open_trades.insert(order.id.clone(), trade);
                        // Send TradeOutcome to mpsc channel
                        let trade_outcome = crate::modules::types::TradeOutcome {
                            trade_id: order.id.clone(),
                            user_id: order.user_id.clone(),
                            asset: order.asset.clone(),
                            side: order.side.clone(),
                            quantity: order.quantity,
                            entry_price: order.price,
                            close_price: Some(close_price),
                            pnl: Some(pnl),
                            status: Some("filled".to_string()),
                            timestamp: Some(order.created_at as i64),
                            margin: Some(order.margin),
                            leverage: Some(order.leverage),
                            slippage: Some(0.0),
                            reason: None,
                            success: Some(true),
                            order_type: Some(order.order_type.clone()),
                            limit_price: if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                        };
                        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
                            let _ = tx.send(json_string).await;
                        }
                    }

                    // Also apply executions for each matched counterparty (they sold)
                    for ct in matched_trades {
                        let exec_price = ct.price.unwrap_or(close_price);
                        let exec_qty = ct.quantity.min(order.quantity);
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
                            if matches!(ct.order_type, OrderType::Limit) { ct.price } else { None },
                            ct.margin,
                            ct.created_at,
                            &tx,
                        ).await;
                    }
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price, _matched_trades) = add_limit_order(&mut order, &mut order_book.sell, &tx).await;
                    if filled > 0.0 {
                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;

                            // Apply execution for this user (they bought)
                            apply_execution(
                                &mut engine_state,
                                &order.user_id,
                                &order.asset,
                                &Side::Buy,
                                order.quantity,
                                close_price,
                                order.leverage,
                                &order.id,
                                &order.order_type,
                                if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                                order.margin,
                                order.created_at,
                                &tx,
                            ).await;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
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
                                if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                                order.margin,
                                order.created_at,
                                &tx,
                            ).await;
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

                    // Check if user has an open BUY position to close
                    let existing_position_data = engine_state
                        .open_trades
                        .iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id
                                && trade.asset == order.asset
                                && matches!(trade.side, Side::Buy)
                        })
                        .map(|(id, trade)| (id.clone(), trade.entry_price.unwrap_or(0.0)));

                    if let Some((existing_id, entry_price)) = existing_position_data {
                        // Closing an existing BUY position
                        let pnl = (close_price - entry_price) * order.quantity * order.leverage;

                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!(
                            "Order {} filled. Closing long position. Entry: {}, Close: {}, PnL: {}",
                            order.id, entry_price, close_price, pnl
                        );

                        // Remove the closed position - DO NOT re-insert
                        engine_state.open_trades.remove(&existing_id);

                        // Send TradeOutcome for the CLOSED position
                        let trade_outcome = crate::modules::types::TradeOutcome {
                            trade_id: order.id.clone(),
                            user_id: order.user_id.clone(),
                            asset: order.asset.clone(),
                            side: order.side.clone(),
                            quantity: order.quantity,
                            entry_price: Some(entry_price),
                            close_price: Some(close_price),
                            pnl: Some(pnl),
                            status: Some("closed".to_string()),
                            timestamp: Some(order.created_at as i64),
                            margin: Some(order.margin),
                            leverage: Some(order.leverage),
                            slippage: Some(0.0),
                            reason: None,
                            success: Some(true),
                            order_type: Some(order.order_type.clone()),
                            limit_price: if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                        };
                        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
                            let _ = tx.send(json_string).await;
                        }
                    } else {
                        // Opening a new SELL position
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);

                        let pnl = 0.0;
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!(
                            "Order {} filled. Opening new short position at {}. PnL: {}",
                            order.id, close_price, pnl
                        );

                        engine_state.open_trades.insert(order.id.clone(), trade);
                        
                        // Send TradeOutcome to mpsc channel
                        let trade_outcome = crate::modules::types::TradeOutcome {
                            trade_id: order.id.clone(),
                            user_id: order.user_id.clone(),
                            asset: order.asset.clone(),
                            side: order.side.clone(),
                            quantity: order.quantity,
                            entry_price: Some(close_price),
                            close_price: Some(close_price),
                            pnl: Some(pnl),
                            status: Some("filled".to_string()),
                            timestamp: Some(order.created_at as i64),
                            margin: Some(order.margin),
                            leverage: Some(order.leverage),
                            slippage: Some(0.0),
                            reason: None,
                            success: Some(true),
                            order_type: Some(order.order_type.clone()),
                            limit_price: if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                        };
                        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
                            let _ = tx.send(json_string).await;
                        }
                    }

                    // Also apply executions for each matched counterparty (they bought)
                    for ct in matched_trades {
                        let exec_price = ct.price.unwrap_or(close_price);
                        let exec_qty = ct.quantity.min(order.quantity);
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
                            if matches!(ct.order_type, OrderType::Limit) { ct.price } else { None },
                            ct.margin,
                            ct.created_at,
                            &tx,
                        ).await;
                    }
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price, _matched_trades) = add_limit_order(&mut order, &mut order_book.buy, &tx).await;
                    if filled > 0.0 {
                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;

                            // Apply execution for this user (they sold)
                            apply_execution(
                                &mut engine_state,
                                &order.user_id,
                                &order.asset,
                                &Side::Sell,
                                order.quantity,
                                close_price,
                                order.leverage,
                                &order.id,
                                &order.order_type,
                                if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                                order.margin,
                                order.created_at,
                                &tx,
                            ).await;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
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
                                if matches!(order.order_type, OrderType::Limit) { order.price } else { None },
                                order.margin,
                                order.created_at,
                                &tx,
                            ).await;
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
