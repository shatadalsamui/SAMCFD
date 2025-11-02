use crate::modules::pnl::calculate_pnl; // Updated import for pnl
use crate::modules::state::EngineState; // Updated import for state
use crate::modules::types::{trade_to_order, Order}; // Updated import for types

/// Check if liquidation is needed for a trade
pub fn check_liquidation(
    entry_price: f64,
    latest_price: f64,
    quantity: f64,
    margin_used: f64,
) -> bool {
    let maintenance_margin_percent = get_maintenance_margin_percent(margin_used);

    let unrealized_pnl = (latest_price - entry_price) * quantity / entry_price;
    let current_margin = margin_used + unrealized_pnl;
    let maintenance_margin = (margin_used * maintenance_margin_percent) / 100.0;

    current_margin < maintenance_margin
}

/// Liquidate a trade
pub fn liquidate_trade(
    state: &mut EngineState,
    order_id: &str,
    latest_price: f64,
) {
    if let Some(trade) = state.open_trades.remove(order_id) {
        let order = trade_to_order(&trade);
        let pnl = calculate_pnl(&order, &latest_price);

        if let Some(user_balance) = state.balances.get_mut(&order.user_id) {
            *user_balance += pnl;
            println!(
                "Liquidated order {} at price {} with PnL: {}. Updated balance: {}",
                order_id, latest_price, pnl, user_balance
            );
        }
    }
}

fn get_maintenance_margin_percent(margin_used: f64) -> f64 {
    match margin_used {
        x if x < 100.0 => 1.0,
        x if x < 1000.0 => 2.0,
        x if x < 10000.0 => 3.0,
        x if x < 100000.0 => 4.0,
        _ => 5.0,
    }
}