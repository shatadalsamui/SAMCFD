use crate::modules::types::Order;
use std::collections::{BTreeMap, VecDeque};

/// Match a market order with the opposite side of the order book
pub fn match_market_order(
    order: Order,
    opposite_book: &mut BTreeMap<i64, VecDeque<Order>>,
) -> Vec<Order> {
    let mut matched_trades = Vec::new();
    let mut remaining_quantity = order.quantity;

    let mut to_remove = Vec::new();
    for (price, orders_at_price) in opposite_book.iter_mut() {
        while let Some(mut limit_order) = orders_at_price.pop_front() {
            let available_quantity = (limit_order.quantity - limit_order.filled).max(0);
            if available_quantity <= 0 {
                continue;
            }

            let match_quantity = remaining_quantity.min(available_quantity);
            if match_quantity <= 0 {
                orders_at_price.push_front(limit_order);
                break;
            }

            if limit_order.quantity <= 0 {
                continue;
            }

            let executed_margin = ((limit_order.margin as i128 * match_quantity as i128)
                / limit_order.quantity as i128) as i64;

            limit_order.filled += match_quantity;
            limit_order.margin -= executed_margin;
            if limit_order.margin < 0 {
                limit_order.margin = 0;
            }
            remaining_quantity -= match_quantity;

            let mut executed_order = limit_order.clone();
            executed_order.quantity = match_quantity;
            executed_order.filled = match_quantity;
            executed_order.margin = executed_margin;
            executed_order.price = Some(*price);

            matched_trades.push(executed_order);

            println!(
                "Matched market order {} with limit order {} for {} units at price {}",
                order.id, limit_order.id, match_quantity, price
            );

            let remaining_for_order = (limit_order.quantity - limit_order.filled).max(0);
            if remaining_for_order > 0 {
                orders_at_price.push_front(limit_order);
            }

            if orders_at_price.is_empty() {
                to_remove.push(*price);
            }

            if remaining_quantity <= 0 {
                break;
            }
        }
        if remaining_quantity == 0 {
            break;
        }
    }

    for price in to_remove {
        opposite_book.remove(&price);
    }

    if remaining_quantity > 0 {
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
    opposite_book: &mut BTreeMap<i64, VecDeque<Order>>,
    _tx: &tokio::sync::mpsc::Sender<String>,
    _engine_state: &crate::modules::state::EngineState,
) -> (i64, i64, Vec<Order>) {
    let matched_trades = match_market_order(order.clone(), opposite_book);

    let filled: i64 = matched_trades.iter().map(|trade| trade.quantity).sum();
    let total_cost: i128 = matched_trades
        .iter()
        .map(|trade| trade.price.unwrap_or(0) as i128 * trade.quantity as i128)
        .sum();
    let close_price = if filled > 0 {
        (total_cost / filled as i128) as i64
    } else {
        0
    };

    // Update the original order's filled quantity
    order.filled = filled;

    let remaining_quantity = order.quantity - filled;

    if remaining_quantity > 0 {
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

    (filled, close_price, matched_trades)
}
