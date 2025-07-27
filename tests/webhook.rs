use ctb::webhook::lib::{send_webhook, send_buy_signal, send_sell_signal, send_trade_summary};

#[tokio::test]
async fn test_send_webhook() {
    let content = "test";
    send_webhook("테스트", content).await;
}

#[tokio::test]
async fn test_send_buy_signal() {
    send_buy_signal(
        "KRW-BTC",
        50000000.0,
        0.001,
        48000000.0,
        55000000.0,
        2.5,
        "SuperTrend + EMA"
    ).await;
}

#[tokio::test]
async fn test_send_sell_signal() {
    send_sell_signal(
        "KRW-BTC",
        52000000.0,
        0.001,
        2000000.0,
        4.0,
        "SuperTrend + EMA",
        "목표가 도달"
    ).await;
}

#[tokio::test]
async fn test_send_sell_signal_loss() {
    send_sell_signal(
        "KRW-BTC",
        48000000.0,
        0.001,
        -2000000.0,
        -4.0,
        "SuperTrend + EMA",
        "손절가 도달"
    ).await;
}

#[tokio::test]
async fn test_send_trade_summary() {
    send_trade_summary(
        "KRW-BTC",
        100,
        65,
        15000000.0,
        65.0,
        150000.0,
        -500000.0
    ).await;
}





