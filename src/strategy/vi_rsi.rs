use crate::{backtest::lib::PositionState, core::signal::{Signal, SignalReason},
    helper::{rsi::calculate_rsi, trend::{analyze_trend_moving_average, Trend}, vi::{calculate_vortex_indicator, ViCandle}},
    strategy::lib::MarketState
};

pub struct StrategyParams {

}

///
/// 보텍스 지표를 사용한 전략
/// 
/// 보텍스 지표가 강세 추세를 나타내고 RSI가 모멘텀이 강하지만 과매수 상태가 아닐 때, 매수 신호 발생
/// 손절선은 trailing stop을 사용하여 추적
pub fn run(state: &mut MarketState, params: &StrategyParams, current_position: &mut PositionState) -> Signal {
    let current_price = state.historical_candles.back().unwrap().get_trade_price();
    let closes: Vec<f64> = state.historical_candles.iter().map(|c| c.get_trade_price()).collect();

    let vi_candles = state.historical_candles.iter().map(|c| ViCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect();
    let vi = calculate_vortex_indicator(&vi_candles, 14).unwrap();

    let vi_pluses: Vec<f64> = vi.iter().map(|v| v.vi_plus).collect();
    let latest_vi_plus = vi_pluses.last().unwrap();
    let second_last_vi_plus = vi_pluses[vi_pluses.len() - 2];

    let vi_minuses: Vec<f64> = vi.iter().map(|v| v.vi_minus).collect();
    let latest_vi_minus = vi_minuses.last().unwrap();
    let second_last_vi_minus = vi_minuses[vi_minuses.len() - 2];

    let vi_plus_trend = analyze_trend_moving_average(&vi_pluses, 7, 14);
    
    let rsi = calculate_rsi(&closes, 14);
    let rsi_trend = analyze_trend_moving_average(&rsi,7, 14);
    let latest_rsi = *rsi.last().unwrap();

    let vi_cross_rsi = latest_vi_plus > latest_vi_minus && second_last_vi_plus < second_last_vi_minus;

    if vi_plus_trend == Some(Trend::Uptrend) && rsi_trend == Some(Trend::Uptrend) {
        println!("price: {}, VI+: {}, VI-: {}, RSI: {}", current_price, latest_vi_plus, latest_vi_minus, rsi.last().unwrap());
    }

    if let PositionState::None = current_position {
        if vi_plus_trend == Some(Trend::Uptrend) && rsi_trend == Some(Trend::Uptrend) && vi_cross_rsi && latest_rsi <= 70.0 {
            // println!("VI+: {}, VI-: {}, RSI: {}", latest_vi_plus, latest_vi_minus, rsi.last().unwrap());
            return Signal::Buy {
                reason: "보텍스 지표가 강세 추세를 나타내고 RSI가 모멘텀이 강하지만 과매수 상태가 아닐 때, 매수 신호 발생".to_string(),
                initial_trailing_stop: current_price - (0.05 * current_price),
                take_profit: current_price + (current_price),
                asset_pct: 1.0,
            };
        }
    } else if let PositionState::InPosition { entry_price: _, entry_asset: _, take_profit_price: _, trailing_stop_price } = current_position {
        if current_price < *trailing_stop_price {
            return Signal::Sell(SignalReason { reason: "추적 손절매 도달".to_string() });
        }

        let new_trailing_stop = current_price - (0.05 * current_price);
        if new_trailing_stop > *trailing_stop_price {
            return Signal::UpdateTrailingStop(new_trailing_stop);
        }
    }

    Signal::Hold
}