use ctb::{backtest::lib::{BacktestParams, BacktesterState}, core::{candle::CandleTrait, orderbook::Orderbook, trade::Trade}, strategy::swc::{self, run, MarketState, StrategyParams}, upbit_api::{self, candle::{get_candle_minutes, get_candle_seconds}}};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, to_string_pretty, Value};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;
use url::Url;
use std::time::{Duration, Instant};


const CODE: &str = "KRW-SAHARA";

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
        {"format": "SIMPLE"}
    ]);

    write.send(Message::Text(request.to_string().into())).await.unwrap();
    println!("Upbit WebSocket 연결 및 구독 성공!");

    let mut state = MarketState::new();
    let params = StrategyParams {
        trade_delta_window: Duration::from_secs(1), // 최근 1초간의 거래를 분석
        obi_depth: 5,                                // 위아래 5호가까지 분석
        wall_krw_threshold: 5_000_000.0,
        atr_period: 14,
        atr_multiplier: 2.0,
        base_delta_threshold: 0.1,
        bb_period: 20,
        bb_multiplier: 1.5,
        adx_period: 14,
        min_rsi: 35.0,
        max_rsi: 65.0,
    };

    let backtest_params = BacktestParams {
        take_profit_pct: 0.02,
        stop_loss_pct: 0.04,
        trailing_stop_pct: 0.004,
        fees_pct: 0.0005,
    };

    let mut backtester = BacktesterState::new(backtest_params);
    
    // 1초마다 캔들 업데이트
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    // 10초마다 현재 수익률 출력
    let mut profit_interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            // WebSocket 메시지 수신 처리
            Some(msg) = read.next() => {
                if let Ok(msg) = msg {
                    if let Ok(text) = msg.to_text() {
                        let value: Value = serde_json::from_str(&text).unwrap();
                        
                        if value["ty"] == "trade" {
                            let trade: Trade = serde_json::from_str(&text).unwrap();
                            let current_price = trade.trade_price;
                            backtester.check_and_close_position(current_price);
                            state.recent_trades.push_back((Instant::now(), trade));

                            state.prune_old_trades(params.trade_delta_window);
                            
                            let signal = swc::run(&mut state, &params);

                            backtester.handle_signal(signal, current_price);
                        } else if value["ty"] == "orderbook" {
                            let orderbook: Orderbook = serde_json::from_str(&text).unwrap();
                            state.current_orderbook = Some(orderbook);
                        }
                    }
                }
            }
            
            // 1분마다 주기적으로 실행되는 로직
            _ = interval.tick() => {
                let mut candles = get_candle_seconds(CODE, None, 30).await.unwrap();
                // println!("초 봉 데이터 업데이트");
                candles.reverse(); // 최신 데이터가 배열의 끝에 와야 함
                state.historical_candles = candles.into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>).collect();
            }

            // 10초마다 현재 수익률 출력
            _ = profit_interval.tick() => {
                // backtester.print_results();
            }
        }
    }
}