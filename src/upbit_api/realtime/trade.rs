use crate::core::{orderbook::Orderbook, trade::Trade};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::client::IntoClientRequest;

pub async fn get_trade(code: &str) {
    let url = "wss://api.upbit.com/websocket/v1".into_client_request().unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let request = json!([
        {"ticket":"test"},
        {"type":"trade","codes":[code],"is_only_realtime": true},
        {"format":"SIMPLE"}
    ]);

    write.send(Message::Text(request.to_string().into())).await.unwrap();
    while let Some(msg) = read.next().await {
        let trade: Trade = serde_json::from_str(&msg.unwrap().to_text().unwrap()).unwrap();
        println!("trade: {:?}", trade);
    }
}