use std::collections::{BTreeMap, VecDeque};

use crate::{backtest::lib::PositionState, core::{candle::{Candle, CandleTrait}, orderbook::Orderbook, signal::{Signal, SignalReason}, ticker::Ticker, trade::Trade}, helper::{footprint::{footprint, FootprintValue}, orderbook::top_n_orderbook_ratio}};


pub struct Of1State {
    // pub recent_trades: &'a VecDeque<Trade>,
    // pub current_orderbook: &'a Orderbook,
    pub current_ticker: Option<Ticker>,
    pub history_candles: VecDeque<Candle>,
    pub footprints: Vec<BTreeMap<String, FootprintValue>>,
    pub trades: Vec<Trade>,
    pub current_mutation_candle: Option<Candle>,

    // --- 세션 상태 ---
    pub absorb_price: Option<f64>, // 흡수 가격
    pub absorb_candle_low_price: Option<f64>, // 흡수 캔들 최저가

    // --- 지표 ---
    pub indicator: Of1Indicator,

}

#[derive(Clone)]
pub struct Of1Params {
    pub volume_threshold_multiplier: f64, // 의미있는 거래량으로 판단할 최소 거래량
    pub absorption_delta_ratio: f64, // 흡수로 판단할 CVD 델타 비율
    pub rr_ratio: f64, // 리스크 리워드 비율
    
    // --- 모멘텀 돌파 전략 ---
    pub momentum_volume_multiplier: f64,
    pub momentum_candle_range_multiplier: f64,
}

impl Of1Params {
    pub fn new() -> Self {
        Self { volume_threshold_multiplier: 1.2, absorption_delta_ratio: 0.7, rr_ratio: 1.5,
            momentum_volume_multiplier: 2.0, momentum_candle_range_multiplier: 2.0 }
    }
}

impl Of1State {
    pub fn new() -> Self {
        Self { current_ticker: None, history_candles: VecDeque::new(), footprints: Vec::new(), trades: Vec::new(), current_mutation_candle: None,
            absorb_price: None, absorb_candle_low_price: None,
            indicator: Of1Indicator::new()
        }
    }

    pub fn initialize_session(&mut self) {
        self.absorb_price = None;
        self.absorb_candle_low_price = None;
    }
}

pub struct Of1Indicator {
    pub top_n_trade_volume_avg: f64,
    pub candle_10_avg_volume: f64, // 10캔 평균 거래량
    pub candle_20_avg_candle_range: f64, // 20캔 평균 캔들 범위
    pub absorption_price: Option<f64>,
    pub footprint_sorted_keys: Vec<String>,
    pub footprint_delta_ratio: f64, // 매수 / 전체 거래량 비율
}

impl Of1Indicator {
    pub fn new() -> Self {
        Self { top_n_trade_volume_avg: 0.0, candle_10_avg_volume: 0.0,
             candle_20_avg_candle_range: 0.0, absorption_price: None,
              footprint_sorted_keys: Vec::new(), footprint_delta_ratio: 0.0 }
    }
}

/// of1 전략의 지표를 미리 계산하는 함수
pub fn calculate_of1_indicator_every_1mcandle(state: &mut Of1State, params: &Of1Params) {
    // 10캔 평균 거래량
    let recent_candle_10 = state.history_candles.iter().rev().take(10).collect::<Vec<&Candle>>();
    let candle_10_avg_volume = recent_candle_10.iter().map(|c| c.get_candle_acc_trade_volume()).sum::<f64>() / recent_candle_10.len() as f64;
    state.indicator.candle_10_avg_volume = candle_10_avg_volume;

    // 20캔 평균 캔들 범위
    let recent_candle_20 = state.history_candles.iter().rev().take(20).collect::<Vec<&Candle>>();
    let candle_20_avg_candle_range = recent_candle_20.iter().map(|c| (c.get_high_price() - c.get_low_price()).abs()).sum::<f64>() / recent_candle_20.len() as f64;
    state.indicator.candle_20_avg_candle_range = candle_20_avg_candle_range;

    // 흡수 가격 설정
    let latest_footprint = state.footprints.last();
    match latest_footprint {
        Some(footprint) => {
            let mut footprint_sorted_keys = footprint.keys().cloned().collect::<Vec<String>>();
            let footprint_sorted_keys_ref = footprint_sorted_keys.clone();
            footprint_sorted_keys.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mut absorption_price = None;
            let mut absorption_volume = 0.0;

            for key in footprint_sorted_keys_ref.iter() {
                let volume = footprint[key].ask_volume + footprint[key].bid_volume;
                if absorption_volume < volume {
                    absorption_price = Some(key);
                    absorption_volume = volume; 
                }
            }

            let mut volume_sum = 0.0;
            let mut ask_volume_sum = 0.0;
            let mut bid_volume_sum = 0.0;

            // 흡수 가격보다 현재 가격이 높고 이 보다 높은 가격에 대해 강력한 양수 델타 관찰
            for key in footprint_sorted_keys_ref.iter() {
                let ask_volume = footprint[key].ask_volume;
                let bid_volume = footprint[key].bid_volume;

                // 델타 관찰
                volume_sum += ask_volume + bid_volume;
                ask_volume_sum += ask_volume;
                bid_volume_sum += bid_volume;
            }

            state.indicator.footprint_delta_ratio = bid_volume_sum / volume_sum;
            state.indicator.footprint_sorted_keys = footprint_sorted_keys;
            state.indicator.absorption_price = absorption_price.map(|p| p.parse::<f64>().unwrap());
        }
        None => {
            return;
        }
    }
    
}

pub fn of1(state: &mut Of1State, params: &Of1Params, position: &mut PositionState) -> Signal {
    if state.current_ticker.is_none() {
        return Signal::Hold;
    }

    if state.footprints.is_empty() {
        return Signal::Hold;
    }

    if state.current_mutation_candle.is_none() {
        return Signal::Hold;
    }

    // history_candle, footprints 최대 200개 저장
    if state.history_candles.len() > 200 {
        state.history_candles.pop_front();
    }
    if state.footprints.len() > 200 {
        state.footprints.remove(0);
    }

    let last_minute_candle = state.history_candles.back().unwrap();
    let current_price = state.current_ticker.as_ref().unwrap().trade_price;

    let candle_10_avg_volume = state.indicator.candle_10_avg_volume;
    let volume_threshold = candle_10_avg_volume * params.volume_threshold_multiplier;

    let absorption_price = state.indicator.absorption_price;
    let is_current_volume_higher_than_threshold = last_minute_candle.get_candle_acc_trade_volume() > volume_threshold;

    if let PositionState::None = position {
        // 전략1. 흡수 가격 설정 후 추세 반전 관찰
        if absorption_price.is_some() && state.absorb_price.is_none() && is_current_volume_higher_than_threshold {
            state.absorb_price = absorption_price;
            state.absorb_candle_low_price = Some(last_minute_candle.get_low_price());
        } 
        else if state.absorb_price.is_some() && state.absorb_candle_low_price.is_some() {
            let absorb_price = state.absorb_price.unwrap();
            // 흡수 가격보다 현재 가격이 높고 이 보다 높은 가격에 대해 강력한 양수 델타 관찰
            let delta_ratio = state.indicator.footprint_delta_ratio;

            if absorb_price < current_price && delta_ratio > params.absorption_delta_ratio {
                if let Some(absorb_candle_low_price) = state.absorb_candle_low_price {
                    state.initialize_session();
                    return Signal::Buy {
                        reason: format!("OF1 Absorption | 흡수 가격: {} | 흡수 캔들 저가: {}", absorb_price, absorb_candle_low_price),
                        initial_trailing_stop: absorb_candle_low_price,
                        take_profit: current_price + (current_price - absorb_candle_low_price) * params.rr_ratio,
                        asset_pct: 1.0,
                    };
                }  
                state.initialize_session();
            }

            // 흡수 캔들 저가보다 현재 가격이 낮으면 포지션 초기화
            if state.absorb_candle_low_price.is_some() && state.absorb_candle_low_price.unwrap() > current_price {
                state.initialize_session();
            }
        }

        // 전략2. 양봉 움직임 && 현재까지의 거래량이 평균을 초과 && 현재까지의 캔들 폭이 평균을 초과
        let mut_candle = state.current_mutation_candle.as_ref().unwrap();
        let current_volume = mut_candle.get_candle_acc_trade_volume();
        let current_range = mut_candle.get_high_price() - mut_candle.get_low_price();

        let avg_volume = state.indicator.candle_10_avg_volume;
        let avg_range = state.indicator.candle_20_avg_candle_range;

        let is_bullish = mut_candle.get_opening_price() < mut_candle.get_trade_price();
        if is_bullish &&
        current_volume > avg_volume * params.momentum_volume_multiplier &&
        current_range > avg_range * params.momentum_candle_range_multiplier {
            // 필요한 값들을 미리 추출
            let stop_loss = mut_candle.get_low_price();
            let entry_price = mut_candle.get_trade_price();
            
            state.initialize_session();

            return Signal::Buy {
                reason: format!("OF1 Momentum | 캔들 길이 배수: {}배 | 거래량 배수: {}배", current_range / avg_range, current_volume / avg_volume),
                initial_trailing_stop: stop_loss,
                take_profit: entry_price + (entry_price - stop_loss) * params.rr_ratio,
                asset_pct: 1.0,
            };
        }
    } else if let PositionState::InPosition { entry_price, entry_asset, take_profit_price, trailing_stop_price } = position {
        if current_price > *take_profit_price {
            return Signal::Sell(SignalReason { reason: "OF1 Take Profit".to_string() });
        }

        if current_price < *trailing_stop_price {
            return Signal::Sell(SignalReason { reason: "OF1 Trailing Stop".to_string() });
        }
    }

    Signal::Hold
}