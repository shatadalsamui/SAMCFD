use crate::types::Order;

/// Calculate the profit or loss (PnL) for an order
pub fn calculate_pnl(order: &Order, close_price: &i64) -> i64 {
    let entry_price = order.price.unwrap();
    let quantity = order.quantity;

    match order.side {
        crate::types::Side::Buy => (close_price - entry_price) * quantity / entry_price,
        crate::types::Side::Sell => (entry_price - close_price) * quantity / entry_price,
    }
}