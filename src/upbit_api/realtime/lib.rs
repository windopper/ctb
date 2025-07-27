use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::{sync::mpsc, time::interval};
use tokio_tungstenite::connect_async;
use tungstenite::{client::IntoClientRequest, Message};

use crate::core::{candle::{Candle, CandleBase}, orderbook::Orderbook, ticker::Ticker, trade::Trade};

pub async fn listen_realtime_data(
    code: &str,
    shutdown_recv: &mut mpsc::Receiver<()>,
    orderbook_fn: &mut dyn FnMut(&Orderbook),
    trade_fn: &mut dyn FnMut(&Trade),
    ticker_fn: &mut dyn FnMut(&Ticker),
    candle_fn: &mut dyn FnMut(&Candle),
    exit_fn: &mut dyn FnMut(),
) {
    let url = "wss://api.upbit.com/websocket/v1".into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let request = json!([
        {"ticket": "unique_ticket"},
        {"type": "trade", "codes": [code], "is_only_realtime": true},
        {"type": "orderbook", "codes": [code], "is_only_realtime": true},
        {"type": "ticker", "codes": [code], "is_only_realtime": true},
        {"type": "candle.1m", "codes": [code], "is_only_realtime": true},
        {"format": "SIMPLE"}
    ]);

    write.send(Message::Text(request.to_string().into())).await.unwrap();
    
    loop {
        tokio::select! {
            Some(msg) = read.next() => {
                if let Ok(msg) = msg {
                    if let Ok(text) = msg.to_text() {
                        let value: Value = serde_json::from_str(&text).unwrap();
                        if value["ty"] == "orderbook" {
                            let orderbook: Orderbook = serde_json::from_str(&text).unwrap();
                            orderbook_fn(&orderbook);
                        } else if value["ty"] == "trade" {
                            let trade: Trade = serde_json::from_str(&text).unwrap();
                            trade_fn(&trade);
                        } else if value["ty"] == "ticker" {
                            let ticker: Ticker = serde_json::from_str(&text).unwrap();
                            ticker_fn(&ticker);
                        } else if value["ty"] == "candle.1m" {
                            let candle = Candle {
                                base: CandleBase {
                                    market: code.to_string(),
                                    candle_date_time_utc: value["cdttmu"].as_str().unwrap().to_string(),
                                    candle_date_time_kst: value["cdttmk"].as_str().unwrap().to_string(),
                                    opening_price: value["op"].as_f64().unwrap(),
                                    high_price: value["hp"].as_f64().unwrap(),
                                    low_price: value["lp"].as_f64().unwrap(),
                                    trade_price: value["tp"].as_f64().unwrap(),
                                    timestamp: value["tms"].as_u64().unwrap(),
                                    candle_acc_trade_price: value["catp"].as_f64().unwrap(),
                                    candle_acc_trade_volume: value["catv"].as_f64().unwrap(),
                                }
                            };

                            candle_fn(&candle);
                        }
                    }
                }
            }

            _ = shutdown_recv.recv() => {
                println!("종료 신호를 수신하여 메인 루프를 중단합니다.");
                break; // select! 와 loop 를 모두 빠져나감
            }
        }
    }

    exit_fn();
    
    // WebSocket 연결 종료
    if let Err(e) = write.close().await {
        eprintln!("WebSocket 연결 종료 중 오류 발생: {}", e);
    }
    println!("WebSocket 연결이 정상적으로 종료되었습니다.");
}