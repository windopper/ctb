use std::collections::HashSet;

use ctb::{backtest::{fetch::{fetch_n_day_candles, fetch_n_minute_candles}, lib::{BacktestParams, BacktesterState}}, core::signal::Signal};

#[tokio::test]
async fn test_fetch_n_minute_candles() {
    let candles = fetch_n_minute_candles("KRW-BTC", 400, "2024-01-01T00:00:00Z", 5).await.unwrap();
    let mut set = HashSet::new();
    for c in &candles {
        set.insert(c.get_candle_date_time_utc());
    }
    assert_eq!(set.len(), candles.len(), "캔들 UTC 시간이 중복됩니다");
    assert_eq!(candles[0].as_ref().get_candle_date_time_utc() > candles[1].as_ref().get_candle_date_time_utc(), true);
    assert_eq!(candles[398].as_ref().get_candle_date_time_utc() > candles[399].as_ref().get_candle_date_time_utc(), true);
    eprintln!("{:?}", candles.len());
}

#[tokio::test]
async fn test_fetch_n_day_candles() {
    let candles = fetch_n_day_candles("KRW-BTC", 400, "2024-01-01T00:00:00Z").await.unwrap();
    let mut set = HashSet::new();
    for c in &candles {
        set.insert(c.get_candle_date_time_utc());
    }
    assert_eq!(set.len(), candles.len(), "캔들 UTC 시간이 중복됩니다");
}

#[test]
fn test_buy_webhook() {
    let mut backtester = BacktesterState::new(BacktestParams::default("KRW-BTC", "TEST"));
    backtester.handle_signal(&Signal::Buy {
        reason: "TEST".to_string(),
        initial_trailing_stop: 0.0,
        take_profit: 0.0,
        asset_pct: 1.0,
    }, 0.0, "2024-01-01T00:00:00Z");
}