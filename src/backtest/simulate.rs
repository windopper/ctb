use std::{cell::RefCell, collections::{BTreeMap, HashMap}, io::Write, rc::Rc};

use chrono::Utc;
use tokio::sync::mpsc;

use crate::{backtest::{fetch::fetch_n_minute_candles, lib::{BacktestParams, BacktesterState}}, core::{candle::{Candle, CandleBase, CandleTrait}, 
orderbook::Orderbook, signal::{Signal, SignalReason}, ticker::Ticker, trade::{filter_trades_by_same_minute, is_trade_time_previous_minute, Trade}}, 
helper::footprint::{footprint, log_footprint, FootprintTrade, FootprintValue}, strategy::{candle_pattern, lib::MarketState, of1::{calculate_of1_indicator_every_1mcandle, of1, Of1Params, Of1State}, orderbook}, 
upbit_api::realtime::lib::{listen_realtime_data, RealtimeCallback}};

// Trade를 FootprintTrade로 변환하는 함수
fn convert_trade_to_footprint_trade(trade: &Trade) -> FootprintTrade {
    FootprintTrade {
        ask_bid: trade.ask_bid.clone(),
        price: trade.trade_price,
        volume: trade.trade_volume,
    }
}


#[derive(Clone)]
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


pub fn simulate(candles: Vec<Candle>, backtester: &mut BacktesterState) {
    // let mut state = MarketState::new();
    let mut state = candle_pattern::CandlePatternStrategyState::new();
    let config = candle_pattern::CandlePatternStrategyConfig::new();
    // state.historical_candles = history_candles.into_iter().map(|c| c as Box<dyn CandleTrait>).collect();

    let first_trade_utc = candles.first().unwrap().get_candle_date_time_utc().to_string();
    println!("first_trade_utc: {}", first_trade_utc);

    let last_price = candles.last().unwrap().get_trade_price();
    let last_candle_date_time_utc = candles.last().unwrap().get_candle_date_time_utc().to_string();

    for candle in candles {
        let current_price = candle.get_trade_price();
        // println!("current_price: {}", current_price);
        let candle_date_time_utc = candle.get_candle_date_time_utc().to_string();
        backtester.check_and_close_position(current_price, &candle_date_time_utc);
        let signal = candle_pattern::candle_pattern_strategy(&mut state, &config, &mut backtester.get_position(), Some(candle));
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
/// - codes: 종목 코드 배열
/// - shutdown_recv: 종료 신호 수신 채널
pub async fn simulate_with_realtime_data(codes: &[&str], shutdown_recv: &mut mpsc::Receiver<()>, config: &SimulationConfig) -> Vec<BacktesterState> {
    println!("realtime backtest start - codes: {:?}", codes);
    
    let mut backtesters = Vec::new();
    let mut callback_maps = HashMap::new();

    // 각 코드에 대해 백테스터와 상태 초기화
    for &code in codes {
        let backtest_params = BacktestParams::default(code, "of1");
        let backtester = Rc::new(RefCell::new(BacktesterState::new(backtest_params)));
        let of1_state = Rc::new(RefCell::new(Of1State::new()));
        let of1_params = Of1Params::new();
        let top_n_trade_volume_avg = Rc::new(RefCell::new(0.0));

        // prefetch
        {
            println!("prefetching for {}...", code);
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
            println!("prefetching done for {}", code);
        }

        // 미리 계산
        {
            let mut of1_state_ref = of1_state.borrow_mut();
            calculate_of1_indicator_every_1mcandle(&mut of1_state_ref, &of1_params);
        }

        let (top_n_trade_volume_avg_fn, log_footprint_fn, log_indicator_fn) = get_simulate_log_fns();
        
        // 클로저 정의
        let trade_fn = {
            let of1_state = of1_state.clone();
            move |trade: &Trade| {
                let mut of1_state_ref = of1_state.borrow_mut();
                of1_state_ref.trades.push(trade.clone());
            }
        };

        let orderbook_fn = |_orderbook: &Orderbook| {
            // println!("orderbook: {:?}", orderbook);
        };
        
        let candle_fn = {
            let of1_state = of1_state.clone();
            let of1_params = of1_params.clone();
            let top_n_trade_volume_avg = top_n_trade_volume_avg.clone();
            let top_n_trade_volume_avg_fn = top_n_trade_volume_avg_fn.clone();
            let log_footprint_fn = log_footprint_fn.clone();
            let config = config.clone();
            move |candle: &Candle| {
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

                if config.enable_log {
                    // 이전 캔들 폭 계산
                    let previous_candle = of1_state_ref.history_candles.back().unwrap();
                    let range = (previous_candle.get_high_price() - previous_candle.get_low_price()).abs();
                    let avg_range = of1_state_ref.indicator.candle_20_avg_candle_range;
                    let volume = previous_candle.get_candle_acc_trade_volume();
                    let avg_volume = of1_state_ref.indicator.candle_10_avg_volume;
                    let bullish = previous_candle.get_opening_price() < previous_candle.get_trade_price();
                    println!("\nrange: {} | avg_range: {} | volume: {} | avg_volume: {} | bullish: {}", range, avg_range, volume, avg_volume, bullish);

                    log_footprint_fn(&footprint);
                }

                // 새로운 footprint 추가
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
            }
        };

        let ticker_fn = {
            let of1_state = of1_state.clone();
            let of1_params = of1_params.clone();
            let backtester = backtester.clone();
            let top_n_trade_volume_avg = top_n_trade_volume_avg.clone();
            let log_indicator_fn = log_indicator_fn.clone();
            let config = config.clone();
            move |ticker: &Ticker| {
                let current_price = ticker.trade_price;
                let current_timestamp = ticker.trade_timestamp;
                let mut backtester_ref = backtester.borrow_mut();
                let mut of1_state_ref = of1_state.borrow_mut();
                of1_state_ref.current_ticker = Some(ticker.clone());

                backtester_ref.check_and_close_position(current_price, &current_timestamp.to_string());
                let signal = of1(&mut of1_state_ref, &of1_params, &mut backtester_ref.get_position());
                backtester_ref.handle_signal(&signal, current_price, &current_timestamp.to_string()); // 포지션 관리

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
            }
        };

        let exit_fn = {
            let backtester = backtester.clone();
            let code = code.to_string();
            move || {
                let backtester_ref = backtester.borrow();
                let win_count = backtester_ref.win_count;
                let loss_count = backtester_ref.loss_count;
                let win_rate = win_count as f64 / (win_count + loss_count) as f64;
                let total_pnl_pct = backtester_ref.total_pnl_pct;

                println!("backtest result {} - [win: {} | loss: {} | win_rate: {:.2}% | total_pnl_pct: {:.2}%]", code, win_count, loss_count, win_rate * 100.0, total_pnl_pct * 100.0);
            }
        };

        // 콜백 맵에 추가
        callback_maps.insert(code, RealtimeCallback {
            orderbook_fn: Box::new(orderbook_fn),
            trade_fn: Box::new(trade_fn),
            ticker_fn: Box::new(ticker_fn),
            candle_fn: Box::new(candle_fn),
            exit_fn: Box::new(exit_fn),
        });

        backtesters.push(backtester);
    }

    listen_realtime_data(codes, shutdown_recv, &mut callback_maps).await;
    
    // 모든 백테스터 결과 반환
    backtesters.into_iter().map(|backtester| backtester.borrow().clone()).collect()
}

pub struct Indicator {
    top_n_trade_volume_avg: f64,
    // 이전 캔들
    previous_candle: Candle,
    current_price: f64,
    current_candle_volume: f64,
}

pub fn get_simulate_log_fns() -> (impl Fn(&Vec<Candle>) -> f64 + Clone, impl Fn(&BTreeMap<String, FootprintValue>) + Clone, impl Fn(&Indicator) + Clone) {
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