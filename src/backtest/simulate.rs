use std::{cell::RefCell, collections::BTreeMap, io::Write, rc::Rc, time::Duration};

use chrono::Utc;
use colored::Colorize;
use tokio::sync::mpsc;

use crate::{backtest::{fetch::fetch_n_minute_candles, lib::{BacktestParams, BacktesterState}}, core::{candle::{Candle, CandleBase, CandleTrait}, 
orderbook::Orderbook, signal::{Signal, SignalReason}, ticker::Ticker, trade::{filter_trades_by_same_minute, is_trade_time_previous_minute, Trade}}, 
helper::footprint::{footprint, log_footprint, FootprintTrade, FootprintValue}, strategy::{lib::MarketState, of1::{calculate_of1_indicator_every_1mcandle, of1, Of1Params, Of1State}, 
orderbook, scalp, supertrend_ema, swc::{self, StrategyParams}, vi_rsi, vwap, vwma_ma, vwma_ma_grok}, 
upbit_api::realtime::lib::listen_realtime_data};

// Trade를 FootprintTrade로 변환하는 함수
fn convert_trade_to_footprint_trade(trade: &Trade) -> FootprintTrade {
    FootprintTrade {
        ask_bid: trade.ask_bid.clone(),
        price: trade.trade_price,
        volume: trade.trade_volume,
    }
}


pub struct SimulationConfig {
    pub enable_log: bool,
}

impl SimulationConfig {
    pub fn new() -> Self {
        Self {
            enable_log: true,
        }
    }
}


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


/// 실시간 백테스트
///
/// params:
/// - code: 종목 코드
/// - shutdown_recv: 종료 신호 수신 채널
pub async fn simulate_with_realtime_data(code: &str, shutdown_recv: &mut mpsc::Receiver<()>, config: &SimulationConfig) -> BacktesterState {
    println!("realtime backtest start - {}", code);
    
    let (top_n_trade_volume_avg_fn, log_footprint_fn, log_indicator_fn) = get_simulate_log_fns();

    let backtest_params = BacktestParams::default(code, "of1");
    let backtester = Rc::new(RefCell::new(BacktesterState::new(backtest_params)));

    let of1_state = Rc::new(RefCell::new(Of1State::new()));
    let of1_params = Of1Params::new();

    // 지표
    let top_n_trade_volume_avg = Rc::new(RefCell::new(0.0));

    // prefetch
    {
        println!("prefetching...");
        let formatted_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let pre_fetch_candles = fetch_n_minute_candles(code, 20, &formatted_time, 1).await.unwrap();
        for candle_trait in pre_fetch_candles {
            let candle = Candle {
                base: CandleBase {
                    market: candle_trait.get_market().to_string(),
                    candle_date_time_utc: candle_trait.get_candle_date_time_utc().to_string(),
                    candle_date_time_kst: candle_trait.get_candle_date_time_kst().to_string(),
                    opening_price: candle_trait.get_opening_price(),
                    high_price: candle_trait.get_high_price(),
                    low_price: candle_trait.get_low_price(),
                    trade_price: candle_trait.get_trade_price(),
                    timestamp: candle_trait.get_timestamp(),
                    candle_acc_trade_price: candle_trait.get_candle_acc_trade_price(),
                    candle_acc_trade_volume: candle_trait.get_candle_acc_trade_volume(),
                }
            };
            of1_state.borrow_mut().history_candles.push_back(candle);
        }
        println!("prefetching done");
    }

    // 미리 계산
    {
        let mut of1_state_ref = of1_state.borrow_mut();
        calculate_of1_indicator_every_1mcandle(&mut of1_state_ref, &of1_params);
    }
    
    // 클로저 정의
    let mut trade_fn = |trade: &Trade| {
        let mut of1_state_ref = of1_state.borrow_mut();
        of1_state_ref.trades.push(trade.clone());
    };

    let mut orderbook_fn = |orderbook: &Orderbook| {
        // println!("orderbook: {:?}", orderbook);
    };
    
    let mut candle_fn = |candle: &Candle| {
        let kst = candle.get_candle_date_time_kst();
        let mut of1_state_ref = of1_state.borrow_mut();
        if of1_state_ref.current_mutation_candle.is_some() {
            let current_mutation_candle_kst = of1_state_ref.current_mutation_candle.as_ref().unwrap().get_candle_date_time_kst();
            // 동일 시간대 캔들이면 계속 갱신
            if current_mutation_candle_kst == kst {
                of1_state_ref.current_mutation_candle = Some(candle.clone());
                return;
            }
        } else {
            of1_state_ref.current_mutation_candle = Some(candle.clone());
            return;
        }

        // 다른 시간대 캔들이면 이전 캔들을 히스토리에 추가하고 현재 캔들을 갱신
        let current_mutation_candle = of1_state_ref.current_mutation_candle.take().unwrap();
        let previous_kst = current_mutation_candle.get_candle_date_time_kst().to_string();

        of1_state_ref.history_candles.push_back(current_mutation_candle);
        of1_state_ref.current_mutation_candle = Some(candle.clone());

        // of1 지표 갱신
        calculate_of1_indicator_every_1mcandle(&mut of1_state_ref, &of1_params);

        // 이전 시간대(분 단위)의 거래 내역만 필터하여 FootprintTrade로 변환
        let filtered_trades = filter_trades_by_same_minute(&of1_state_ref.trades, &previous_kst);
        let footprint_trades = filtered_trades.iter().map(|trade| convert_trade_to_footprint_trade(trade)).collect::<Vec<FootprintTrade>>();
        let footprint = footprint(&footprint_trades);
        let recent_candle_10 = of1_state_ref.history_candles.iter().rev().take(10).cloned().collect::<Vec<Candle>>();
        *top_n_trade_volume_avg.borrow_mut() = top_n_trade_volume_avg_fn(&recent_candle_10);

        // 푸터프린트 출력
        if config.enable_log {
            log_footprint_fn(&footprint);
        }


        of1_state_ref.footprints.push(footprint);
        
        // 3분전 footprint 제거
        let current_kst = candle.get_candle_date_time_kst();
        let parsed_kst = chrono::NaiveDateTime::parse_from_str(current_kst, "%Y-%m-%dT%H:%M:%S");
        match parsed_kst {
            Ok(kst) => {
                let three_minutes_ago = kst - chrono::Duration::minutes(3);
                let three_minutes_ago_str = three_minutes_ago.format("%Y-%m-%dT%H:%M:%S").to_string();
                of1_state_ref.trades.retain(|trade| !is_trade_time_previous_minute(trade, &three_minutes_ago_str));
            }
            Err(_) => {
                println!("failed to parse kst: {}", current_kst);
            }
        }
    };

    let mut ticker_fn = |ticker: &Ticker| {
        let current_price = ticker.trade_price;
        let current_timestamp = ticker.trade_timestamp;
        let mut backtester_ref = backtester.borrow_mut();
        backtester_ref.check_and_close_position(current_price, &current_timestamp.to_string());
        let signal = of1(&mut of1_state.borrow_mut(), &of1_params, &mut backtester_ref.get_position());
        backtester_ref.handle_signal(&signal, current_price, &current_timestamp.to_string()); // 포지션 관리

        let of1_state_ref = of1_state.borrow();

        if of1_state_ref.current_mutation_candle.is_none() {
            return;
        }

        let indicator = Indicator {
            top_n_trade_volume_avg: *top_n_trade_volume_avg.borrow(),
            previous_candle: of1_state_ref.current_mutation_candle.as_ref().unwrap().clone(),
            current_price: current_price,
            current_candle_volume: of1_state_ref.current_mutation_candle.as_ref().unwrap().get_candle_acc_trade_volume(),
        };

        if config.enable_log {
            log_indicator_fn(&indicator);
        }
    };

    let mut exit_fn = || {
        let backtester_ref = backtester.borrow();
        let win_count = backtester_ref.win_count;
        let loss_count = backtester_ref.loss_count;
        let win_rate = win_count as f64 / (win_count + loss_count) as f64;
        let total_pnl_pct = backtester_ref.total_pnl_pct;

        println!("backtest result {} - [win: {} | loss: {} | win_rate: {:.2}% | total_pnl_pct: {:.2}%]", code, win_count, loss_count, win_rate * 100.0, total_pnl_pct * 100.0);
    };

    listen_realtime_data(code, shutdown_recv, &mut orderbook_fn, &mut trade_fn, &mut ticker_fn, &mut candle_fn, &mut exit_fn).await;
    
    backtester.borrow().clone()
}

pub struct Indicator {
    top_n_trade_volume_avg: f64,
    // 이전 캔들
    previous_candle: Candle,
    current_price: f64,
    current_candle_volume: f64,
}

pub fn get_simulate_log_fns() -> (impl Fn(&Vec<Candle>) -> f64, impl Fn(&BTreeMap<String, FootprintValue>), impl Fn(&Indicator)) {
    // 최근 10개 캔들의 거래량 평균
    let top_n_trade_volume_avg_fn = |candles: &Vec<Candle>| {
        let recent_candle_10 = candles.iter().rev().take(10).collect::<Vec<&Candle>>();
        let recent_candle_10_avg = recent_candle_10.iter().map(|c| c.get_candle_acc_trade_volume()).sum::<f64>() / recent_candle_10.len() as f64;
        recent_candle_10_avg
    };

    // 푸터프린트 출력
    let log_footprint_fn = |footprint: &BTreeMap<String, FootprintValue>| {
        let mut prices = footprint.keys().collect::<Vec<&String>>();
        prices.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let mut total_ask_vol = 0.0;
        let mut total_bid_vol = 0.0;

        for price in prices.iter() {
            let ask_vol = footprint[*price].ask_volume;
            let bid_vol = footprint[*price].bid_volume;

            total_ask_vol += ask_vol;
            total_bid_vol += bid_vol;
        }

        let volume_sum = total_ask_vol + total_bid_vol;
        
        println!();
        println!("footprint 총 거래량: {},  매수 비율: {} ", volume_sum, total_bid_vol / volume_sum);

        let footprint_vec = prices.iter().map(|p| (p.to_string(), footprint[*p].clone())).collect::<Vec<(String, FootprintValue)>>();
        log_footprint(footprint_vec);
    };

    let log_indicator_fn = |indicator: &Indicator| {
        print!("\r최근 10개 캔들 거래량 평균: {} | 이전 저가: {} | 이전 고가: {} | 이전 시가: {} | 이전 종가: {} | 현재가: {} | 현재 캔들 거래량: {}"
        , indicator.top_n_trade_volume_avg
        , indicator.previous_candle.get_low_price()
        , indicator.previous_candle.get_high_price()
        , indicator.previous_candle.get_opening_price()
        , indicator.previous_candle.get_trade_price()
        , indicator.current_price
        , indicator.current_candle_volume);
        std::io::stdout().flush().unwrap();
    };

    (top_n_trade_volume_avg_fn, log_footprint_fn, log_indicator_fn)
}