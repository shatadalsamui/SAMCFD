use crate::modules::state::EngineState;
use crate::modules::types::{order_to_trade, Order, OrderStatus, OrderType, Side};

/// Apply an execution to the given user's position for an asset at a price and quantity.
/// If an opposite position exists, close it (realize PnL, update balance, log).
/// Otherwise, open a new position at the execution price (PnL=0 on open).
pub fn apply_execution(
    engine_state: &mut EngineState,
    user_id: &str,
    asset: &str,
    side_executed: &Side,
    quantity: f64,
    price: f64,
    leverage: f64,
    order_id: &str,
) {
    // Determine if opposite side exists for closing
    let opposite_is_buy = matches!(side_executed, Side::Sell);
    let existing_position = engine_state
        .open_trades
        .iter()
        .find(|(_, t)| t.user_id == user_id && t.asset == asset && (matches!(t.side, Side::Buy) == opposite_is_buy))
        .map(|(id, t)| (id.clone(), t.entry_price.unwrap_or(0.0), t.side.clone()));

    if let Some((existing_id, entry_price, existing_side)) = existing_position {
        // Close existing position
        let pnl = match side_executed {
            Side::Buy => (entry_price - price) * quantity * leverage,   // closing short
            Side::Sell => (price - entry_price) * quantity * leverage,  // closing long
        };

        if let Some(balance) = engine_state.balances.get_mut(user_id) {
            *balance += pnl;
        }
        println!(
            "Order {} filled. Closing {} position. Entry: {}, Close: {}, PnL: {}",
            order_id,
            match existing_side { Side::Buy => "long", Side::Sell => "short" },
            entry_price,
            price,
            pnl
        );

        // Remove closed position and add closing record
        engine_state.open_trades.remove(&existing_id);
        let mut closing_trade = order_to_trade(&Order {
            id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            order_type: OrderType::Market,
            price: Some(price),
            quantity,
            filled: quantity,
            status: OrderStatus::Filled,
            margin: 0.0,
            leverage,
            stop_loss_percent: None,
            take_profit_percent: None,
            created_at: 0,
            expiry: None,
        });
        closing_trade.entry_price = Some(entry_price);
        closing_trade.close_price = Some(price);
        engine_state.open_trades.insert(order_id.to_string(), closing_trade);
    } else {
        // Open new position
        let mut new_trade = order_to_trade(&Order {
            id: order_id.to_string(),
            user_id: user_id.to_string(),
            asset: asset.to_string(),
            side: side_executed.clone(),
            order_type: OrderType::Market,
            price: Some(price),
            quantity,
            filled: quantity,
            status: OrderStatus::Filled,
            margin: 0.0,
            leverage,
            stop_loss_percent: None,
            take_profit_percent: None,
            created_at: 0,
            expiry: None,
        });
        new_trade.entry_price = Some(price);
        new_trade.close_price = Some(price);
        println!(
            "Order {} filled. Opening new {} position at {}. PnL: 0",
            order_id,
            match side_executed { Side::Buy => "long", Side::Sell => "short" },
            price
        );
        engine_state.open_trades.insert(order_id.to_string(), new_trade);
    }
}
