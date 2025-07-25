use std::time::Duration;

use crate::{backtest::lib::{BacktestParams, BacktesterState}, core::{candle::CandleTrait, signal::{Signal, SignalReason}}, strategy::{lib::MarketState, scalp, supertrend_ema, swc::{self, StrategyParams}, vi_rsi, vwma_ma}};



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

    let first_trade_utc = candles.first().unwrap().get_candle_date_time_utc().to_string();

    println!("first_trade_utc: {}", first_trade_utc);

    let last_price = candles.last().unwrap().get_trade_price();
    let last_candle_date_time_utc = candles.last().unwrap().get_candle_date_time_utc().to_string();

    for candle in candles {
        let current_price = candle.as_ref().get_trade_price();
        // println!("current_price: {}", current_price);
        backtester.check_and_close_position(current_price);
        let candle_date_time_utc = candle.get_candle_date_time_utc().to_string();
        state.historical_candles.push_back(candle);
        // let signal = swc::run(&mut state, &params, &mut backtester.get_position());
        // let signal = vi_rsi::run(&mut state, &vi_rsi::StrategyParams {}, &mut backtester.get_position());
        // let signal = supertrend_ema::run(&mut state, &supertrend_ema::StrategyParams {}, &mut backtester.get_position());
        // let signal = scalp::run(&mut state, &scalp::StrategyParams {}, &mut backtester.get_position());
        let signal = vwma_ma::run(&mut state, &vwma_ma::StrategyParams {}, &mut backtester.get_position());
        backtester.handle_signal(&signal, current_price, &candle_date_time_utc);
    }

    backtester.handle_signal(&Signal::Sell(SignalReason {
        reason: "End of test".to_string(),
    }), last_price, &last_candle_date_time_utc);

    backtester.print_results();
}