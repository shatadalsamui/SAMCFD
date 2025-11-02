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
pub fn add_limit_order(
    limit_order: Order,
    same_side_book: &mut BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    opposite_book: &mut BTreeMap<OrderedFloat<f64>, VecDeque<Order>>,
    balances: &mut HashMap<String, f64>,
    _prices: &HashMap<String, f64>,
) {
    let matched_trades = match_market_order(limit_order.clone(), opposite_book);

    let filled_quantity: f64 = matched_trades.iter().map(|trade| trade.quantity).sum();
    let remaining_quantity = limit_order.quantity - filled_quantity;

    if remaining_quantity > 0.0 {
        let mut remaining_order = limit_order.clone();
        remaining_order.quantity = remaining_quantity;

        let price_level = same_side_book
            .entry(OrderedFloat(remaining_order.price.unwrap()))
            .or_insert(VecDeque::new());
        price_level.push_back(remaining_order.clone());

        println!(
            "Added remaining limit order {} to the book with {} units at price {}",
            limit_order.id,
            remaining_quantity,
            remaining_order.price.unwrap()
        );
    } else {
        // Limit order fully filled: calculate PnL and update balance
        let close_price = matched_trades
            .last()
            .and_then(|t| t.price)
            .or(limit_order.price)
            .unwrap_or(0.0);
        let pnl = crate::modules::pnl::calculate_pnl(&limit_order, &close_price);
        if let Some(balance) = balances.get_mut(&limit_order.user_id) {
            *balance += pnl;
        }
        println!(
            "Limit order {} fully filled at price {}. PnL: {}",
            limit_order.id, close_price, pnl
        );
    }
}
