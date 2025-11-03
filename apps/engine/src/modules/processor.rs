use crate::kafka::producer;
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
    let balance = engine_state.balances.get_mut(&req.user_id);
    if let Some(balance) = balance {
        if *balance < req.margin {
            println!("Insufficient balance for user: {}", req.user_id);
            return;
        }
        *balance -= req.margin;
    } else {
        println!(
            "No balance found for user: {}. Requesting balance...",
            req.user_id
        );
        if let Err(e) = producer::send_balance_request(&req.user_id).await {
            eprintln!("Failed to send balance request: {:?}", e);
        }
        // Store the pending trade
        engine_state
            .pending_trades
            .entry(req.user_id.clone())
            .or_default()
            .push(req);
        return;
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
                    
                    // Check if user has an open SELL position to close
                    let existing_position_data = engine_state.open_trades.iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id 
                            && trade.asset == order.asset 
                            && matches!(trade.side, Side::Sell)
                        })
                        .map(|(id, trade)| (id.clone(), trade.entry_price.unwrap_or(0.0)));
                    
                    if let Some((existing_id, entry_price)) = existing_position_data {
                        // Closing an existing SELL position
                        let pnl = (entry_price - close_price) * order.quantity * order.leverage;
                        
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!("Order {} filled. Closing short position. Entry: {}, Close: {}, PnL: {}", 
                            order.id, entry_price, close_price, pnl);
                        
                        // Remove the closed position
                        engine_state.open_trades.remove(&existing_id);
                        
                        // Add this closing order to history
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(entry_price);
                        trade.close_price = Some(close_price);
                        engine_state.open_trades.insert(order.id.clone(), trade);
                    } else {
                        // Opening a new BUY position
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);
                        
                        let pnl = 0.0;
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!("Order {} filled. Opening new long position at {}. PnL: {}", order.id, close_price, pnl);
                        
                        engine_state.open_trades.insert(order.id.clone(), trade);
                    }
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
                            
                            // Create trade with close_price
                            let mut trade = order_to_trade(&order);
                            trade.close_price = Some(close_price);
                            
                            let pnl = crate::modules::pnl::calculate_pnl(&trade);
                            if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                                *balance += pnl;
                            }
                            println!("Order {} filled. PnL: {}", order.id, pnl);
                            
                            // Add trade to open_trades
                            engine_state.open_trades.insert(order.id.clone(), trade);
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                            
                            // Create partial trade
                            let mut trade = order_to_trade(&order);
                            trade.close_price = Some(close_price);
                            
                            let pnl = crate::modules::pnl::calculate_pnl(&trade);
                            if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                                *balance += pnl;
                            }
                            println!(
                                "Matched Buy limit order: {} filled {} at avg price {}",
                                order.id, filled, close_price
                            );
                            
                            // Add trade to open_trades
                            engine_state.open_trades.insert(order.id.clone(), trade);
                        }
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
                    
                    // Check if user has an open BUY position to close
                    let existing_position_data = engine_state.open_trades.iter()
                        .find(|(_, trade)| {
                            trade.user_id == order.user_id 
                            && trade.asset == order.asset 
                            && matches!(trade.side, Side::Buy)
                        })
                        .map(|(id, trade)| (id.clone(), trade.entry_price.unwrap_or(0.0)));
                    
                    if let Some((existing_id, entry_price)) = existing_position_data {
                        // Closing an existing BUY position
                        let pnl = (close_price - entry_price) * order.quantity * order.leverage;
                        
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!("Order {} filled. Closing long position. Entry: {}, Close: {}, PnL: {}", 
                            order.id, entry_price, close_price, pnl);
                        
                        // Remove the closed position
                        engine_state.open_trades.remove(&existing_id);
                        
                        // Add this closing order to history
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(entry_price);
                        trade.close_price = Some(close_price);
                        engine_state.open_trades.insert(order.id.clone(), trade);
                    } else {
                        // Opening a new SELL position
                        let mut trade = order_to_trade(&order);
                        trade.entry_price = Some(close_price);
                        trade.close_price = Some(close_price);
                        
                        let pnl = 0.0;
                        if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                            *balance += pnl;
                        }
                        println!("Order {} filled. Opening new short position at {}. PnL: {}", order.id, close_price, pnl);
                        
                        engine_state.open_trades.insert(order.id.clone(), trade);
                    }
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
                            
                            // Create trade with close_price
                            let mut trade = order_to_trade(&order);
                            trade.close_price = Some(close_price);
                            
                            let pnl = crate::modules::pnl::calculate_pnl(&trade);
                            if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                                *balance += pnl;
                            }
                            println!("Order {} filled. PnL: {}", order.id, pnl);
                            
                            // Add trade to open_trades
                            engine_state.open_trades.insert(order.id.clone(), trade);
                        } else {
                            order.status = OrderStatus::PartiallyFilled;
                            
                            // Create partial trade
                            let mut trade = order_to_trade(&order);
                            trade.close_price = Some(close_price);
                            
                            let pnl = crate::modules::pnl::calculate_pnl(&trade);
                            if let Some(balance) = engine_state.balances.get_mut(&order.user_id) {
                                *balance += pnl;
                            }
                            println!(
                                "Matched Sell limit order: {} filled {} at avg price {}",
                                order.id, filled, close_price
                            );
                            
                            // Add trade to open_trades
                            engine_state.open_trades.insert(order.id.clone(), trade);
                        }
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

    // Add to open trades only if not already added (for unfilled/partial orders)
    if !engine_state.open_trades.contains_key(&order.id) {
        engine_state
            .open_trades
            .insert(order.id.clone(), order_to_trade(&order));
    }

    println!(
        "Trade created and added to open trades: user={}, order={}, status={:?}",
        order.user_id, order.id, order.status
    );
}
