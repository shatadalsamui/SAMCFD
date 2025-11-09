use crate::modules::execution::publish_trade_outcome_for_market_order;
use crate::modules::pnl::calculate_pnl;
use crate::modules::state::EngineState;
use crate::modules::types::{order_to_trade, Order, Side};
use tokio::sync::mpsc::Sender;

/// Apply netting logic for an order fill.
/// Checks for opposite position, closes it if exists, realizes PnL, updates state.
/// If no opposite or excess quantity, opens new position.
pub async fn apply_netting(
    engine_state: &mut EngineState,
    order: &Order,
    close_price: i64,
    tx: &Sender<String>,
) {
    let opposite_side = match order.side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    };

    let existing_position_data = engine_state
        .open_trades
        .iter()
        .find(|(_, trade)| {
            trade.user_id == order.user_id
                && trade.asset == order.asset
                && trade.side == opposite_side
        })
        .map(|(id, trade)| {
            let locked_margin = engine_state.get_locked_margin_or(id, trade.margin);
            (id.clone(), trade.clone(), locked_margin)
        });

    if let Some((existing_id, mut existing_trade, locked_margin)) = existing_position_data {
        let close_qty = order.quantity.min(existing_trade.quantity);
        existing_trade.close_price = Some(close_price);
        let existing_total_qty = existing_trade.quantity;
        let pnl = if existing_total_qty > 0 {
            calculate_pnl(&existing_trade) * close_qty / existing_total_qty
        } else {
            0
        };
        let margin_return = if existing_total_qty > 0 {
            locked_margin * close_qty / existing_total_qty
        } else {
            0
        };

        // Update balance with PnL and return margin
        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
            *balance += pnl;
            *balance += margin_return;
        }

        // Update holdings ledger for the closed exposure
        let holdings_key = (order.user_id.clone(), order.asset.clone());
        {
            let entry = engine_state
                .holdings
                .entry(holdings_key.clone())
                .or_insert(0);
            match existing_trade.side {
                Side::Buy => *entry -= close_qty,
                Side::Sell => *entry += close_qty,
            }
        }

        println!(
            "Order {} filled. Closing {} position. Entry: {}, Close: {}, PnL: {}",
            order.id,
            if matches!(opposite_side, Side::Buy) {
                "long"
            } else {
                "short"
            },
            existing_trade.entry_price.unwrap_or(0),
            close_price,
            pnl
        );

        // Update existing trade
        let mut updated_state: Option<(i64, i64)> = None;
        if let Some(trade) = engine_state.open_trades.get_mut(&existing_id) {
            trade.quantity -= close_qty;
            let new_quantity = trade.quantity;
            let new_margin = if existing_total_qty > 0 {
                locked_margin * new_quantity / existing_total_qty
            } else {
                0
            };
            trade.margin = new_margin;
            updated_state = Some((new_quantity, new_margin));
        }
        if let Some((new_quantity, new_margin)) = updated_state {
            if new_quantity <= 0 {
                engine_state.open_trades.remove(&existing_id);
                engine_state.release_locked_margin(&existing_id);
            } else {
                engine_state.set_locked_margin(&existing_id, new_margin);
            }
        }

        let remaining_qty = order.quantity - close_qty;
        let remaining_margin = if remaining_qty > 0 { order.margin } else { 0 };
        if remaining_qty > 0 {
            // Open new position for the net new exposure
            let mut trade = order_to_trade(order);
            trade.quantity = remaining_qty;
            trade.entry_price = Some(close_price);
            trade.close_price = Some(close_price);
            trade.margin = remaining_margin;
            let holdings_key = (order.user_id.clone(), order.asset.clone());
            match order.side {
                Side::Buy => {
                    *engine_state
                        .holdings
                        .entry(holdings_key.clone())
                        .or_insert(0) += remaining_qty
                }
                Side::Sell => {
                    *engine_state.holdings.entry(holdings_key).or_insert(0) -= remaining_qty
                }
            }
            let margin_to_record = trade.margin;
            engine_state.open_trades.insert(order.id.clone(), trade);
            engine_state.set_locked_margin(&order.id, margin_to_record);
            publish_trade_outcome_for_market_order(
                engine_state,
                order,
                Some(close_price),
                close_price,
                0,
                "filled",
                tx,
            )
            .await;
        } else {
            publish_trade_outcome_for_market_order(
                engine_state,
                order,
                Some(existing_trade.entry_price.unwrap_or(0)),
                close_price,
                pnl,
                "closed",
                tx,
            )
            .await;
        }
    } else {
        // No opposite position, open new position
        let mut trade = order_to_trade(order);
        trade.entry_price = Some(close_price);
        trade.close_price = Some(close_price);
        trade.margin = order.margin;
        let holdings_key = (order.user_id.clone(), order.asset.clone());
        match order.side {
            Side::Buy => {
                *engine_state
                    .holdings
                    .entry(holdings_key.clone())
                    .or_insert(0) += order.quantity
            }
            Side::Sell => {
                *engine_state.holdings.entry(holdings_key).or_insert(0) -= order.quantity;
                println!(
                    "Order {} filled. Opening new short position at {}. PnL: 0",
                    order.id, close_price
                );
            }
        }
        let margin_to_record = trade.margin;
        engine_state.open_trades.insert(order.id.clone(), trade);
        engine_state.set_locked_margin(&order.id, margin_to_record);
        publish_trade_outcome_for_market_order(
            engine_state,
            order,
            Some(close_price),
            close_price,
            0,
            "filled",
            tx,
        )
        .await;
    }
}
