use std::time::Duration;

use crate::{
    backtest::lib::PositionState, core::{
        signal::{Signal, SignalReason}, 
    }, helper::{
        adx::{calculate_adx, AdxCandle}, 
        atr::{calculate_atr, AtrCandle}, 
        bollinger_bands::calculate_bollinger_bands, 
        di::{calculate_di, DiCandle}, 
        previous::find_previous_trough_with_index, 
        rsi::calculate_rsi, 
    }, strategy::lib::MarketState
};


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
    pub rsi_period: usize,           // RSI 계산 기간
    
    // --- 리스크 관리 파라미터 ---
    pub risk_reward_ratio: f64,      // 최소 리스크/보상 비율 (예: 2.0)
    pub atr_trailing_multiplier: f64, // 추적 손절매에 사용할 ATR 승수 (예: 1.5)
}

pub fn run(state: &mut MarketState, params: &StrategyParams, current_position: &mut PositionState) -> Signal {
    // 1. 데이터 유효성 검사: 전략에 필요한 최소 캔들 수 확인
    let required_data_points = params.bb_period
        .max(params.adx_period as usize)
        .max(params.atr_period) + 5; // ATR 기간도 고려
    if state.historical_candles.len() < required_data_points {
        return Signal::Hold;
    }

    // --- 2. 핵심 지표 계산 ---
    let closes: Vec<f64> = state.historical_candles.iter().map(|c| c.get_trade_price()).collect();
    let current_price = *closes.last().unwrap();

    // ATR 계산 (리스크 및 추적 손절에 사용)
    let atr_candles: Vec<AtrCandle> = state.historical_candles.iter().map(|c| AtrCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect();
    let last_atr = match calculate_atr(&atr_candles, params.atr_period).pop() {
        Some(atr) if atr > 0.0 => atr,
        _ => return Signal::Hold, // ATR 계산 불가 시 거래 중지
    };

    // --- 로직 분기: 현재 포지션 상태에 따라 다른 로직 수행 ---
    match current_position {
        // A. 포지션이 없는 경우: 진입 조건 확인
        PositionState::None => {
            // --- 지표 계산 (진입에 필요한) ---
            let rsi_period = 8; // 전략에 명시된 8 사용
            let rsi = calculate_rsi(&closes, rsi_period);
            let last_rsi = *rsi.last().unwrap();

            let bb = match calculate_bollinger_bands(&closes, params.bb_period, params.bb_multiplier).pop() {
                Some(bb_values) => bb_values,
                None => return Signal::Hold,
            };

            let adx_candles: Vec<AdxCandle> = state.historical_candles.iter().map(|c| AdxCandle {
                high: c.get_high_price(), low: c.get_low_price(), close: c.get_trade_price(),
            }).collect();
            let adx = calculate_adx(&adx_candles, params.adx_period);
            let last_adx = adx.last().unwrap().adx;
            if adx.len() < 2 { return Signal::Hold; }
            let second_last_adx = adx[adx.len() - 2].adx;
            
            let di_candle: Vec<DiCandle> = state.historical_candles.iter().map(|c| DiCandle {
                high: c.get_high_price(), low: c.get_low_price(), close: c.get_trade_price(),
            }).collect();
            let di = calculate_di(&di_candle, params.adx_period as usize);
            let last_di = di.last().unwrap();
            if di.len() < 3 { return Signal::Hold; } // 최소 3개의 DI 값이 필요하다고 가정
            let second_last_di = &di[di.len() - 2];
            let third_last_di = &di[di.len() - 3];

            // --- 강세(Bullish) 진입 조건 분석 ---
            if let Some((prev_low_index, prev_low_price)) = find_previous_trough_with_index(&closes, closes.len() - 2) {
                let prev_low_rsi = rsi[prev_low_index];
                
                // 조건 1: 강세 RSI 다이버전스
                let bullish_divergence = current_price < prev_low_price && last_rsi > prev_low_rsi;

                // 조건 2: 볼린저 밴드 확인
                let bullish_bb_confirmation = current_price > bb.lower;

                // 조건 3: ADX 추세 필터
                let bullish_trend_filter = last_adx > 20.0 && last_adx > second_last_adx 
                && last_di.minus_di < second_last_di.minus_di &&
                second_last_di.minus_di < third_last_di.minus_di;
                
                // 조건 4: 리스크/보상 비율 확인 (보상 > 리스크 * 2)
                let risk = params.atr_multiplier * last_atr; // 손절폭 (리스크)
                let reward = bb.upper - current_price;     // 이익실현 목표(중단선)까지의 거리 (보상)
                let risk_reward_filter = reward > 0.0 && risk > 0.0 && (reward / risk) >= 2.0;
                
                if bullish_divergence {
                    println!(
                        "[{}] 가격: {:.2} | Div: {} | BB: {} | ADX: {} (val:{:.2}) | R/R: {} (R:{:.2}/r:{:.2} | ATR: {:.2})",
                        state.historical_candles.back().unwrap().get_timestamp(), // 현재 캔들 시간
                        current_price,
                        bullish_divergence,
                        bullish_bb_confirmation,
                        bullish_trend_filter,
                        last_adx,
                        risk_reward_filter,
                        reward,
                        risk,
                        last_atr
                    );
                }

                // 모든 강세 조건 충족 시 매수 신호 발생
                if bullish_divergence && bullish_bb_confirmation  && risk_reward_filter && bullish_trend_filter {
                    let initial_stop = current_price - risk;
                    let take_profit_target = bb.middle;
                    
                    return Signal::Buy {
                        reason: "강세 다이버전스 및 볼린저 밴드 확인".to_string(),
                        initial_trailing_stop: initial_stop,
                        take_profit: take_profit_target,
                        asset_pct: 1.0,
                    };
                }
            }
            Signal::Hold
        }

        // B. 포지션이 있는 경우: 종료 조건 확인
        PositionState::InPosition { entry_price: _, entry_asset: _, take_profit_price: _, trailing_stop_price } => {
            // 조건 1: 추적 손절매 가격 도달 시 매도
            if current_price < *trailing_stop_price {
                let reason = format!("추적 손절매 도달 (Price: {:.2} < Stop: {:.2})", current_price, trailing_stop_price);
                return Signal::Sell(SignalReason { reason });
            }

            // 조건 2: 추적 손절매 가격 업데이트
            // 현재 가격에서 ATR 기반의 손절폭을 뺀 가격이 기존 추적 손절 가격보다 높으면, 손절선을 위로 올림 (이익 보존)
            let new_trailing_stop = current_price - (params.atr_multiplier * last_atr);
            if new_trailing_stop > *trailing_stop_price {
                return Signal::UpdateTrailingStop(new_trailing_stop);
            }

            Signal::Hold
        }
    }
}