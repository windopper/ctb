use std::{collections::VecDeque, time::Instant};

use crate::core::trade::{AskBid, Trade};

/// Trade Flow Delta
/// 특정 시간 동안의 (시장가 매수량 - 시장가 매도량)을 계산
pub fn calculate_trade_delta(trades: &VecDeque<(Instant, Trade)>) -> f64 {
    let mut aggressive_buy_volume = 0.0;
    let mut aggressive_sell_volume = 0.0;

    for (_, trade) in trades.iter() {
        match trade.ask_bid {
            AskBid::Ask => aggressive_buy_volume += trade.trade_volume,
            AskBid::Bid => aggressive_sell_volume += trade.trade_volume,
            AskBid::Unknown => {}
        }
    }

    aggressive_buy_volume - aggressive_sell_volume
}