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
    quantity: f64,
    price: f64,
    leverage: f64,
    order_id: &str,
    order_type: &OrderType,
    limit_price: Option<f64>,
    margin: f64,
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
        .map(|(id, t)| (id.clone(), t.entry_price.unwrap_or(0.0), t.side.clone()));

    if let Some((existing_id, entry_price, existing_side)) = existing_position {
        // Close existing position
        let pnl = match side_executed {
            Side::Buy => (entry_price - price) * quantity * leverage, // closing short
            Side::Sell => (price - entry_price) * quantity * leverage, // closing long
        };

        if let Some(balance) = engine_state.balances.get_mut(user_id) {
            *balance += pnl;
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

        // Remove closed position - DO NOT re-insert it into open_trades
        // Closed positions should not be monitored for liquidation
        engine_state.open_trades.remove(&existing_id);

        // Get updated values
        let updated_balance = engine_state.balances.get(user_id).copied();
        let updated_holdings = engine_state
            .holdings
            .get(&(user_id.to_string(), asset.to_string()))
            .copied();

        // Publish TradeOutcome for closed position
        let trade_outcome = crate::modules::types::TradeOutcome {
            trade_id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            quantity,
            entry_price: Some(entry_price),
            close_price: Some(price),
            pnl: Some(pnl),
            status: Some("closed".to_string()),
            timestamp: Some(created_at),
            margin: Some(margin),
            leverage: Some(leverage),
            slippage: Some(0.0),
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
            println!("Trade outcome published for closed position: {}", order_id);
        }
        // The closed position is now removed from open_trades and won't be liquidated
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
            margin: 0.0,
            leverage,
            stop_loss_percent: None,
            take_profit_percent: None,
            created_at: 0,
            expiry: None,
        });
        new_trade.entry_price = Some(price);
        new_trade.close_price = Some(price);
        println!(
            "Order {} filled. Opening new {} position at {}. PnL: 0",
            order_id,
            match side_executed {
                Side::Buy => "long",
                Side::Sell => "short",
            },
            price
        );
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
            pnl: Some(0.0),
            status: Some("filled".to_string()),
            timestamp: Some(created_at),
            margin: Some(margin),
            leverage: Some(leverage),
            slippage: Some(0.0),
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
    entry_price: Option<f64>,
    close_price: f64,
    pnl: f64,
    status: &str,
    tx: &tokio::sync::mpsc::Sender<String>,
) {
    let updated_balance = engine_state.balances.get(&order.user_id).copied();
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
        slippage: Some(0.0),
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
