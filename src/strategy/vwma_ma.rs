use crate::{
    backtest::lib::PositionState, 
    core::{candle::CandleTrait, signal::{Signal, SignalReason}}, 
    helper::{level::find_support_resistance, previous::find_previous_trough_with_index, rsi::calculate_rsi, sma::calculate_sma, vwma::{calculate_vwma, VWMACandle}}, 
    strategy::lib::MarketState
};

pub struct StrategyParams {}

pub fn run(state: &mut MarketState, _params: &StrategyParams, position: &mut PositionState) -> Signal {
    let closes = state.historical_candles.iter().map(|c| c.get_trade_price()).collect::<Vec<_>>();
    let current_price = *closes.last().unwrap();
    let vwma_candles = state.historical_candles.iter().map(|c| VWMACandle {
        close: c.get_trade_price(),
        volume: c.get_candle_acc_trade_volume(),
    }).collect::<Vec<_>>();
    let vwma = calculate_vwma(&vwma_candles, 100);
    let vwma_last = vwma.last().unwrap().unwrap();

    let ema_50 = calculate_sma(&closes, 50);
    let ema_50_last = ema_50.unwrap();
    let ema_200 = calculate_sma(&closes, 200);
    let ema_200_last = ema_200.unwrap();

    let previous_trough = find_previous_trough_with_index(&closes[closes.len() - 10..], 9);
    let previous_trough_price = previous_trough.map(|(_, price)| price).unwrap_or(0.0);
    
    if let PositionState::None = position {
        // ema_50 < vwma < ema_200 일때, 현재가가 ema_50를 돌파하면 매수
        if current_price > ema_50_last && ema_50_last < vwma_last && ema_50_last < ema_200_last {
            println!("ema_50_last: {}, vwma_last: {}, ema_200_last: {}, current_price: {}", ema_50_last, vwma_last, ema_200_last, current_price);
            return Signal::Buy {
                reason: "VWMA MA Crossover".to_string(),
                initial_trailing_stop: previous_trough_price,
                take_profit: current_price * 2.0,
                asset_pct: 1.0,
            };
        }
    } else if let PositionState::InPosition { take_profit_price, trailing_stop_price, .. } = position {
        // 현재가가 vwma를 돌파했을 때 매도하면 이득일때 매도
        if current_price > vwma_last && current_price > *take_profit_price {
            return Signal::Sell(SignalReason {
                reason: "VWMA Breakout".to_string(),
            });
        }

        // 이전 저점 가격보다 낮아지면 매도
        if current_price < *trailing_stop_price {
            return Signal::Sell(SignalReason {
                reason: "Previous trough".to_string(),
            });
        }

        // 현재가가 손절가를 돌파하면 매도
        if current_price > *take_profit_price {
            return Signal::Sell(SignalReason {
                reason: "Take profit".to_string(),
            });
        }

        let new_trailing_stop = current_price - (0.01 * current_price);
        if new_trailing_stop > *trailing_stop_price {
            return Signal::UpdateTrailingStop(new_trailing_stop);
        }
    }

    Signal::Hold
}