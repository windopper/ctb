use crate::{core::{signal::Signal, ticker::Ticker}};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;

#[derive(Debug)]
pub enum SignalType {
    Buy,
    Sell,
    None,
}

pub struct VirtualTrader {
    pub cash: f64,
    pub coin: f64,
    pub last_buy_price: Option<f64>,
    pub trade_log: Vec<String>,
    pub initial_cash: f64,
}

pub async fn get_ticker(code: &str) {
    let url = "wss://api.upbit.com/websocket/v1".into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let request = json!([
        {"ticket":"test"},
        {"type":"ticker","codes":[code],"is_only_realtime": true},
        {"format":"SIMPLE"}
    ]);

    write.send(Message::Text(request.to_string().into())).await.unwrap();
    let mut current_prices = Vec::new();
    while let Some(msg) = read.next().await {
        let ticker: Ticker = serde_json::from_str(&msg.unwrap().to_text().unwrap()).unwrap();
        current_prices.push(ticker.trade_price);

        // 항상 14개 유지
        if current_prices.len() > 14 {
            current_prices.remove(0);
        }
    }
}