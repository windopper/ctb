use ctb::upbit_api::candle::{get_candle_days, get_candle_minutes, get_candle_seconds};

#[tokio::test]
async fn test_get_candle_seconds() {
    let result = get_candle_seconds("KRW-BTC", None, 5).await;
    // eprintln!("{:?}", result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_candle_minutes() {
    let result = get_candle_minutes("KRW-BTC", None, 5, 1).await;
    // eprintln!("{:?}", result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_candle_days() {
    let result = get_candle_days("KRW-BTC", None, 5).await;
    // eprintln!("{:?}", result);
    assert!(result.is_ok());
}