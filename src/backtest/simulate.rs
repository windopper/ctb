use std::time::Duration;

use crate::{backtest::lib::{BacktestParams, BacktesterState}, core::{candle::CandleTrait, signal::Signal}, strategy::{lib::MarketState, supertrend_ema, swc::{self, StrategyParams}, vi_rsi}};



pub fn simulate(candles: Vec<Box<dyn CandleTrait>>, history_candles: Vec<Box<dyn CandleTrait>>, backtester: &mut BacktesterState) {
    let mut state = MarketState::new();
    let params = StrategyParams {
        trade_delta_window: Duration::from_secs(1), // 최근 1초간의 거래를 분석
        obi_depth: 5,                                
        wall_krw_threshold: 5_000_000.0,
        atr_period: 14,
        atr_multiplier: 2.0,
        base_delta_threshold: 0.1,
        bb_period: 20,
        bb_multiplier: 2.0,
        adx_period: 14,
        rsi_period: 8,
        risk_reward_ratio: 2.0,
        atr_trailing_multiplier: 1.5,
    };

    state.historical_candles = history_candles.into_iter().map(|c| c as Box<dyn CandleTrait>).collect();

    for candle in candles {
        let current_price = candle.as_ref().get_trade_price();
        // println!("current_price: {}", current_price);
        backtester.check_and_close_position(current_price);
        // let signal = swc::run(&mut state, &params, &mut backtester.get_position());
        // let signal = vi_rsi::run(&mut state, &vi_rsi::StrategyParams {}, &mut backtester.get_position());
        let signal = supertrend_ema::run(&mut state, &supertrend_ema::StrategyParams {}, &mut backtester.get_position());
        backtester.handle_signal(&signal, current_price);
        state.historical_candles.push_back(candle);
    }

    backtester.print_results();
}