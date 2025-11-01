use crate::modules::order_matching::{match_market_order, add_limit_order};
use crate::modules::state::SharedEngineState;
use crate::modules::state::OrderBook; 
use crate::modules::types::{CreateTradeRequest, Order, OrderStatus, OrderType, Side, order_to_trade};

pub async fn process_trade_create(state: SharedEngineState, req: CreateTradeRequest) {
    let mut engine_state = state.lock().await;

    // Validate user balance and deduct margin
    if let Some(balance) = engine_state.balances.get_mut(&req.user_id) {
        if *balance < req.margin {
            println!("Insufficient balance for user: {}", req.user_id);
            return;
        }
        *balance -= req.margin;
    } else {
        println!("No balance found for user: {}", req.user_id);
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
        quantity: req.quantity.unwrap_or(0),
        filled: 0,
        status: OrderStatus::Open,
        margin: req.margin,
        leverage: req.leverage,
        stop_loss_percent: req.stop_loss_percent,
        take_profit_percent: req.take_profit_percent,
        created_at: req.timestamp,
        expiry: req.expiry_timestamp,
    };

    // Match or add to the order book
    let order_book = engine_state
        .order_books
        .entry(req.asset.clone())
        .or_insert_with(OrderBook::new);

    match order.side {
    Side::Buy => match order.order_type {
        OrderType::Market => {
            let matched_trades = match_market_order(order.clone(), &mut order_book.sell);
            for trade in matched_trades {
                order.filled += trade.quantity;
                if order.filled >= order.quantity {
                    order.status = OrderStatus::Filled;
                    break;
                }
            }
            if order.filled < order.quantity {
                order.status = OrderStatus::PartiallyFilled;
            }
        }
        OrderType::Limit => {
            if order.filled < order.quantity {
                add_limit_order(order.clone(), &mut order_book.buy);
            }
        }
    },
    Side::Sell => match order.order_type {
        OrderType::Market => {
            let matched_trades = match_market_order(order.clone(), &mut order_book.buy);
            for trade in matched_trades {
                order.filled += trade.quantity;
                if order.filled >= order.quantity {
                    order.status = OrderStatus::Filled;
                    break;
                }
            }
            if order.filled < order.quantity {
                order.status = OrderStatus::PartiallyFilled;
            }
        }
        OrderType::Limit => {
            if order.filled < order.quantity {
                add_limit_order(order.clone(), &mut order_book.sell);
            }
        }
    },
}

    // Add to open trades
    engine_state.open_trades.insert(order.id.clone(), order_to_trade(&order));
}






