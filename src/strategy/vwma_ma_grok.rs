use crate::{
    backtest::lib::PositionState,
    core::{candle::CandleTrait, signal::{Signal, SignalReason}},
    helper::{adx::{calculate_adx, AdxCandle}, atr::{calculate_atr, AtrCandle}, ema::calculate_ema, level::find_support_resistance, previous::find_previous_trough_with_index, rsi::calculate_rsi, sma::calculate_sma, vwma::{calculate_vwma, VWMACandle}},
    strategy::lib::MarketState
};

pub struct StrategyParams {
    ma_short_period: usize,
    ma_long_period: usize,
    vwma_period: usize,
    adx_period: usize,
    atr_multiplier: f64,
    risk_pct: f64,
}

impl Default for StrategyParams {
    fn default() -> Self {
        Self {
            ma_short_period: 20,
            ma_long_period: 200,
            vwma_period: 100,
            adx_period: 14,
            atr_multiplier: 2.0,
            risk_pct: 0.01,
        }
    }
}

pub fn run(state: &mut MarketState, params: &StrategyParams, position: &mut PositionState) -> Signal {
    let closes = state.historical_candles.iter().map(|c| c.get_trade_price()).collect::<Vec<_>>();
    let current_price = *closes.last().unwrap();
    let vwma_candles = state.historical_candles.iter().map(|c| VWMACandle {
        close: c.get_trade_price(),
        volume: c.get_candle_acc_trade_volume(),
    }).collect::<Vec<_>>();
    let adx_candles = state.historical_candles.iter().map(|c| AdxCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect::<Vec<_>>();
    let atr_candles = state.historical_candles.iter().map(|c| AtrCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect::<Vec<_>>();

    let vwma = calculate_vwma(&vwma_candles, params.vwma_period);
    let vwma_last = vwma.last().unwrap().unwrap();
    let ma_short = calculate_sma(&closes, params.ma_short_period).unwrap();
    let ma_long = calculate_sma(&closes, params.ma_long_period).unwrap();
    let ema_short = calculate_ema(&closes, 20);
    let rsi = calculate_rsi(&closes, 14);
    let rsi_last = rsi.last().unwrap();
    let adx = calculate_adx(&adx_candles, params.adx_period as u32);
    let adx_last = adx.last().unwrap();
    let atr = calculate_atr(&atr_candles, 14);
    let atr_last = atr.last().unwrap();

    let previous_trough = closes[closes.len() - 15..closes.len() - 5].iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let previous_trough_price = *previous_trough;

    if let PositionState::None = position {
        if current_price > ma_short && ma_short < vwma_last && ma_short < ma_long
            && adx_last.adx > 25.0
            && rsi_last > &50.0 && current_price > *ema_short.last().unwrap() {
            let stop_distance = current_price - previous_trough_price;
            
            return Signal::Buy {
                reason: "Enhanced VWMA Crossover".to_string(),
                initial_trailing_stop: previous_trough_price,
                take_profit: current_price + stop_distance * 2.0,
                asset_pct: 1.0,
            };
        }
    } else if let PositionState::InPosition { take_profit_price, trailing_stop_price, entry_price, .. } = position {
        let initial_stop = *entry_price * 0.99; // 1% initial stop
        if current_price < initial_stop {
            return Signal::Sell(SignalReason {
                reason: "Initial stop-loss".to_string(),
            });
        }
        if current_price < *trailing_stop_price {
            return Signal::Sell(SignalReason {
                reason: "Trailing stop".to_string(),
            });
        }
        if current_price > *take_profit_price {
            return Signal::Sell(SignalReason {
                reason: "Take profit".to_string(),
            });
        }
        let new_trailing_stop = current_price - (atr_last * params.atr_multiplier);
        if new_trailing_stop > *trailing_stop_price {
            return Signal::UpdateTrailingStop(new_trailing_stop);
        }
    }

    Signal::Hold
}