use crate::modules::types::Order;
use std::collections::VecDeque;

/// Match a market order with the opposite side of the order book
pub fn match_market_order(
    order: Order,
    opposite_book: &mut std::collections::BTreeMap<i64, VecDeque<Order>>,
) -> Vec<Order> {
    let mut matched_trades = Vec::new(); // Collect matched trades
    let mut remaining_quantity = order.quantity;

    // Iterate through the opposite book (best price first)
    let mut to_remove = Vec::new(); // Track empty price levels to remove
    for (price, orders_at_price) in opposite_book.iter_mut() {
        while let Some(mut limit_order) = orders_at_price.pop_front() {
            let match_quantity = remaining_quantity.min(limit_order.quantity - limit_order.filled);

            // Update filled quantities
            limit_order.filled += match_quantity;
            remaining_quantity -= match_quantity;

            // Add the matched trade
            matched_trades.push(limit_order.clone());

            println!(
                "Matched market order {} with limit order {} for {} units at price {}",
                order.id, limit_order.id, match_quantity, price
            );

            // If the limit order is fully filled, remove it
            if limit_order.filled < limit_order.quantity {
                orders_at_price.push_front(limit_order); // Put it back if partially filled
                break;
            }

            // If no more orders at this price level, mark it for removal
            if orders_at_price.is_empty() {
                to_remove.push(*price);
            }

            // If the market order is fully filled, stop matching
            if remaining_quantity == 0 {
                break;
            }
        }

        // Stop if the market order is fully filled
        if remaining_quantity == 0 {
            break;
        }
    }

    // Remove empty price levels
    for price in to_remove {
        opposite_book.remove(&price);
    }

    // Log if the market order is partially or fully unfilled
    if remaining_quantity > 0 {
        println!(
            "Market order {} partially filled. Remaining quantity: {}",
            order.id, remaining_quantity
        );
    } else {
        println!("Market order {} fully filled.", order.id);
    }

    matched_trades // Return the matched trades
}

/// Add a limit order to the appropriate side of the order book
pub fn add_limit_order(
    order: Order,
    same_side_book: &mut std::collections::BTreeMap<i64, VecDeque<Order>>,
) {
    // Add the order to the appropriate price level
    let price_level = same_side_book.entry(order.price.unwrap()).or_insert(VecDeque::new());
    price_level.push_back(order.clone());
    println!("Added limit order: {:?}", order);
}