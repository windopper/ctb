use ctb::upbit_api::market::get_market_info;

#[tokio::test]
pub async fn test_get_market_info() {
    let result = get_market_info().await;
    eprintln!("{:?}", result);
}