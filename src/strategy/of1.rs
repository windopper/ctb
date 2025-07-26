use std::collections::{BTreeMap, VecDeque};

use crate::{backtest::lib::PositionState, core::{candle::{Candle, CandleTrait}, orderbook::Orderbook, signal::{Signal, SignalReason}, ticker::Ticker, trade::Trade}, helper::{footprint::{footprint, FootprintValue}, orderbook::top_n_orderbook_ratio}};


pub struct Of1State {
    // pub recent_trades: &'a VecDeque<Trade>,
    // pub current_orderbook: &'a Orderbook,
    pub current_ticker: Option<Ticker>,
    pub history_candles: VecDeque<Candle>,
    pub footprints: Vec<BTreeMap<String, FootprintValue>>,

    // --- 세션 상태 ---
    pub absorb_price: Option<f64>, // 흡수 가격
    pub absorb_candle_low_price: Option<f64>, // 흡수 캔들 최저가
}

pub struct Of1Params {
    pub volume_threshold_multiplier: f64, // 의미있는 거래량으로 판단할 최소 거래량
    pub absorption_delta_ratio: f64, // 흡수로 판단할 CVD 델타 비율
    pub rr_ratio: f64, // 리스크 리워드 비율
}

impl<'a> Of1Params {
    pub fn new() -> Self {
        Self { volume_threshold_multiplier: 1.2, absorption_delta_ratio: 0.7, rr_ratio: 1.5 }
    }
}

impl Of1State {
    pub fn new() -> Self {
        Self { current_ticker: None, history_candles: VecDeque::new(), footprints: Vec::new(), 
            absorb_price: None, absorb_candle_low_price: None
        }
    }

    pub fn initialize_session(&mut self) {
        self.absorb_price = None;
        self.absorb_candle_low_price = None;
    }
}

pub fn of1(state: &mut Of1State, params: &Of1Params, position: &mut PositionState) -> Signal {
    if state.current_ticker.is_none() {
        return Signal::Hold;
    }

    if state.footprints.is_empty() {
        return Signal::Hold;
    }

    // history_candle, footprints 최대 200개 저장
    if state.history_candles.len() > 200 {
        state.history_candles.pop_front();
    }
    if state.footprints.len() > 200 {
        state.footprints.pop();
    }

    let current_candle = state.history_candles.back().unwrap();
    // let current_trade = state.recent_trades.back().unwrap();
    let current_price = state.current_ticker.as_ref().unwrap().trade_price;

    let recent_candle_20 = state.history_candles.iter().rev().take(20).collect::<Vec<&Candle>>();
    let candle_20_avg_volume = recent_candle_20.iter().map(|c| c.get_candle_acc_trade_volume()).sum::<f64>() / recent_candle_20.len() as f64;
    let volume_threshold = candle_20_avg_volume * params.volume_threshold_multiplier;

    let latest_footprint = state.footprints.last().unwrap();

    let mut footprint_sorted_keys = latest_footprint.keys().collect::<Vec<&String>>();
    footprint_sorted_keys.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let footprint_04_len_floor = (footprint_sorted_keys.len() as f64 * 0.4).floor() as usize;
    // 전체 중 40%만
    let footprint_sorted_keys = footprint_sorted_keys.iter().take(footprint_04_len_floor).cloned().collect::<Vec<&String>>();

    // 40% 아래의 footprint 가격 중 가장 높은 거래량이 volume_threshold 보다 크면 흡수
    let mut absorption_price = None;
    let mut absorption_volume = 0.0;
    for key in footprint_sorted_keys.iter() {
        let volume = latest_footprint[*key].ask_volume + latest_footprint[*key].bid_volume;
        if volume > volume_threshold && absorption_volume < volume {
            absorption_price = Some(key);
            absorption_volume = volume; 
        }
    }

    if let PositionState::None = position {
        // 흡수 가격 설정
        if let Some(absorption_price) = absorption_price && state.absorb_price.is_none() {
            state.absorb_price = Some(absorption_price.parse::<f64>().unwrap());
            state.absorb_candle_low_price = Some(current_candle.get_low_price());
            println!("흡수 가격 설정: {}", absorption_price);
        } 
        else if state.absorb_price.is_some() {
            let absorb_price = state.absorb_price.unwrap();
            let mut volume_sum = 0.0;
            let mut volume_count = 0;

            // 흡수 가격보다 현재 가격이 높고 이 보다 높은 가격에 대해 강력한 양수 델타 관찰
            for key in footprint_sorted_keys.iter() {
                // 흡수 가격보다 현재 가격이 높은 가격에 대해서만 확인
                if key.parse::<f64>().unwrap() <= current_price {
                    continue;
                }

                let ask_volume = latest_footprint[*key].ask_volume;
                let bid_volume = latest_footprint[*key].bid_volume;

                // 델타 관찰
                let delta = ask_volume - bid_volume;
                volume_sum += delta;
                volume_count += 1;
            }

            // 흡수 가격보다 현재 가격이 높고 이 보다 높은 가격에 대해 강력한 양수 델타 관찰
            let delta_ratio = volume_sum / volume_count as f64;

            if absorb_price < current_price && delta_ratio > params.absorption_delta_ratio {
                state.initialize_session();
                if let Some(absorb_candle_low_price) = state.absorb_candle_low_price {
                    return Signal::Buy {
                        reason: "OF1".to_string(),
                        initial_trailing_stop: absorb_candle_low_price,
                        take_profit: current_price + (current_price - absorb_candle_low_price) * params.rr_ratio,
                        asset_pct: 1.0,
                    };
                }  
            }

            if absorb_price > current_price {
                state.initialize_session();
                println!("흡수 가격 초기화");
                return Signal::Hold;
            }
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