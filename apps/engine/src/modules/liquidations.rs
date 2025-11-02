use crate::modules::pnl::calculate_pnl; // Updated import for pnl
use crate::modules::state::EngineState; // Updated import for state
use crate::modules::types::{trade_to_order, Order, Side}; // Updated import for types

/// Check if liquidation is needed for a trade
pub fn check_liquidation(
    entry_price: f64,
    latest_price: f64,
    quantity: f64,
    margin_used: f64,
    maintenance_margin_percent: f64,
) -> bool {
    // Calculate unrealized PnL
    let unrealized_pnl = match entry_price < latest_price {
        true => (latest_price - entry_price) * quantity / entry_price,
        false => (entry_price - latest_price) * quantity / entry_price,
    };

    // Calculate current margin
    let current_margin = margin_used + unrealized_pnl;

    // Calculate maintenance margin
    let maintenance_margin = (margin_used * maintenance_margin_percent) / 100.0;

    // Return true if liquidation is needed
    current_margin < maintenance_margin
}

/// Liquidate a trade
pub fn liquidate_trade(
    state: &mut EngineState,
    order_id: &str,
    latest_price: f64,
    maintenance_margin_percent: f64,
) {
    if let Some(trade) = state.open_trades.remove(order_id) {
        let order = trade_to_order(&trade);
        let pnl = calculate_pnl(&order, &latest_price);

        // Deduct losses from user balance
        if let Some(user_balance) = state.balances.get_mut(&order.user_id) {
            *user_balance += pnl;
            println!(
                "Liquidated order {} at price {} with PnL: {}. Updated balance: {}",
                order_id, latest_price, pnl, user_balance
            );
        }
    }
}
