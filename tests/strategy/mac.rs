use ctb::strategy::mac::run;

#[tokio::test]
async fn test_mac() {
    run().await.unwrap();
}