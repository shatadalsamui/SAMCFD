use crate::modules::state::EngineState;
use crate::modules::types::{order_to_trade, Order, OrderStatus, OrderType, Side};

/// Apply an execution to the given user's position for an asset at a price and quantity.
/// If an opposite position exists, close it (realize PnL, update balance, log).
/// Otherwise, open a new position at the execution price (PnL=0 on open).
pub async fn apply_execution(
    engine_state: &mut EngineState,
    user_id: &str,
    asset: &str,
    side_executed: &Side,
    quantity: i64,
    price: i64,
    leverage: i64,
    order_id: &str,
    order_type: &OrderType,
    limit_price: Option<i64>,
    margin: i64,
    created_at: i64,
    tx: &tokio::sync::mpsc::Sender<String>,
) {
    // Determine if opposite side exists for closing
    let opposite_is_buy = matches!(side_executed, Side::Sell);
    let existing_position = engine_state
        .open_trades
        .iter()
        .find(|(_, t)| {
            t.user_id == user_id
                && t.asset == asset
                && (matches!(t.side, Side::Buy) == opposite_is_buy)
        })
        .map(|(id, t)| {
            (
                id.clone(),
                t.entry_price.unwrap_or(0),
                t.quantity,
                t.margin,
                t.side.clone(),
                t.leverage,
            )
        });

    let holdings_key = (user_id.to_string(), asset.to_string());

    if let Some((
        existing_id,
        entry_price,
        existing_qty,
        existing_margin,
        existing_side,
        existing_leverage,
    )) = existing_position
    {
        let close_qty = quantity.min(existing_qty);
        let pnl = match side_executed {
            Side::Buy => (entry_price - price) * close_qty * existing_leverage, // closing short
            Side::Sell => (price - entry_price) * close_qty * existing_leverage, // closing long
        };

        let margin_return = if existing_qty > 0 {
            existing_margin * close_qty / existing_qty
        } else {
            0
        };

        // Update balance with PnL and return margin for closed portion
        if let Some(balance) = engine_state.balances.get_mut(user_id) {
            *balance += pnl;
            *balance += margin_return;
        }

        // Update holdings ledger to reflect the closed exposure
        if let Some(holdings) = engine_state.holdings.get_mut(&holdings_key) {
            match existing_side {
                Side::Buy => *holdings -= close_qty,
                Side::Sell => *holdings += close_qty,
            }
        }

        println!(
            "Order {} filled. Closing {} position. Entry: {}, Close: {}, PnL: {}",
            order_id,
            match existing_side {
                Side::Buy => "long",
                Side::Sell => "short",
            },
            entry_price,
            price,
            pnl
        );

        // Update existing trade quantity
        if let Some(trade) = engine_state.open_trades.get_mut(&existing_id) {
            trade.quantity -= close_qty;
            if existing_qty > 0 {
                trade.margin = existing_margin * trade.quantity / existing_qty;
            } else {
                trade.margin = 0;
            }
            if trade.quantity <= 0 {
                engine_state.open_trades.remove(&existing_id);
            }
        }

        let remaining_qty = quantity - close_qty;
        let remaining_margin = if remaining_qty > 0 { margin } else { 0 };
        if remaining_qty > 0 {
            // Open new opposite position
            let new_trade = crate::modules::types::Trade {
                id: order_id.to_string(),
                user_id: user_id.to_string(),
                asset: asset.to_string(),
                side: side_executed.clone(),
                margin: remaining_margin,
                leverage,
                quantity: remaining_qty,
                entry_price: Some(price),
                close_price: Some(price),
                pnl: Some(0),
                status: Some("filled".to_string()),
                created_at: Some(created_at),
                closed_at: None,
                take_profit_percent: None,
                stop_loss_percent: None,
                price: limit_price,
            };
            engine_state
                .open_trades
                .insert(order_id.to_string(), new_trade);

            // Update holdings for remaining
            match side_executed {
                Side::Buy => {
                    *engine_state
                        .holdings
                        .entry(holdings_key.clone())
                        .or_insert(0) += remaining_qty
                }
                Side::Sell => {
                    *engine_state
                        .holdings
                        .entry(holdings_key.clone())
                        .or_insert(0) -= remaining_qty
                }
            }

            // Publish for remaining
            let trade_outcome = crate::modules::types::TradeOutcome {
                trade_id: order_id.to_string(),
                user_id: user_id.to_string(),
                asset: asset.to_string(),
                side: side_executed.clone(),
                quantity: remaining_qty,
                entry_price: Some(price),
                close_price: Some(price),
                pnl: Some(0),
                status: Some("filled".to_string()),
                timestamp: Some(created_at),
                margin: Some(remaining_margin),
                leverage: Some(leverage),
                slippage: Some(0),
                reason: None,
                success: Some(true),
                order_type: Some(order_type.clone()),
                limit_price: if matches!(order_type, OrderType::Limit) {
                    limit_price
                } else {
                    None
                },
                updated_balance: engine_state.balances.get(user_id).copied(),
                updated_holdings: engine_state
                    .holdings
                    .get(&(user_id.to_string(), asset.to_string()))
                    .copied(),
            };
            if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
                let _ = tx.send(json_string).await;
            }
        }

        // Publish for closed portion
        let trade_outcome = crate::modules::types::TradeOutcome {
            trade_id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            quantity: close_qty,
            entry_price: Some(entry_price),
            close_price: Some(price),
            pnl: Some(pnl),
            status: Some("closed".to_string()),
            timestamp: Some(created_at),
            margin: Some(margin_return),
            leverage: Some(leverage),
            slippage: Some(0),
            reason: None,
            success: Some(true),
            order_type: Some(order_type.clone()),
            limit_price: if matches!(order_type, OrderType::Limit) {
                limit_price
            } else {
                None
            },
            updated_balance: engine_state.balances.get(user_id).copied(),
            updated_holdings: engine_state
                .holdings
                .get(&(user_id.to_string(), asset.to_string()))
                .copied(),
        };
        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
            let _ = tx.send(json_string).await;
            println!("Trade outcome published for closed position: {}", order_id);
        }
    } else {
        // Open new position
        let mut new_trade = order_to_trade(&Order {
            id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            order_type: OrderType::Market,
            price: Some(price),
            quantity,
            filled: quantity,
            status: OrderStatus::Filled,
            margin,
            leverage,
            stop_loss_percent: None,
            take_profit_percent: None,
            created_at: 0,
            expiry: None,
        });
        new_trade.entry_price = Some(price);
        new_trade.close_price = Some(price);

        // Update holdings and balance for new position
        match side_executed {
            Side::Buy => {
                // New long position - increase holdings
                *engine_state
                    .holdings
                    .entry(holdings_key.clone())
                    .or_insert(0) += quantity;
                println!(
                    "Order {} filled. Opening new long position at {}. PnL: 0",
                    order_id, price
                );
            }
            Side::Sell => {
                *engine_state
                    .holdings
                    .entry(holdings_key.clone())
                    .or_insert(0) -= quantity;
                println!(
                    "Order {} filled. Opening new short position at {}. PnL: 0",
                    order_id, price
                );
            }
        }

        engine_state
            .open_trades
            .insert(order_id.to_string(), new_trade);

        let updated_balance = engine_state.balances.get(user_id).copied();
        let updated_holdings = engine_state
            .holdings
            .get(&(user_id.to_string(), asset.to_string()))
            .copied();

        // Publish TradeOutcome for new position
        let trade_outcome = crate::modules::types::TradeOutcome {
            trade_id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            quantity,
            entry_price: Some(price),
            close_price: Some(price),
            pnl: Some(0),
            status: Some("filled".to_string()),
            timestamp: Some(created_at),
            margin: Some(margin),
            leverage: Some(leverage),
            slippage: Some(0),
            reason: None,
            success: Some(true),
            order_type: Some(order_type.clone()),
            limit_price: if matches!(order_type, OrderType::Limit) {
                limit_price
            } else {
                None
            },
            updated_balance,
            updated_holdings,
        };
        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
            let _ = tx.send(json_string).await;
            println!("Trade outcome published for new position: {}", order_id);
        }
    }
}

pub async fn publish_trade_outcome_for_market_order(
    engine_state: &crate::modules::state::EngineState,
    order: &Order,
    entry_price: Option<i64>,
    close_price: i64,
    pnl: i64,
    status: &str,
    tx: &tokio::sync::mpsc::Sender<String>,
) {
    let current_balance = engine_state.balances.get(&order.user_id).copied();
    let updated_balance = if status == "closed" {
        current_balance // Already updated in netting.rs with PnL + returned margin
    } else {
        current_balance.map(|b| b + pnl) // For new positions (PnL=0)
    };
    let updated_holdings = engine_state
        .holdings
        .get(&(order.user_id.clone(), order.asset.clone()))
        .copied();

    let trade_outcome = crate::modules::types::TradeOutcome {
        trade_id: order.id.clone(),
        user_id: order.user_id.clone(),
        asset: order.asset.clone(),
        side: order.side.clone(),
        quantity: order.quantity,
        entry_price,
        close_price: Some(close_price),
        pnl: Some(pnl),
        status: Some(status.to_string()),
        timestamp: Some(order.created_at as i64),
        margin: Some(order.margin),
        leverage: Some(order.leverage),
    slippage: Some(0),
        reason: None,
        success: Some(true),
        order_type: Some(order.order_type.clone()),
        limit_price: if matches!(order.order_type, crate::modules::types::OrderType::Limit) {
            order.price
        } else {
            None
        },
        updated_balance,
        updated_holdings,
    };

    if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
        let _ = tx.send(json_string).await;
    }
}
