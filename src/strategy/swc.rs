use std::{collections::VecDeque, time::{Duration, Instant}};

use crate::{core::{candle::CandleTrait, orderbook::Orderbook, signal::Signal, trade::Trade}, helper::{adx::{calculate_adx, Ohlc}, atr::{calculate_atr, AtrCandle}, bollinger_bands::calculate_bollinger_bands, di::{calculate_di, DiCandle}, ema::calculate_ema, obi::calculate_obi, parabolic_sar::{calculate_parabolic_sar, ParabolicSarCandle}, rsi::calculate_rsi, tfd::calculate_trade_delta}};

pub struct StrategyParams {
    pub trade_delta_window: Duration, // 거래 흐름 델타를 계산할 시간 윈도우
    pub obi_depth: usize, // 호가창 불균형 계산할 호가 깊이
    pub wall_krw_threshold: f64, // 벽으로 간주할 최소 원화 가치

    // --- ATR 기반 동적 임계값 파라미터 ---
    pub atr_period: usize,           // ATR 계산 기간
    pub atr_multiplier: f64,         // ATR 값에 곱할 승수 (임계값 민감도 조절)
    pub base_delta_threshold: f64,   // 최소 기본 델타 임계값 (BTC 단위)

    // --- 볼린저 밴드 파라미터 ---
    pub bb_period: usize,            // 볼린저 밴드 계산 기간
    pub bb_multiplier: f64,          // 볼린저 밴드 표준편차 승수

    // --- ADX 파라미터 ---
    pub adx_period: u32,          // ADX 계산 기간

    // --- RSI 파라미터 ---
    pub min_rsi: f64,
    pub max_rsi: f64,
}

pub struct MarketState {
    pub recent_trades: VecDeque<(Instant, Trade)>,
    pub current_orderbook: Option<Orderbook>,
    // ATR과 볼린저 밴드 계산을 위한 과거 캔들 데이터
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

#[derive(Debug)]
pub enum Trend {
    Ranging,
    UpTrend,
    DownTrend,
}

pub fn run(state: &mut MarketState, params: &StrategyParams) -> Signal {
    // 1. 데이터 유효성 검사
    let required_data_points = params.atr_period.max(params.bb_period) + 1;
    if state.historical_candles.len() < required_data_points {
        return Signal::Hold;
    }

    // 2. 핵심 지표 계산
    let closes: Vec<f64> = state.historical_candles.iter().map(|c| c.get_trade_price()).collect();
    let atx_candles: Vec<Ohlc> = state.historical_candles.iter().map(|c| Ohlc { 
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect();
 
    
    let adx = calculate_adx(&atx_candles, params.adx_period);
    let last_adx = adx.last().unwrap().adx;

    let ema_9 = calculate_ema(&closes, 9);
    let last_ema_9 = ema_9.last().unwrap();
    let ema_21 = calculate_ema(&closes, 21);
    let last_ema_21 = ema_21.last().unwrap();
    // let last_atr = calculate_atr(&atr_candles, params.atr_period).pop().unwrap_or(0.0);
    let last_bb = calculate_bollinger_bands(&closes, params.bb_period, params.bb_multiplier).pop();
    let current_price = closes.last().unwrap_or(&0.0);
    let second_last_price = closes[closes.len() - 2];

    let rsi = calculate_rsi(&closes, 14);
    let last_rsi = *rsi.last().unwrap();

    let psar_candles: Vec<ParabolicSarCandle> = state.historical_candles.iter().map(|c| ParabolicSarCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
    }).collect();
    let psar = calculate_parabolic_sar(&psar_candles, 0.02, 0.2, 0.02);
    let last_psar = *psar.last().unwrap();

    let di_candle: Vec<DiCandle> = state.historical_candles.iter().map(|c| DiCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect();
    let di = calculate_di(&di_candle, 14);
    let last_di = di.last().unwrap();
    let plus_di = last_di.plus_di;
    let minus_di = last_di.minus_di;
    
    // 현재 지표 계산
    let delta = calculate_trade_delta(&state.recent_trades);
    let obi = calculate_obi(&state.current_orderbook, params.obi_depth);

    #[allow(unused_assignments)]
    // let mut current_trend = Trend::Ranging;


    if last_adx > 25.0 {
        // psar가 하락추세에서 상승추세로 전환하면 매수 신호
        let di_plus_high = plus_di > minus_di;
        let second_last_psar = psar[psar.len() - 2];
        if di_plus_high && second_last_psar > second_last_price && last_psar < *current_price {
            return Signal::Buy;
        } else if second_last_psar < second_last_price && last_psar > *current_price {
            return Signal::Sell;
        } else {
            return Signal::Hold;
        }
    } else if last_adx < 20.0 {
        return Signal::Hold;
    } else {
        return Signal::Hold;
    }

    // println!("current_trend: {:?}", current_trend);

    // return match current_trend {
    //     Trend::Ranging => {
    //         let last_bb_lower = last_bb.as_ref().map(|bb| bb.lower).unwrap_or(0.0);
    //         let last_bb_upper = last_bb.as_ref().map(|bb| bb.upper).unwrap_or(0.0);

    //         if rsi.len() >= 2 {
    //             let prev_rsi = rsi[rsi.len() - 2];
    //             let curr_rsi = rsi[rsi.len() - 1];

    //             if prev_rsi <= params.min_rsi && curr_rsi > params.min_rsi {
    //                 return Signal::Buy;
    //             } else if prev_rsi >= params.max_rsi && curr_rsi < params.max_rsi {
    //                 return Signal::Sell;
    //             }
    //         }
    //         Signal::Hold
    //     }
    //     Trend::UpTrend => {
    //         // 두 이평선 사이에 있는 가격인지 확인
    //         let price_in_buy_zone = current_price < last_ema_9 && current_price > last_ema_21;
    //         // 모멘텀 확인
    //         let rsi_in_healty_range = last_rsi >= 45.0 && last_rsi <= 55.0;
    //         // 21-ema 아래 가격인지 확인
    //         let price_below_21_ema = current_price < last_ema_21;

    //         if price_in_buy_zone && rsi_in_healty_range {
    //             Signal::Buy
    //         } 
    //         // 과매수 구간에 진입했거나 21-ema 아래에서 마감
    //         else if last_rsi > params.max_rsi || price_below_21_ema {
    //             Signal::Sell
    //         } else {
    //             Signal::Hold
    //         }
    //     }
    //     Trend::DownTrend => {
    //         let price_in_sell_zone = current_price > last_ema_9 && current_price < last_ema_21;
    //         let rsi_in_weak_bounce_range = last_rsi >= 45.0 && last_rsi <= 55.0;
    //         if price_in_sell_zone && rsi_in_weak_bounce_range && last_rsi < params.min_rsi && current_price > last_ema_21 {
    //             Signal::Sell
    //         } else {
    //             Signal::Hold
    //         }
    //     }
    // };

    // println!(
    //     "--- 델타: {:.4} BTC | OBI: {:.2}",
    //     delta, obi
    // );
    // println!(
    //     "--- 동적 매수 임계값: {:.4} | 동적 매도 임계값: {:.4} (ATR: {:.4}) ---",
    //     delta_buy_signal_threshold, delta_sell_signal_threshold, last_atr
    // );
    // println!(
    //     "--- 볼린저 밴드 상단: {:.2} | 하단: {:.2} | 하단 vs 현재가: {:.2} | 현재가 vs 상단: {:.2} ---",
    //     last_bb.as_ref().map(|bb| bb.upper).unwrap_or(0.0),
    //     last_bb.as_ref().map(|bb| bb.lower).unwrap_or(0.0),
    //     current_price - last_bb.as_ref().map(|bb| bb.lower).unwrap_or(0.0),
    //     last_bb.as_ref().map(|bb| bb.upper - current_price).unwrap_or(0.0)
    // );

    // if let Some(bb) = last_bb {
    //     // 매수 신호: (강력한 매수세가 매도벽 소진) AND (가격이 볼린저 밴드 하단 근처)
    //     if let Some(ob) = &state.current_orderbook {
    //         // 볼린저 밴드 필터 (가격이 과매도 구간이거나 하단에 근접)
    //         let bb_confirmation = *current_price <= bb.lower;
            
    //         if bb_confirmation {
    //             // println!("매수 신호: 델타 임계값 초과 및 볼린저 밴드 하단 터치");
    //             return Signal::Buy;
    //         }
    //     }

    //     // 볼린저 밴드 필터 (가격이 과매수 구간이거나 상단에 근접) 
    //     let bb_confirmation = *current_price >= bb.upper;

    //     if bb_confirmation {
    //         // println!("매도 신호: 델타 임계값 미만 및 볼린저 밴드 상단 터치");
    //         return Signal::Sell;
    //     }
    // }
    
    
    Signal::Hold
}