use crate::{backtest::lib::PositionState, core::signal::{Signal, SignalReason},
    helper::{ema::calculate_ema, macd::calculate_macd, rsi::calculate_rsi, supertrend::{calculate_supertrend, Ohlcv}, trend::{analyze_trend_moving_average, Trend}, vi::{calculate_vortex_indicator, ViCandle}},
    strategy::lib::MarketState
};

pub struct StrategyParams {

}

pub fn run(state: &mut MarketState, params: &StrategyParams, current_position: &mut PositionState) -> Signal {
    let current_price = state.historical_candles.back().unwrap().get_trade_price();
    let closes = state.historical_candles.iter().map(|c| c.get_trade_price()).collect::<Vec<_>>();
    let ohlcvs = state.historical_candles.iter().map(|c| Ohlcv {
        open: c.get_opening_price(),
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
        volume: c.get_candle_acc_trade_volume(),
    }).collect::<Vec<_>>();

    let supertrend = calculate_supertrend(&ohlcvs, 14, 3.0);
    let last_supertrend = supertrend.last().unwrap();

    let ema_5 = calculate_ema(&closes, 5);
    let last_ema_5 = ema_5.last().unwrap();
    let second_last_ema_5 = ema_5[ema_5.len() - 2];
    let ema_20 = calculate_ema(&closes, 20);
    let last_ema_20 = ema_20.last().unwrap();
    let second_last_ema_20 = ema_20[ema_20.len() - 2];

    let macd = calculate_macd(&closes, 12, 26, 9);
    let last_macd = macd.unwrap();
    let is_last_macd_positive = last_macd > 0.0;

    let ema_5_higher_than_20 = last_ema_5 > last_ema_20;
    let ema_5_lower_than_20 = last_ema_5 < last_ema_20;

    if let PositionState::None = current_position {
        if last_supertrend.is_uptrend && ema_5_higher_than_20 {
            return Signal::Buy {
                reason: "Supertrend EMA Golden Cross".to_string(),
                initial_trailing_stop: current_price - (0.01 * current_price),
                take_profit: current_price + (0.2 * current_price),
            }
        } 
    } else if let PositionState::InPosition { entry_price: _, take_profit_price: _, trailing_stop_price } = current_position {
        if current_price < *trailing_stop_price {
            return Signal::Sell(SignalReason { reason: "추적 손절매 도달".to_string() });
        }

        let new_trailing_stop = current_price - (0.01 * current_price);
        if new_trailing_stop > *trailing_stop_price {
            return Signal::UpdateTrailingStop(new_trailing_stop);
        }

        if !last_supertrend.is_uptrend && ema_5_lower_than_20 {
            return Signal::Sell(SignalReason { reason: "Supertrend EMA Death Cross".to_string() });
        }
    }

    Signal::Hold
}