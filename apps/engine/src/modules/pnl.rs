use crate::modules::types::{Side, Trade};

/// Calculate absolute PnL for a trade execution
/// Buy: (close - entry) * qty * leverage
/// Sell: (entry - close) * qty * leverage
pub fn calculate_pnl(trade: &Trade) -> f64 {
    let entry_price = trade.entry_price.unwrap_or(0.0);
    let close_price = trade.close_price.unwrap_or(0.0);
    let quantity = trade.quantity;
    let leverage = trade.leverage;

    match trade.side {
        Side::Buy => (close_price - entry_price) * quantity * leverage,
        Side::Sell => (entry_price - close_price) * quantity * leverage,
    }
}
