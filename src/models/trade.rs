use std::env;

use super::mode::{self, ExecutionMode};
use super::order::{Order, OrderType};
use super::pricing::Pricing;
use crate::helpers::calc;
use crate::helpers::date::*;
use crate::helpers::uuid;
use crate::scanner::instrument::*;

use serde::{Deserialize, Serialize};

pub trait Trade {
    fn get_date(&self) -> &DbDateTime;
    fn get_chrono_date(&self) -> DateTime<Local>;
    fn get_price_in(&self) -> &f64;
    fn get_price_out(&self) -> &f64;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeDirection {
    Long,
    Short,
    None,
}
impl TradeDirection {
    pub fn is_long(&self) -> bool {
        match *self {
            TradeDirection::Long => true,
            _ => false,
        }
    }

    pub fn is_short(&self) -> bool {
        match *self {
            TradeDirection::Short => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeType {
    MarketInLong,
    MarketOutLong,
    MarketInShort,
    MarketOutShort,
    OrderInLong,
    OrderOutLong,
    OrderInShort,
    OrderOutShort,
    StopLossLong,
    StopLossShort,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Position {
    MarketIn(Option<Vec<OrderType>>),
    MarketOut(Option<Vec<OrderType>>),
    MarketInOrder(Order),
    MarketOutOrder(Order),
    Order(Vec<OrderType>),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionResult {
    MarketIn(TradeResult, Option<Vec<Order>>),
    MarketOut(TradeResult),
    PendingOrder(Vec<Order>),
    MarketInOrder(TradeResult, Order),
    MarketOutOrder(TradeResult, Order),
    None,
}

impl TradeType {
    pub fn is_entry(&self) -> bool {
        match *self {
            TradeType::MarketInLong
            | TradeType::MarketInShort
            | TradeType::OrderInLong
            | TradeType::OrderInShort => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match *self {
            TradeType::MarketOutLong
            | TradeType::MarketOutShort
            | TradeType::OrderOutLong
            | TradeType::StopLossLong
            | TradeType::StopLossShort
            | TradeType::OrderOutShort => true,
            _ => false,
        }
    }

    pub fn is_long(&self) -> bool {
        match *self {
            TradeType::MarketInLong
            | TradeType::MarketOutLong
            | TradeType::StopLossLong
            | TradeType::OrderInLong
            | TradeType::OrderOutLong => true,
            _ => false,
        }
    }

    pub fn is_long_entry(&self) -> bool {
        match *self {
            TradeType::MarketInLong | TradeType::OrderInLong => true,
            _ => false,
        }
    }

    pub fn is_short(&self) -> bool {
        match *self {
            TradeType::MarketInShort
            | TradeType::MarketOutShort
            | TradeType::OrderInShort
            | TradeType::OrderOutShort => true,
            _ => false,
        }
    }

    pub fn is_short_entry(&self) -> bool {
        match *self {
            TradeType::MarketInShort | TradeType::OrderInShort => true,
            _ => false,
        }
    }

    pub fn is_order(&self) -> bool {
        match *self {
            TradeType::OrderInLong
            | TradeType::OrderOutLong
            | TradeType::OrderInShort
            | TradeType::OrderOutShort
            | TradeType::StopLossLong
            | TradeType::StopLossShort => true,
            _ => false,
        }
    }

    pub fn is_stop(&self) -> bool {
        match *self {
            TradeType::StopLossLong | TradeType::StopLossShort => true,
            _ => false,
        }
    }
}

pub fn type_from_str(trade_type: &str) -> TradeType {
    match trade_type {
        "MarketInLong" => TradeType::MarketInLong,
        "MarketOutLong" => TradeType::MarketOutLong,
        "MarketInShort" => TradeType::MarketInShort,
        "MarketOutShort" => TradeType::MarketOutShort,
        "OrderInLong" => TradeType::OrderInLong,
        "OrderOutLong" => TradeType::OrderOutLong,
        "OrderInShort" => TradeType::OrderInShort,
        "OrderOutShort" => TradeType::OrderOutShort,
        "StopLossLong" => TradeType::StopLossLong,
        "StopLossShort" => TradeType::StopLossShort,
        _ => TradeType::None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TradeResult {
    TradeIn(TradeIn),
    TradeOut(TradeOut),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeIn {
    pub id: usize,
    pub index_in: usize,
    pub quantity: f64,
    pub origin_price: f64,
    pub price_in: f64,
    pub ask: f64,
    pub spread: f64,
    pub date_in: DbDateTime,
    pub trade_type: TradeType,
}

impl Trade for TradeIn {
    fn get_date(&self) -> &DbDateTime {
        &self.date_in
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        from_dbtime(&self.date_in)
    }
    fn get_price_in(&self) -> &f64 {
        &self.price_in
    }
    fn get_price_out(&self) -> &f64 {
        &self.price_in
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeOut {
    pub id: usize,
    pub trade_type: TradeType,
    pub index_in: usize,
    pub price_in: f64,
    pub ask: f64,
    pub spread_in: f64,
    pub date_in: DbDateTime,
    pub index_out: usize,
    pub price_origin: f64,
    pub price_out: f64,
    pub bid: f64,
    pub spread_out: f64,
    pub date_out: DbDateTime,
    pub profit: f64,
    pub profit_per: f64,
    pub run_up: f64,
    pub run_up_per: f64,
    pub draw_down: f64,
    pub draw_down_per: f64,
}

impl Trade for TradeOut {
    fn get_date(&self) -> &DbDateTime {
        &self.date_out
    }
    fn get_chrono_date(&self) -> DateTime<Local> {
        from_dbtime(&self.date_out)
    }
    fn get_price_in(&self) -> &f64 {
        &self.price_in
    }
    fn get_price_out(&self) -> &f64 {
        &self.price_out
    }
}

impl std::fmt::Display for TradeIn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for TradeOut {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn resolve_trade_in(
    index: usize,
    trade_size: f64,
    instrument: &Instrument,
    pricing: &Pricing,
    trade_type: &TradeType,
    order: Option<&Order>,
) -> TradeResult {
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let order_engine = &env::var("ORDER_ENGINE").unwrap();
    let index = calculate_trade_index(index, order, &execution_mode);

    if trade_type.is_entry() {
        let spread = pricing.spread();
        let current_candle = instrument.data.get(index).unwrap();
        let current_date = current_candle.date();
        let id = uuid::generate_ts_id(current_date);

        let price = match order_engine.as_ref() {
            "broker" => match order {
                Some(order) => order.target_price,
                None => current_candle.open(),
            },
            _ => current_candle.open(),
        };

        let ask = match trade_type.is_long() {
            true => price + spread,
            false => price,
        };

        let price_in = match trade_type.is_long() {
            true => ask,
            false => price,
        };

        let quantity = calc::calculate_quantity(trade_size, price_in);

        let index_in = match execution_mode.is_back_test() {
            true => index,
            false => id,
        };

        TradeResult::TradeIn(TradeIn {
            id,
            index_in,
            origin_price: price,
            price_in,
            ask,
            spread,
            quantity,
            date_in: to_dbtime(current_date),
            trade_type: trade_type.clone(),
        })
    } else {
        TradeResult::None
    }
}

pub fn resolve_trade_out(
    index: usize,
    instrument: &Instrument,
    pricing: &Pricing,
    trade_in: &TradeIn,
    trade_type: &TradeType,
    order: Option<&Order>,
) -> TradeResult {
    let quantity = trade_in.quantity;
    let data = &instrument.data;
    let spread = pricing.spread();
    let trade_in_type = &trade_in.trade_type;
    let index_in = trade_in.index_in;
    let spread_in = trade_in.spread;
    let execution_mode = mode::from_str(&env::var("EXECUTION_MODE").unwrap());
    let non_profitable_outs = &env::var("NON_PROFITABLE_OUTS")
        .unwrap()
        .parse::<bool>()
        .unwrap();
    let order_engine = &env::var("ORDER_ENGINE").unwrap();

    let index = calculate_trade_index(index, order, &execution_mode);
    let current_candle = instrument.data.get(index).unwrap();
    let current_date = current_candle.date();
    let price_origin = *trade_in.get_price_in();

    let close_trade_price = match trade_type {
        TradeType::StopLossLong | TradeType::StopLossShort => order.unwrap().target_price,
        _ => current_candle.open(),
    };

    let price_out = match order_engine.as_ref() {
        "broker" => match order {
            Some(order) => order.target_price,
            None => close_trade_price,
        },
        _ => close_trade_price,
    };

    let (price_in, price_out) = match execution_mode.is_back_test() {
        true => match trade_in_type.is_long() {
            true => (trade_in.price_in, price_out),
            false => (trade_in.price_in, price_out + spread),
        },
        false => (trade_in.price_in, price_out),
    };

    let bid = match trade_type.is_long() {
        true => price_out + spread,
        false => price_out,
    };
    let index_out = index;

    let profit = match trade_in_type.is_long() {
        true => price_out - price_in,
        false => price_in - price_out,
    };

    let is_profitable = match profit {
        _ if profit > 0. => true,
        _ => false,
    };

    if trade_type.is_stop() && profit > 0. {
        log::error!(
            "Profitable stop loss! {} @ {:?} {} ",
            index,
            (price_in, price_out),
            profit
        )
    }

    let profit_check = match non_profitable_outs {
        true => true || trade_type.is_stop(),
        false => is_profitable || trade_type.is_stop(),
    };

    if profit_check {
        let date_out = to_dbtime(current_candle.date());

        let date_in = match execution_mode.is_back_test() {
            true => to_dbtime(instrument.data.get(index_in).unwrap().date()),
            false => to_dbtime(current_date),
        };

        let profit = match execution_mode.is_back_test() {
            true => calc::calculate_profit(quantity, price_in, price_out, trade_in_type),
            false => 0.,
        };

        let profit_per = match execution_mode.is_back_test() {
            true => calc::calculate_profit_per(price_in, price_out, trade_in_type),
            false => 0.,
        };

        let run_up = match execution_mode.is_back_test() {
            true => calc::calculate_runup(data, price_in, index_in, index, trade_in_type),
            false => 0.,
        };

        let run_up_per = match execution_mode.is_back_test() {
            true => calc::calculate_runup_per(run_up, price_in, trade_in_type),
            false => 0.,
        };

        let draw_down = match execution_mode.is_back_test() {
            true => calc::calculate_drawdown(data, price_in, index_in, index, trade_in_type),
            false => 0.,
        };

        let draw_down_per = match execution_mode.is_back_test() {
            true => calc::calculate_drawdown_per(draw_down, price_in, trade_in_type),
            false => 0.,
        };

        TradeResult::TradeOut(TradeOut {
            id: uuid::generate_ts_id(current_date),
            index_in,
            price_in,
            trade_type: trade_type.clone(),
            date_in,
            spread_in,
            ask: price_in,
            index_out,
            price_origin,
            price_out,
            bid,
            spread_out: pricing.spread(),
            date_out,
            profit,
            profit_per,
            run_up,
            run_up_per,
            draw_down,
            draw_down_per,
        })
    } else {
        log::warn!("Non profitable {:?} exit", trade_type);
        TradeResult::None
    }
}

pub fn calculate_trade_index(
    index: usize,
    order: Option<&Order>,
    execution_mode: &ExecutionMode,
) -> usize {
    match execution_mode.is_back_test() {
        true => index + 1,
        false => index,
    }
}
