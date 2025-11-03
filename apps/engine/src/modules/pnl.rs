use crate::modules::types::{Trade, Side};

/// Calculate the profit or loss (PnL) for a trade
pub fn calculate_pnl(trade: &Trade) -> f64 {
    let entry_price = trade.entry_price.unwrap_or(0.0);
    let close_price = trade.close_price.unwrap_or(0.0);
    let quantity = trade.quantity;

    match trade.side {
        Side::Buy => (close_price - entry_price) * quantity / entry_price,
        Side::Sell => (entry_price - close_price) * quantity / entry_price,
    }
}
