use std::time::Duration;

use crate::{backtest::lib::{BacktestParams, BacktesterState}, core::{candle::CandleTrait, signal::{Signal, SignalReason}}, strategy::{lib::MarketState, orderbook, scalp, supertrend_ema, swc::{self, StrategyParams}, vi_rsi, vwap, vwma_ma, vwma_ma_grok}};


pub fn simulate(candles: Vec<Box<dyn CandleTrait>>, history_candles: Vec<Box<dyn CandleTrait>>, backtester: &mut BacktesterState) {
    let mut state = MarketState::new();
    
    state.historical_candles = history_candles.into_iter().map(|c| c as Box<dyn CandleTrait>).collect();

    let first_trade_utc = candles.first().unwrap().get_candle_date_time_utc().to_string();

    println!("first_trade_utc: {}", first_trade_utc);

    let last_price = candles.last().unwrap().get_trade_price();
    let last_candle_date_time_utc = candles.last().unwrap().get_candle_date_time_utc().to_string();

    for candle in candles {
        let current_price = candle.as_ref().get_trade_price();
        // println!("current_price: {}", current_price);
        let candle_date_time_utc = candle.get_candle_date_time_utc().to_string();
        backtester.check_and_close_position(current_price, &candle_date_time_utc);
        state.historical_candles.push_back(candle);

        // 최대 300개 캔들
        if state.historical_candles.len() > 300 {
            state.historical_candles.pop_front();
        }
        // let signal = swc::run(&mut state, &params, &mut backtester.get_position());
        // let signal = vi_rsi::run(&mut state, &vi_rsi::StrategyParams {}, &mut backtester.get_position());
        // let signal = supertrend_ema::run(&mut state, &supertrend_ema::StrategyParams {}, &mut backtester.get_position());
        // let signal = scalp::run(&mut state, &scalp::StrategyParams {}, &mut backtester.get_position());
        // let signal = vwma_ma::run(&mut state, &vwma_ma::StrategyParams {}, &mut backtester.get_position());
        // let signal = vwma_ma_grok::run(&mut state, &vwma_ma_grok::StrategyParams::default(), &mut backtester.get_position());
        // let signal = vwap::run(&mut state, &vwap::StrategyParams {}, &mut backtester.get_position());
        let signal = orderbook::run(&mut state, &orderbook::StrategyParams {}, &mut backtester.get_position());
        backtester.handle_signal(&signal, current_price, &candle_date_time_utc);
    }

    backtester.handle_signal(&Signal::Sell(SignalReason {
        reason: "End of test".to_string(),
    }), last_price, &last_candle_date_time_utc);

    backtester.print_results();
}