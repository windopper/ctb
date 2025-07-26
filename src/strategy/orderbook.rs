use crate::{backtest::lib::PositionState, core::signal::{Signal, SignalReason}, helper::williams_fractal::{calculate_williams_fractals, FractalCandle, FractalType}, strategy::lib::MarketState};

pub struct StrategyParams {}

// 
pub fn run(state: &mut MarketState, params: &StrategyParams, position: &mut PositionState) -> Signal {
    let current_price = state.historical_candles.back().unwrap().get_trade_price();
    
    let fractal_candles = state.historical_candles.iter().map(|c| FractalCandle {
        high: c.get_high_price(),
        low: c.get_low_price(),
    }).collect::<Vec<FractalCandle>>();

    let fractals = calculate_williams_fractals(&fractal_candles);
    
    // 지지선. 최근 2개의 캔들이 오차범위 5% 이내에 있는 경우
    // 없다면 가장 최근의 캔들의 지지선
    // 저항선도 마찬가지로 최근 2개의 캔들이 오차범위 5% 이내에 있는 경우
    // 없다면 가장 최근의 캔들의 저항선

    let supports : Vec<(usize, f64)> = fractals.iter().enumerate().filter_map(|(i, f)| {
        if let Some(FractalType::Bullish) = f {
            Some((i, fractal_candles[i].low))
        } else {
            None
        }
    }).collect::<Vec<(usize, f64)>>();

    let mut support_price_line = 0.0;
    let mut resistance_price_line = 0.0;
    let mut support_idx: Option<usize> = None;
    let mut resistance_idx: Option<usize> = None;

    if supports.len() >= 2 {
        let last = supports.len() - 1;
        let first_support_candle = fractal_candles[supports[last].0];
        let second_support_candle = fractal_candles[supports[last - 1].0];

        let first_support_price = first_support_candle.low;
        let second_support_price = second_support_candle.low;

        let support_price_diff = (first_support_price - second_support_price).abs();
        let support_price_diff_ratio = support_price_diff / first_support_price;

        if support_price_diff_ratio < 0.05 {
            support_price_line = (first_support_price + second_support_price) / 2.0;
            support_idx = Some(supports[last].0);
        } else {
            support_price_line = first_support_price;
            support_idx = Some(supports[last].0);
        }
    } else if (supports.len() == 1) {
        let last_support_candle = fractal_candles[supports[0].0];
        support_price_line = last_support_candle.low;
        support_idx = Some(supports[0].0);
    }
    
    let resistances : Vec<(usize, f64)> = fractals.iter().enumerate().filter_map(|(i, f)| {
        if let Some(FractalType::Bearish) = f {
            Some((i, fractal_candles[i].high))
        } else {
            None
        }
    }).collect::<Vec<(usize, f64)>>();

    if resistances.len() >= 2 {
        let last = resistances.len() - 1;
        let first_resistance_candle = fractal_candles[resistances[last].0];
        let second_resistance_candle = fractal_candles[resistances[last - 1].0];

        let first_resistance_price = first_resistance_candle.high;
        let second_resistance_price = second_resistance_candle.high;

        let resistance_price_diff = (first_resistance_price - second_resistance_price).abs();
        let resistance_price_diff_ratio = resistance_price_diff / first_resistance_price;

        if resistance_price_diff_ratio < 0.05 {
            resistance_price_line = (first_resistance_price + second_resistance_price) / 2.0;
            resistance_idx = Some(resistances[last].0);
        } else {
            resistance_price_line = first_resistance_price;
            resistance_idx = Some(resistances[last].0);
        }
    } else if (resistances.len() == 1) {
        let last_resistance_candle = fractal_candles[resistances[0].0];
        resistance_price_line = last_resistance_candle.high;
        resistance_idx = Some(resistances[0].0);
    }


    // 지지선 부근 왔을 때 매수
    if let PositionState::None = position {
        if support_price_line > 0.0 && support_price_line < resistance_price_line && current_price < support_price_line {
            // 로그 출력: 지지선/저항선 가격과 날짜
            if let (Some(s_idx), Some(r_idx)) = (support_idx, resistance_idx) {
                let support_date = state.historical_candles.get(s_idx).map(|c| c.get_candle_date_time_utc()).unwrap_or("N/A");
                let resistance_date = state.historical_candles.get(r_idx).map(|c| c.get_candle_date_time_utc()).unwrap_or("N/A");
                println!("[매수신호] 지지선: {} (날짜: {}), 저항선: {} (날짜: {})", support_price_line, support_date, resistance_price_line, resistance_date);
            }

            // 손익비 0.5% / 1%
            return Signal::Buy {
                reason: format!("Support price: {}, Resistance price: {}", support_price_line, resistance_price_line),
                initial_trailing_stop: current_price - 0.003 * current_price,
                take_profit: current_price + 0.006 * current_price,
                asset_pct: 1.0,
            };
        }
    } else if let PositionState::InPosition { entry_price, entry_asset, take_profit_price, trailing_stop_price } = position {
        if current_price > *take_profit_price {
            return Signal::Sell(SignalReason {
                reason: format!("Take profit price is reached: {}", take_profit_price),
            });
        }
    }

    Signal::Hold
}