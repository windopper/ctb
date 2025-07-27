use ctb::{
    backtest::{
        simulate::{simulate_with_realtime_data, SimulationConfig}
    }, webhook::lib::send_webhook
};
use tokio::sync::mpsc;


const CODES: [&str; 1] = ["KRW-XRP"];

#[tokio::main]
async fn main() {
    // shutdown 신호 수신 채널
    let (shutdown_send, mut shutdown_recv) = mpsc::channel(1);

    // Ctrl+C 핸들러 설정
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("failed to install CTRL+C handler");
        println!("\nCtrl+C 신호 수신. 종료를 시작합니다.");
        let _ = send_webhook("shutdown...", "shutdown signal received").await;
        shutdown_send.send(()).await.expect("failed to send shutdown signal");
    });

    let config = SimulationConfig::new();
    send_webhook("booting...", &format!("realtime backtest start - {}", CODES.join(", "))).await;
    let results = simulate_with_realtime_data(&CODES, &mut shutdown_recv, &config).await;

    println!("모든 백테스트 완료. 결과: {:?}", results.len());
    println!("프로그램을 종료합니다.");
    return;
}