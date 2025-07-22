use ctb::upbit_api::account::check_my_account;

#[tokio::test]
async fn test_check_my_account() {
    let result = check_my_account().await;
    eprintln!("{:?}", result);
}