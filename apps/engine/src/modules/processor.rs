use crate::modules::liquidations::{check_liquidation, liquidate_trade};
use crate::modules::order_matching::{add_limit_order, match_market_order};
use crate::modules::state::OrderBook;
use crate::modules::state::SharedEngineState;
use crate::modules::types::{
    order_to_trade, CreateTradeRequest, Order, OrderStatus, OrderType, Side,
};
use std::collections::VecDeque;

pub async fn process_trade_create(state: SharedEngineState, req: CreateTradeRequest) {
    let mut engine_state = state.lock().await;

    // Validate user balance and deduct margin in its own scope
    {
        let balance = engine_state.balances.get_mut(&req.user_id);
        if let Some(balance) = balance {
            if *balance < req.margin {
                println!("Insufficient balance for user: {}", req.user_id);
                return;
            }
            *balance -= req.margin;
        } else {
            println!("No balance found for user: {}", req.user_id);
            return;
        }
    }

    // Create the order
    let mut order = Order {
        id: req.order_id.clone(),
        user_id: req.user_id.clone(),
        asset: req.asset.clone(),
        side: req.side.clone(),
        order_type: req.order_type.unwrap_or(OrderType::Market),
        price: req.limit_price,
        quantity: req.quantity.map(|q| q as f64).unwrap_or(0.0),
        filled: 0.0,
        status: OrderStatus::Open,
        margin: req.margin,
        leverage: req.leverage,
        stop_loss_percent: req.stop_loss_percent,
        take_profit_percent: req.take_profit_percent,
        created_at: req.timestamp,
        expiry: req.expiry_timestamp,
    };

    let latest_price = engine_state
        .prices
        .get(&order.asset)
        .cloned()
        .unwrap_or(order.price.unwrap_or(0.0));
    
    // Check for liquidation
    if check_liquidation(
        order.price.unwrap_or(0.0),
        latest_price,
        order.quantity,
        order.margin,
    ) {
        liquidate_trade(&mut engine_state, &order.id, latest_price);
        println!("Order {} liquidated due to insufficient margin.", order.id);
        return;
    }

    let asset_key = order.asset.clone();
    // Temporarily take ownership of the asset book to avoid overlapping borrows.
    let mut order_book = engine_state
        .order_books
        .remove(&asset_key)
        .unwrap_or_else(OrderBook::new);
    let prices_snapshot = engine_state.prices.clone();

    match order.side {
        Side::Buy => match order.order_type {
            OrderType::Market => {
                let matched_trades = match_market_order(order.clone(), &mut order_book.sell);
                for trade in &matched_trades {
                    order.filled += trade.quantity;
                    println!(
                        "Matched Buy order {} with Sell order {} for {} units at price {}",
                        order.id,
                        trade.id,
                        trade.quantity,
                        trade.price.unwrap()
                    );
                    if order.filled >= order.quantity {
                        order.status = OrderStatus::Filled;
                        break;
                    }
                }
                if order.status == OrderStatus::Filled {
                    let close_price = matched_trades.last().and_then(|t| t.price).unwrap_or(0.0);
                    order.price = Some(close_price);
                    let pnl = crate::modules::pnl::calculate_pnl(&order, &close_price);
                    if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                        *balance += pnl;
                    }
                    println!("Order {} filled. PnL: {}", order.id, pnl);
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price) = add_limit_order(&mut order, &mut order_book.sell);
                    if filled > 0.0 {
                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                        }
                        let pnl = crate::modules::pnl::calculate_pnl(&order, &close_price);
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance -= pnl;
                        }
                        println!(
                            "Matched Buy limit order: {} filled {} at avg price {}",
                            order.id, filled, close_price
                        );
                    }
                    if order.filled < order.quantity {
                        let mut remaining_order = order.clone();
                        remaining_order.quantity = order.quantity - order.filled;
                        remaining_order.filled = 0.0;
                        let price_level = order_book
                            .buy
                            .entry(ordered_float::OrderedFloat(order.price.unwrap()))
                            .or_insert(VecDeque::new());
                        price_level.push_back(remaining_order);
                        println!("Added Buy limit order to book: {:?}", order.id);
                    }
                }
            }
        },
        Side::Sell => match order.order_type {
            OrderType::Market => {
                let matched_trades = match_market_order(order.clone(), &mut order_book.buy);
                for trade in &matched_trades {
                    order.filled += trade.quantity;
                    println!(
                        "Matched Sell order {} with Buy order {} for {} units at price {}",
                        order.id,
                        trade.id,
                        trade.quantity,
                        trade.price.unwrap()
                    );
                    if order.filled >= order.quantity {
                        order.status = OrderStatus::Filled;
                        break;
                    }
                }
                if order.status == OrderStatus::Filled {
                    let close_price = matched_trades.last().and_then(|t| t.price).unwrap_or(0.0);
                    order.price = Some(close_price);
                    let pnl = crate::modules::pnl::calculate_pnl(&order, &close_price);
                    if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                        *balance += pnl;
                    }
                    println!("Order {} filled. PnL: {}", order.id, pnl);
                } else if order.filled < order.quantity {
                    order.status = OrderStatus::PartiallyFilled;
                }
            }
            OrderType::Limit => {
                if order.filled < order.quantity {
                    let (filled, close_price) = add_limit_order(&mut order, &mut order_book.buy);
                    if filled > 0.0 {
                        if filled == order.quantity {
                            order.price = Some(close_price);
                            order.status = OrderStatus::Filled;
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                        }
                        let pnl = crate::modules::pnl::calculate_pnl(&order, &close_price);
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!(
                            "Matched Sell limit order: {} filled {} at avg price {}",
                            order.id, filled, close_price
                        );
                    }
                    if order.filled < order.quantity {
                        let mut remaining_order = order.clone();
                        remaining_order.quantity = order.quantity - order.filled;
                        remaining_order.filled = 0.0;
                        let price_level = order_book
                            .sell
                            .entry(ordered_float::OrderedFloat(order.price.unwrap()))
                            .or_insert(VecDeque::new());
                        price_level.push_back(remaining_order);
                        println!("Added Sell limit order to book: {:?}", order.id);
                    }
                }
            }
        },
    }

    engine_state.order_books.insert(asset_key, order_book);

    // Add to open trades
    engine_state
        .open_trades
        .insert(order.id.clone(), order_to_trade(&order));
}
