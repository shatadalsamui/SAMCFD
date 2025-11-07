use crate::modules::types::Order;
use ordered_float::OrderedFloat;
use std::collections::{BTreeMap, HashMap, VecDeque};

/// Match a market order with the opposite side of the order book
pub fn match_market_order(
    order: Order,
    opposite_book: &mut BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
) -> Vec<Order> {
    let mut matched_trades = Vec::new();
    let mut remaining_quantity = order.quantity;

    let mut to_remove = Vec::new();
    for (price, orders_at_price) in opposite_book.iter_mut() {
        while let Some(mut limit_order) = orders_at_price.pop_front() {
            let match_quantity = remaining_quantity.min(limit_order.quantity - limit_order.filled);

            limit_order.filled += match_quantity;
            remaining_quantity -= match_quantity;

            matched_trades.push(limit_order.clone());

            println!(
                "Matched market order {} with limit order {} for {} units at price {}",
                order.id,
                limit_order.id,
                match_quantity,
                price.0 // .0 to get f64
            );

            if limit_order.filled < limit_order.quantity {
                orders_at_price.push_front(limit_order);
                break;
            }

            if orders_at_price.is_empty() {
                to_remove.push(*price);
            }

            if remaining_quantity == 0.0 {
                break;
            }
        }
        if remaining_quantity == 0.0 {
            break;
        }
    }

    for price in to_remove {
        opposite_book.remove(&price);
    }

    if remaining_quantity > 0.0 {
        println!(
            "Market order {} partially filled. Remaining quantity: {}",
            order.id, remaining_quantity
        );
    } else {
        println!("Market order {} fully filled.", order.id);
    }

    matched_trades
}

/// Add a limit order to the appropriate side of the order book
pub async fn add_limit_order(
    order: &mut Order,
    opposite_book: &mut BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    tx: &tokio::sync::mpsc::Sender<String>,
    engine_state: &crate::modules::state::EngineState,
) -> (f64, f64, Vec<Order>) {
    let matched_trades = match_market_order(order.clone(), opposite_book);

    let filled: f64 = matched_trades.iter().map(|trade| trade.quantity).sum();
    let total_cost: f64 = matched_trades.iter().map(|trade| trade.price.unwrap_or(0.0) * trade.quantity).sum();
    let close_price = if filled > 0.0 { total_cost / filled } else { 0.0 };

    // Update the original order's filled quantity
    order.filled = filled;

    let remaining_quantity = order.quantity - filled;

    if remaining_quantity > 0.0 {
        println!(
            "Limit order {} partially filled: {} units at avg price {}. Remaining: {}",
            order.id, filled, close_price, remaining_quantity
        );
    } else {
        println!(
            "Limit order {} fully filled: {} units at avg price {}",
            order.id, filled, close_price
        );
    }

    // Publish TradeOutcome for each matched counterparty
    for ct in &matched_trades {
        let exec_price = ct.price.unwrap_or(close_price);

        let updated_balance = engine_state.balances.get(&ct.user_id).copied();
        let updated_holdings = engine_state.holdings.get(&(ct.user_id.clone(), ct.asset.clone())).copied();

        let trade_outcome = crate::modules::types::TradeOutcome {
            trade_id: ct.id.clone(),
            user_id: ct.user_id.clone(),
            asset: ct.asset.clone(),
            side: ct.side.clone(),
            quantity: ct.quantity,
            entry_price: ct.price,
            close_price: Some(exec_price),
            pnl: Some(0.0),
            status: Some("filled".to_string()),
            timestamp: Some(ct.created_at as i64),
            margin: Some(ct.margin),
            leverage: Some(ct.leverage),
            slippage: Some(0.0),
            reason: None,
            success: Some(true),
            order_type: Some(ct.order_type.clone()),
            limit_price: if matches!(ct.order_type, crate::modules::types::OrderType::Limit) { ct.price } else { None },
            updated_balance,
            updated_holdings
        };
        if let Ok(json_string) = serde_json::to_string(&trade_outcome) {
            let _ = tx.send(json_string).await;
            println!("Trade outcome published for counterparty: {}", ct.id);
        }
    }

    (filled, close_price, matched_trades)
}
