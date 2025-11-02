use crate::modules::types::{Order, Side};

/// Calculate the profit or loss (PnL) for an order
pub fn calculate_pnl(order: &Order, close_price: &f64) -> f64 {
    let entry_price = order.price.unwrap() as f64;
    let quantity = order.quantity as f64;

    match order.side {
        Side::Buy => (close_price - entry_price) * quantity / entry_price,
        Side::Sell => (entry_price - close_price) * quantity / entry_price,
    }
}
