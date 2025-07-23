use crate::{core::{signal::Signal, ticker::Ticker}, strategy::rsi};
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

impl VirtualTrader {
    pub fn new(initial_cash: f64) -> Self {
        Self {
            cash: initial_cash,
            coin: 0.0,
            last_buy_price: None,
            trade_log: Vec::new(),
            initial_cash,
        }
    }
    pub fn on_signal(&mut self, price: f64, signal: Signal, time: &str) {
        match signal {
            Signal::Buy => {
                if self.cash > 0.0 {
                    let buy_amount = self.cash / price;
                    self.coin += buy_amount;
                    self.last_buy_price = Some(price);
                    self.trade_log.push(format!("{}: BUY {} at {}", time, buy_amount, price));
                    self.cash = 0.0;
                }
            },
            Signal::Sell => {
                if self.coin > 0.0 {
                    let sell_amount = self.coin * price;
                    self.cash += sell_amount;
                    self.trade_log.push(format!("{}: SELL {} at {}", time, self.coin, price));
                    self.coin = 0.0;
                    self.last_buy_price = None;
                }
            },
            Signal::Hold => {}
        }
    }
    pub fn total_asset(&self, current_price: f64) -> f64 {
        self.cash + self.coin * current_price
    }
    pub fn profit_rate(&self, current_price: f64) -> f64 {
        (self.total_asset(current_price) - self.initial_cash) / self.initial_cash * 100.0
    }
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
    let mut trader = VirtualTrader::new(1_000_000.0); // 초기 자본 100만원
    while let Some(msg) = read.next().await {
        let ticker: Ticker = serde_json::from_str(&msg.unwrap().to_text().unwrap()).unwrap();
        current_prices.push(ticker.trade_price);

        // 항상 14개 유지
        if current_prices.len() > 14 {
            current_prices.remove(0);
        }
        // rsi 계산
        let (rsi, signal) = rsi::run(&current_prices).await;
        trader.on_signal(ticker.trade_price, signal, &ticker.trade_time);
        println!("time: {}, current price: {}, rsi: {}, signal: {:?}, prices stacked: {:?}, total asset: {:.2}, profit: {:.2}%"
            , ticker.trade_time, ticker.trade_price, rsi, signal, current_prices.len(), trader.total_asset(ticker.trade_price), trader.profit_rate(ticker.trade_price));
    }
}