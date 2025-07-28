use ctb::{
    backtest::{
        fetch::fetch_n_minute_candles, lib::{BacktestParams, BacktesterState}, simulate::{self, simulate_with_realtime_data, SimulationConfig}
    }, core::candle::{Candle, CandleBase}, webhook::lib::send_webhook
};
use tokio::sync::mpsc;


const CODES: [&str; 5] = ["KRW-XRP", "KRW-BLAST", "KRW-BTC", "KRW-ETH", "KRW-GLM"];

#[tokio::main]
async fn main() {
    // realtime_simulation().await;
    snapshop_simulation().await;
}

async fn snapshop_simulation() {
    let random_date = "2025-07-28 06:20:00";
    let candles = fetch_n_minute_candles("KRW-XRP", 400, random_date, 3).await.unwrap();
    let candles = candles.into_iter().map(|c| {
        // Box<dyn CandleTrait>에서 Candle로 변환
        // 실제로는 MinuteCandle이므로 Candle로 변환
        let base = CandleBase {
            market: c.get_market().to_string(),
            candle_date_time_utc: c.get_candle_date_time_utc().to_string(),
            candle_date_time_kst: c.get_candle_date_time_kst().to_string(),
            opening_price: c.get_opening_price(),
            high_price: c.get_high_price(),
            low_price: c.get_low_price(),
            trade_price: c.get_trade_price(),
            timestamp: c.get_timestamp(),
            candle_acc_trade_price: c.get_candle_acc_trade_price(),
            candle_acc_trade_volume: c.get_candle_acc_trade_volume(),
        };
        Candle { base }
    }).rev().collect();
    let mut backtester_params = BacktestParams::default("KRW-XRP", "candle_pattern");
    backtester_params.enable_webhook_log = false;
    let mut backtester = BacktesterState::new(backtester_params);
    simulate::simulate(candles, &mut backtester);
}

async fn realtime_simulation() {
    // shutdown 신호 수신 채널
    let (shutdown_send, mut shutdown_recv) = mpsc::channel(1);

    // Ctrl+C 핸들러 설정
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
        println!("\nCtrl+C 신호 수신. 종료를 시작합니다.");
        let _ = send_webhook("shutdown...", "shutdown signal received").await;
        shutdown_send.send(()).await.expect("failed to send shutdown signal");
    });

    let mut config = SimulationConfig::new();
    config.enable_log = false;
    send_webhook("booting...", &format!("realtime backtest start - {}", CODES.join(", "))).await;
    let results = simulate_with_realtime_data(&CODES, &mut shutdown_recv, &config).await;

    println!("모든 백테스트 완료. 결과: {:?}", results.len());
    println!("프로그램을 종료합니다.");
    return;
}