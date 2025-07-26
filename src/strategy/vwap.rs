use crate::{
    backtest::lib::PositionState, 
    core::signal::Signal, 
    helper::{adx::{calculate_adx, AdxCandle}, vwap_band::{calculate_vwap_bands, VwapCandle}}, 
    strategy::lib::MarketState
};

pub struct StrategyParams {}    

pub fn run(state: &mut MarketState, _params: &StrategyParams, position: &mut PositionState) -> Signal {
    let closes = state.historical_candles.iter().map(|c| c.get_trade_price()).collect::<Vec<_>>();
    let current_price = *closes.last().unwrap();

    let adx_candles = state.historical_candles.iter().map(|c| AdxCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
        close: c.get_trade_price(),
    }).collect::<Vec<_>>();

    let vwap_candles = state.historical_candles.iter().map(|c| VwapCandle {
        close: c.get_trade_price(),
        volume: c.get_candle_acc_trade_volume(),
        high: c.get_high_price(),
        low: c.get_low_price(),
    }).collect::<Vec<_>>();

    let vwap_band = calculate_vwap_bands(&vwap_candles, 14, 2.0);
    let vwap_band_last = vwap_band.last().unwrap().as_ref().unwrap();
    
    let adx = calculate_adx(&adx_candles, 14);
    let adx_last = adx.last().unwrap().adx;

    if let PositionState::None = position {
        if adx_last < 20.0 && vwap_band_last.lower_band < current_price {
            return Signal::Buy {
                reason: "ADX가 20 미만이고 VWAP 밴드가 형성되었습니다.".to_string(),
                initial_trailing_stop: current_price - (0.003 * current_price),
                take_profit: current_price * 1.009,
                asset_pct: 1.0,
            };
        }
    }

    Signal::Hold
}