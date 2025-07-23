use ctb::{core::candle::CandleTrait, strategy::rsi, upbit_api::candle::get_candle_seconds};

#[tokio::test]
async fn test_rsi() {
    let mut candles = get_candle_seconds("KRW-BTC", None, 100).await.unwrap().into_iter().map(|c| c.get_trade_price()).collect::<Vec<f64>>();
    candles.reverse();
    rsi::run(&candles).await;
}