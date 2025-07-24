use std::{collections::VecDeque, time::{Duration, Instant}};

use crate::{core::{candle::CandleTrait, orderbook::Orderbook, trade::Trade}};

pub struct MarketState {
    pub recent_trades: VecDeque<(Instant, Trade)>,
    pub current_orderbook: Option<Orderbook>,
    pub historical_candles: VecDeque<Box<dyn CandleTrait>>,
}


impl MarketState {
    pub fn new() -> Self {
        MarketState {
            recent_trades: VecDeque::new(),
            current_orderbook: None,
            historical_candles: VecDeque::new(),
        }
    }

    // 오래된 거래 기록을 제거하여 VecDeque를 항상 최신 상태로 유지
    pub fn prune_old_trades(&mut self, window: Duration) {
        let now = Instant::now();
        while let Some((timestamp, _)) = self.recent_trades.front() {
            if now.duration_since(*timestamp) > window {
                self.recent_trades.pop_front();
            } else {
                break;
            }
        }
    }

    // 오래된 캔들 데이터를 관리하는 함수
    pub fn prune_old_candles(&mut self, max_candles: usize) {
        while self.historical_candles.len() > max_candles {
            self.historical_candles.pop_front();
        }
    }
}