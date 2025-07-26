use ctb::{
    backtest::{
        fetch::{fetch_n_day_candles, fetch_n_minute_candles, fetch_n_seconds_candles},
        lib::{BacktestParams, BacktesterState, PositionState},
        simulate::simulate
    }, core::{
        candle::{Candle, CandleBase, CandleTrait}, orderbook::Orderbook, ticker::Ticker, trade::{AskBid, Trade}
    }, helper::{cvd::{calculate_cvd, CvdTrade, TradeSide}, footprint::{footprint, FootprintTrade}, orderbook::{orderbook_helper, top_n_orderbook_ratio}, trade::{latest_n_ask_bid_volume_ratio, TradeState}}, strategy::{of1::{of1, Of1Params, Of1State}, swc::{self, run, StrategyParams}}, upbit_api::{
        self,
        candle::{get_candle_minutes, get_candle_seconds}
    }
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, to_string_pretty, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;
use url::Url;
use std::{collections::VecDeque, time::{Duration, Instant}};
use std::io::Write;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::Rng;
use colored::*;


const CODE: &str = "KRW-XRP";

#[tokio::main]
async fn main() {
    let url = "wss://api.upbit.com/websocket/v1".into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // 이제 trade와 orderbook을 모두 구독합니다.
    let request = json!([
        {"ticket": "unique_ticket"},
        {"type": "trade", "codes": [CODE], "is_only_realtime": true},
        {"type": "orderbook", "codes": [CODE], "is_only_realtime": true},
        {"type": "ticker", "codes": [CODE], "is_only_realtime": true},
        {"type": "candle.1m", "codes": [CODE], "is_only_realtime": true},
        {"format": "SIMPLE"}
    ]);

    write.send(Message::Text(request.to_string().into())).await.unwrap();
    println!("Upbit WebSocket 연결 및 구독 성공!");

    let mut trade_state = TradeState {
        latest_n_volume_diff: 0.0,
        latest_n_volume_total: 0.0,
        trades: Vec::new(),
    };

    let mut orderbook_ratio = 0.0;
    let mut ask_bid_volume_ratio = 0.0;

    let mut trades: Vec<CvdTrade> = Vec::new();
    let mut footprint_trades: Vec<FootprintTrade> = Vec::new();

    let mut interval_1_minute = tokio::time::interval(Duration::from_secs(60));
    let mut cvd = (0.0, 0.0);

    let mut of1_state = Of1State::new();
    let mut of1_params = Of1Params::new();
    let backtest_params = BacktestParams {
        take_profit_pct: 0.2,
        fees_pct: 0.0005,
    };
    let mut backtester = BacktesterState::new(backtest_params);

    let formatted_time = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let pre_fetch_candles = fetch_n_minute_candles(CODE, 20, &formatted_time, 1).await.unwrap();
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
        of1_state.history_candles.push_back(candle);
    }

    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                if let Ok(msg) = msg {
                    if let Ok(text) = msg.to_text() {
                        
                        let value: Value = serde_json::from_str(&text).unwrap();
                        
                        if value["ty"] == "orderbook" {
                            let orderbook: Orderbook = serde_json::from_str(&text).unwrap();
                            orderbook_ratio = top_n_orderbook_ratio(&orderbook, 2);
                        } else if value["ty"] == "trade" {
                            let trade: Trade = serde_json::from_str(&text).unwrap();
                            ask_bid_volume_ratio = latest_n_ask_bid_volume_ratio(&trade, &mut trade_state);
                            trades.push(CvdTrade {
                                volume: trade.trade_price * trade.trade_volume,
                                side: if trade.ask_bid == AskBid::Ask { TradeSide::Sell } else { TradeSide::Buy },
                            });
                            cvd = calculate_cvd(&trades);

                            footprint_trades.push(FootprintTrade {
                                ask_bid: trade.ask_bid,
                                price: trade.trade_price,
                                volume: trade.trade_volume,
                            });
                        } else if value["ty"] == "ticker" {
                            let ticker: Ticker = serde_json::from_str(&text).unwrap();
                            let current_price = ticker.trade_price;
                            let current_timestamp = ticker.trade_timestamp;
                            of1_state.current_ticker = Some(ticker);

                            backtester.check_and_close_position(current_price, &current_timestamp.to_string());
                            let signal = of1(&mut of1_state, &of1_params, &mut backtester.get_position());
                            backtester.handle_signal(&signal, current_price, &current_timestamp.to_string());
                        } else if value["ty"] == "candle.1m" {
                            let kst = value["cdttmk"].as_str().unwrap().to_string();
                            let candle_acc_trade_volume = value["catv"].as_f64().unwrap();
                            let candle = Candle {
                                base: CandleBase {
                                    market: CODE.to_string(),
                                    candle_date_time_utc: value["cdttmu"].as_str().unwrap().to_string(),
                                    candle_date_time_kst: kst.clone(),
                                    opening_price: value["op"].as_f64().unwrap(),
                                    high_price: value["hp"].as_f64().unwrap(),
                                    low_price: value["lp"].as_f64().unwrap(),
                                    trade_price: value["tp"].as_f64().unwrap(),
                                    timestamp: value["tms"].as_u64().unwrap(),
                                    candle_acc_trade_price: value["catp"].as_f64().unwrap(),
                                    candle_acc_trade_volume: value["catv"].as_f64().unwrap(),
                                }
                            };
                            

                            // 최근 캔들과 시간이 똑같으면 제외
                            let last_candle = of1_state.history_candles.back();
                            if last_candle.is_some() && last_candle.unwrap().get_candle_date_time_utc() == candle.get_candle_date_time_utc() {
                                continue;
                            }
                            of1_state.history_candles.push_back(candle);

                            // 지표
                            let recent_candle_20 = of1_state.history_candles.iter().rev().take(20).collect::<Vec<&Candle>>();
                            let recent_candle_20_avg = recent_candle_20.iter().map(|c| c.get_candle_acc_trade_volume()).sum::<f64>() / recent_candle_20.len() as f64;
                            let volume_threshold = recent_candle_20_avg * of1_params.volume_threshold_multiplier;
                            println!("\n최근 20개 거래량 평균: {:?} | 현재 캔들 거래량: {} | 거래량 임계치: {}", recent_candle_20_avg, candle_acc_trade_volume, volume_threshold);
                            // 캔들 업데이트마다 footprint 갱신
                            let footprint = footprint(&footprint_trades);
                            let mut prices = footprint.keys().collect::<Vec<&String>>();
                            println!("footprint kst: {}", kst);
                            // prices 순으로 정렬
                            prices.sort_by(|a, b| b.partial_cmp(a).unwrap());
                            
                            // 최대 길이 계산을 위한 임시 출력
                            let max_ask_len = footprint.values().map(|f| format!("{:.6}", f.ask_volume).len()).max().unwrap_or(0);
                            let max_bid_len = footprint.values().map(|f| format!("{:.6}", f.bid_volume).len()).max().unwrap_or(0);
                            
                            for price in prices {
                                let ask_vol = footprint[price].ask_volume;
                                let bid_vol = footprint[price].bid_volume;
                                let diff = bid_vol - ask_vol; // 매수 - 매도 차이
                                let diff_sign = if diff >= 0.0 { "+" } else { "" };
                                let total_vol = ask_vol + bid_vol;
                                let diff_pct = if total_vol.abs() > 1e-8 {
                                    (diff / total_vol) * 100.0
                                } else {
                                    0.0
                                };
                                let diff_str = format!("{}{:.*}", diff_sign, 6, diff);
                                let diff_pct_str = format!("({:+.2}%)", diff_pct);
                                let colored_diff = if diff >= 0.0 {
                                    diff_str.green()
                                } else {
                                    diff_str.red()
                                };
                                let colored_pct = if diff >= 0.0 {
                                    diff_pct_str.green()
                                } else {
                                    diff_pct_str.red()
                                };
                                println!("{:<12}: {:<width$} | {:<width$} | {} {}",
                                    price,
                                    format!("{:.6}", ask_vol),
                                    format!("{:.6}", bid_vol),
                                    colored_diff,
                                    colored_pct,
                                    width = max_ask_len.max(max_bid_len)
                                );
                            }

                            trades.clear();
                            footprint_trades.clear();


                        }

                        // 상위 2개 매물대 매도/매수 비율이 낮은데 거래량 차이 비율이 음수면 지정가 매수가 흡수
                        print!("\r상위 2개 매물대 비율: {:.6} | 최근 40개 매수/매도 비율: {:.6} | 누적 거래량 델타: {:.6} | 거래량 차이 비율: {:.6}"
                        , orderbook_ratio, ask_bid_volume_ratio, cvd.0, cvd.1);
                        std::io::stdout().flush().unwrap();
                    }
                }
            }

            _ = interval_1_minute.tick() => {
                let footprint = footprint(&footprint_trades);
                
            }
        }   
    }


    return;

    let backtest_params = BacktestParams {
        take_profit_pct: 0.2,
        fees_pct: 0.0005,
    };

    let mut win_count = 0;
    let mut loss_count = 0;
    let mut total_pnl_pct = 0.0;
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    let mut count = 0;
    let round = 1;
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                print!("\rcount: {}", count);
                std::io::stdout().flush().unwrap();
                if count == round {
                    break;
                }
                let mut backtester = BacktesterState::new(backtest_params);
                let now = Utc::now();
                let day_diff = now - ChronoDuration::days(365);
                let seconds_range = (now.timestamp() - day_diff.timestamp()) as u64;
                // 랜덤 오프셋 생성
                let mut rng = rand::rng();
                let random_offset = rng.random_range(0..seconds_range);
                // 랜덤 시각
                let random_time = day_diff + ChronoDuration::seconds(random_offset as i64);
                let formatted_time = random_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                let current_time = "2025-07-26T19:09:00Z";

                // 14일 / 15분 캔들
                let candle_count = 1000;

                // let candles = fetch_n_day_candles("KRW-BTC", candle_count, &formatted_time).await;
                let candles = fetch_n_minute_candles("KRW-XRP", candle_count, &current_time, 15).await;
                // let candles = fetch_n_seconds_candles("KRW-BTC", candle_count, &formatted_time).await;
                match candles {
                    Ok(mut candles) => {
                        candles.reverse();
                        let history_candles = candles.split_off(candle_count as usize - 200);
                        simulate(candles, history_candles, &mut backtester); 
                        win_count += backtester.win_count;
                        loss_count += backtester.loss_count;
                        total_pnl_pct += backtester.total_pnl_pct;
                        count += 1;
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
        }
    }
    println!("{}회 백테스트 완료", round);
    println!("win_count: {}", win_count);
    println!("loss_count: {}", loss_count);
    println!("total_pnl_pct: {}", total_pnl_pct / round as f64);


    // // 1초마다 캔들 업데이트
    // let mut interval = tokio::time::interval(Duration::from_secs(1));

    // // 10초마다 현재 수익률 출력
    // let mut profit_interval = tokio::time::interval(Duration::from_secs(10));

    // loop {
    //     tokio::select! {
    //         // WebSocket 메시지 수신 처리
    //         Some(msg) = read.next() => {
    //             if let Ok(msg) = msg {
    //                 if let Ok(text) = msg.to_text() {
    //                     let value: Value = serde_json::from_str(&text).unwrap();
                        
    //                     if value["ty"] == "trade" {
    //                         let trade: Trade = serde_json::from_str(&text).unwrap();
    //                         let current_price = trade.trade_price;
    //                         backtester.check_and_close_position(current_price);
    //                         state.recent_trades.push_back((Instant::now(), trade));

    //                         state.prune_old_trades(params.trade_delta_window);
                            
    //                         let signal = swc::run(&mut state, &params);

    //                         backtester.handle_signal(signal, current_price);
    //                     } else if value["ty"] == "orderbook" {
    //                         let orderbook: Orderbook = serde_json::from_str(&text).unwrap();
    //                         state.current_orderbook = Some(orderbook);
    //                     }
    //                 }
    //             }
    //         }
            
    //         // 1분마다 주기적으로 실행되는 로직
    //         _ = interval.tick() => {
    //             let mut candles = get_candle_seconds(CODE, None, 30).await.unwrap();
    //             // println!("초 봉 데이터 업데이트");
    //             candles.reverse(); // 최신 데이터가 배열의 끝에 와야 함
    //             state.historical_candles = candles.into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>).collect();
    //         }

    //         // 10초마다 현재 수익률 출력
    //         _ = profit_interval.tick() => {
    //             // backtester.print_results();
    //         }
    //     }
    // }
}