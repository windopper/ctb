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

// 이전 시간대(분 단위)의 거래 내역만 필터하는 함수
// current_candle_kst: KST 기준 현재 캔들 시간
// trade_date와 trade_time은 UTC 기준
pub fn filter_trades_by_same_minute(trades: &Vec<Trade>, current_candle_kst: &str) -> Vec<Trade> {
    // KST를 UTC로 변환 (KST = UTC + 9시간)
    let current_time_kst = chrono::NaiveDateTime::parse_from_str(current_candle_kst, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());
    
    // KST에서 9시간을 빼서 UTC로 변환
    let current_time_utc = current_time_kst - chrono::Duration::hours(9);
    let current_minute_str = current_time_utc.format("%Y-%m-%dT%H:%M").to_string();
    
    // 이전 분의 거래 내역만 필터 (UTC 기준으로 비교)
    trades.iter()
        .filter(|trade| {
            // trade_date와 trade_time은 UTC 기준
            let trade_date = &trade.trade_date; // "YYYY-MM-DD" 형식 (UTC)
            let trade_time = &trade.trade_time; // "HH:MM:SS" 형식 (UTC)
            // "HH:MM"로 변환
            let trade_datetime_str = format!("{}T{}", trade_date, trade_time.split(":").take(2).collect::<Vec<&str>>().join(":"));
            trade_datetime_str == current_minute_str
        })
        .map(|trade| trade.clone())
        .collect()
}

pub fn is_trade_time_previous_minute(trade: &Trade, current_candle_kst: &str) -> bool {
    let current_time_kst = chrono::NaiveDateTime::parse_from_str(current_candle_kst, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc());
    
    let current_time_utc = current_time_kst - chrono::Duration::hours(9);
    let current_minute_str = current_time_utc.format("%Y-%m-%dT%H:%M").to_string();
    
    let trade_date = &trade.trade_date; // "YYYY-MM-DD" 형식 (UTC)
    let trade_time = &trade.trade_time; // "HH:MM:SS" 형식 (UTC)
    // "HH:MM"로 변환
    let trade_datetime_str = format!("{}T{}", trade_date, trade_time.split(":").take(2).collect::<Vec<&str>>().join(":"));
    trade_datetime_str < current_minute_str
}