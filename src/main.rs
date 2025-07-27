use ctb::{
    backtest::{
        fetch::fetch_n_minute_candles,
        lib::{BacktestParams, BacktesterState}, simulate::{simulate_with_realtime_data, SimulationConfig}
    }, core::{
        candle::{Candle, CandleBase, CandleTrait}, orderbook::Orderbook, ticker::Ticker, trade::{AskBid, Trade}
    }, helper::{cvd::{calculate_cvd, CvdTrade, TradeSide}, footprint::{footprint, FootprintTrade}, orderbook::top_n_orderbook_ratio, trade::{latest_n_ask_bid_volume_ratio, TradeState}}, strategy::of1::{of1, Of1Params, Of1State}, webhook::lib::send_webhook
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;
use std::time::Duration;
use std::io::Write;
use chrono::Utc;
use rand::Rng;
use colored::*;


const CODE: &str = "KRW-BLAST";

#[tokio::main]
async fn main() {
    // shutdown 신호 수신 채널
    let (shutdown_send, mut shutdown_recv) = mpsc::channel(1);

    // Ctrl+C 핸들러 설정
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
        println!("\nCtrl+C 신호 수신. 종료를 시작합니다.");
        let _ = send_webhook("CTB Trading Bot", "프로그램을 종료합니다.").await;
        shutdown_send.send(()).await.expect("failed to send shutdown signal");
    });

    let config = SimulationConfig::new();
    simulate_with_realtime_data(CODE, &mut shutdown_recv, &config).await;
    println!("프로그램을 종료합니다.");
    return;
}