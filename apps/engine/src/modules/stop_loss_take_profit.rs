use crate::modules::pnl::calculate_pnl;
use crate::modules::state::SharedEngineState;
use crate::modules::types::{trade_to_order, Side};

pub async fn monitor_stop_loss_take_profit(state: SharedEngineState) {
    let mut engine_state = state.lock().await;

    // Iterate through all open trades
    let mut to_close = Vec::new(); // Track trades to close
    for (order_id, trade) in engine_state.open_trades.iter() {
        if let Some(latest_price) = engine_state.prices.get(&trade.asset) {
            // Use entry_price as trade.price or fallback to latest_price
            let entry_price = trade.price.unwrap_or(*latest_price);
            match trade.side {
                Side::Buy => {
                    // Check take profit
                    if let Some(tp) = trade.take_profit_percent {
                        let take_profit_price = entry_price + (entry_price * tp as f64 / 100.0);
                        if *latest_price >= take_profit_price {
                            println!("Take profit triggered for order {}", order_id);
                            to_close.push(order_id.clone());
                            continue;
                        }
                    }

                    // Check stop loss
                    if let Some(sl) = trade.stop_loss_percent {
                        let stop_loss_price = entry_price - (entry_price * sl as f64 / 100.0);
                        if *latest_price <= stop_loss_price {
                            println!("Stop loss triggered for order {}", order_id);
                            to_close.push(order_id.clone());
                            continue;
                        }
                    }
                }
                Side::Sell => {
                    // Check take profit
                    if let Some(tp) = trade.take_profit_percent {
                        let take_profit_price = entry_price - (entry_price * tp as f64 / 100.0);
                        if *latest_price <= take_profit_price {
                            println!("Take profit triggered for order {}", order_id);
                            to_close.push(order_id.clone());
                            continue;
                        }
                    }

                    // Check stop loss
                    if let Some(sl) = trade.stop_loss_percent {
                        let stop_loss_price = entry_price + (entry_price * sl as f64 / 100.0);
                        if *latest_price >= stop_loss_price {
                            println!("Stop loss triggered for order {}", order_id);
                            to_close.push(order_id.clone());
                            continue;
                        }
                    }
                }
            }
        }
    }

    // Close trades that hit stop loss or take profit
    for order_id in to_close {
        if let Some(trade) = engine_state.open_trades.remove(&order_id) {
            let order = trade_to_order(&trade);
            let pnl = calculate_pnl(&order, engine_state.prices.get(&order.asset).unwrap());
            *engine_state.balances.get_mut(&order.user_id).unwrap() += pnl;
            println!("Closed order {} with PnL: {}", order_id, pnl);
        }
    }
}
