use serde::{Deserialize, Serialize};

pub trait TradeTrait {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Change {
    #[serde(rename = "RISE")] Rise,
    #[serde(rename = "EVEN")] Even,
    #[serde(rename = "FALL")] Fall,
    #[serde(other)] Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AskBid {
    #[serde(rename = "ASK")] Ask,
    #[serde(rename = "BID")] Bid,
    #[serde(other)] Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum StreamType {
    #[serde(rename = "SNAPSHOT")] Snapshot,
    #[serde(rename = "REALTIME")] Realtime,
    #[serde(other)] Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    #[serde(rename = "ty")]
    pub trade_type: String, // trade : 체결
    #[serde(rename = "cd")]
    pub code: String, // 마켓 코드
    #[serde(rename = "tp")]
    pub trade_price: f64, // 체결 가격
    #[serde(rename = "tv")]
    pub trade_volume: f64, // 체결량
    #[serde(rename = "ab")]
    pub ask_bid: AskBid, // 매수/매도 구분
    #[serde(rename = "pcp")]
    pub prev_closing_price: f64, // 전일 종가
    #[serde(rename = "c")]
    pub change: Change, // 전일 대비
    #[serde(rename = "cp")]
    pub change_price: f64, // 부호 없는 전일 대비 값
    #[serde(rename = "td")]
    pub trade_date: String, // 체결 일자(UTC)
    #[serde(rename = "ttm")]
    pub trade_time: String, // 체결 시각(UTC)
    #[serde(rename = "ttms")]
    pub trade_timestamp: i64, // 체결 타임스탬프
    #[serde(rename = "tms")]
    pub timestamp: i64, // 타임스탬프
    #[serde(rename = "sid")]
    pub sequential_id: i64, // 체결 번호
    #[serde(rename = "bap")]
    pub best_ask_price: f64, // 최우선 매도 호가
    #[serde(rename = "bas")]
    pub best_ask_size: f64, // 최우선 매도 잔량
    #[serde(rename = "bbp")]
    pub best_bid_price: f64, // 최우선 매수 호가
    #[serde(rename = "bbs")]
    pub best_bid_size: f64, // 최우선 매수 잔량
    #[serde(rename = "st")]
    pub stream_type: StreamType, // 스트림 타입
}
