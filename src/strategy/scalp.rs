// 필요한 모듈과 타입을 가져옵니다.
use crate::{
    backtest::lib::PositionState, 
    core::{candle::CandleTrait, signal::{Signal, SignalReason}}, 
    helper::{level::find_support_resistance, rsi::calculate_rsi, sma::calculate_sma}, 
    strategy::lib::MarketState
};

// 전략에 사용될 파라미터 (향후 확장성을 위해 남겨둠)
pub struct StrategyParams {}

// 지지/저항선 근접 여부를 판단하기 위한 허용 오차 (1% = 0.01)
const PROXIMITY_TOLERANCE: f64 = 0.01;
// 최소 진입 손익비 (예: 2.0 -> 수익이 손실의 2배 이상일 때만 진입)
const MIN_RISK_REWARD_RATIO: f64 = 2.0;
// RSI 과매수/과매도 기준
const RSI_OVERBOUGHT: f64 = 70.0;
const RSI_OVERSOLD: f64 = 30.0; // 매수 조건에 활용 가능

pub fn run(state: &mut MarketState, _params: &StrategyParams, position: &mut PositionState) -> Signal {
    // --- 1. 데이터 및 지표 준비 ---
    if state.historical_candles.len() < 20 { // 최소 캔들 수 확인 (가장 긴 이평선 기간 이상)
        return Signal::Hold;
    }

    let last_candle = state.historical_candles.back().unwrap();
    let closes = state.historical_candles.iter().map(|c| c.get_trade_price()).collect::<Vec<_>>();
    let current_price = *closes.last().unwrap();
    
    // 지지/저항선 계산
    let candles_for_levels: Vec<&Box<dyn CandleTrait>> = state.historical_candles.iter().collect();
    let (supports, resistances) = find_support_resistance(candles_for_levels, 0.01, 3, 10);

    // 이동평균선 및 RSI 계산
    let short_sma = calculate_sma(&closes[closes.len().saturating_sub(5)..], 5).unwrap_or(0.0);
    let long_sma = calculate_sma(&closes[closes.len().saturating_sub(10)..], 10).unwrap_or(0.0);
    let rsi = calculate_rsi(&closes, 14);
    // let last_rsi = rsi.last().unwrap_or(&50.0); // 기본값 50 (중립)

    // --- 2. 매매 결정 로직 ---

    if let PositionState::None = position {
        // --- 2-1. 매수(진입) 결정 ---
        let avg_support = supports.iter().sum::<f64>() / supports.len() as f64;
        // 조건 1: 가격이 지지선 근처에서 반등했는가?
        let is_bouncing_off_support = last_candle.get_low_price() <= avg_support * (1.0 + PROXIMITY_TOLERANCE) 
            && current_price > avg_support;

        if is_bouncing_off_support {
            // 조건 2 (필터): 과매수가 아니고 단기 상승 추세인가?
            if short_sma > long_sma {
                // 손익비 계산
                let stop_loss_price = avg_support * (1.0 - PROXIMITY_TOLERANCE); // 손절은 지지선 바로 아래

                // 익절 목표는 현재가보다 높은 첫 번째 저항선으로 설정
                let take_profit_target = resistances
                    .iter()
                    .find(|&&r| r > current_price)
                    .cloned()
                    .unwrap_or(current_price * 1.05); // 저항선이 없으면 5% 위로 설정

                let risk = current_price - stop_loss_price;
                let reward = take_profit_target - current_price;

                if risk > 0.0 && (reward / risk) >= MIN_RISK_REWARD_RATIO {
                    return Signal::Buy {
                        reason: format!("Bounce off support at {:.2} with favorable R/R", avg_support),
                        initial_trailing_stop: stop_loss_price,
                        take_profit: take_profit_target,
                        asset_pct: 1.0, // 자산의 100%를 사용
                    };
                }
            }
        }
    } else if let PositionState::InPosition { take_profit_price, trailing_stop_price, .. } = position {
        // --- 2-2. 매도(청산) 결정 ---

        // 조건 1: 익절가 도달
        if last_candle.get_high_price() >= *take_profit_price {
            return Signal::Sell(SignalReason {
                reason: "Take profit target reached".to_string(),
            });
        }

        // 조건 2: 손절가 도달
        if last_candle.get_low_price() <= *trailing_stop_price {
            return Signal::Sell(SignalReason {
                reason: "Stop loss triggered".to_string(),
            });
        }
        
        // 조건 3: 추세 전환 신호 (데드 크로스)
        if short_sma < long_sma {
            return Signal::Sell(SignalReason {
                reason: "Trend reversal signal (SMA cross down)".to_string(),
            });
        }
        
        // 조건 4: RSI 과매수 신호
        // if last_rsi > &RSI_OVERBOUGHT {
        //     return Signal::Sell(SignalReason {
        //         reason: format!("RSI overbought ({:.2})", last_rsi),
        //     });
        // }
    } 

    // 위 조건들에 해당하지 않으면 포지션을 유지하거나 관망합니다.
    Signal::Hold
}