use ctb::{core::candle::CandleTrait, strategy::mac::run, upbit_api::candle::{get_candle_days, get_candle_seconds}};

#[tokio::test]
async fn test_mac() {
    let candles = get_candle_seconds("KRW-BTC", None, 200).await.unwrap().into_iter().map(|c| c.get_trade_price()).collect::<Vec<f64>>();
    println!("{:?}", run(candles).await.unwrap());
}