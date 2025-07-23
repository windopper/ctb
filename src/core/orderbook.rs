use serde::{Deserialize, Serialize};

pub trait OrderbookTrait {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderbookUnit {
    #[serde(rename = "ap")]
    pub ask_price: f64, // 매도 호가
    #[serde(rename = "bp")]
    pub bid_price: f64, // 매수 호가
    #[serde(rename = "as")]
    pub ask_size: f64, // 매도 잔량
    #[serde(rename = "bs")]
    pub bid_size: f64, // 매수 잔량
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Orderbook {
    #[serde(rename = "ty")]
    pub orderbook_type: String, // orderbook : 호가
    #[serde(rename = "cd")]
    pub code: String, // 마켓 코드
    #[serde(rename = "tas")]
    pub total_ask_size: f64, // 호가 매도 총 잔량
    #[serde(rename = "tbs")]
    pub total_bid_size: f64, // 호가 매수 총 잔량
    #[serde(rename = "obu")]
    pub orderbook_units: Vec<OrderbookUnit>, // 호가 리스트
    #[serde(rename = "tms")]
    pub timestamp: i64, // 타임스탬프
    #[serde(rename = "lv")]
    pub level: i32, // 호가 모아보기 단위
}
