use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Candle {
    // 마켓 코드
    pub market: String,
    // 캔들 시간 (UTC)
    pub candle_date_time_utc: String,
    // 캔들 시간 (KST)
    pub candle_date_time_kst: String,
    // 시가
    pub opening_price: f64,
    // 고가
    pub high_price: f64,
    // 저가
    pub low_price: f64,
    // 종가
    pub trade_price: f64,
    // 타임스탬프
    pub timestamp: u64,
    // 누적 거래 대금
    pub candle_acc_trade_price: f64,
    // 누적 거래량
    pub candle_acc_trade_volume: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DayCandle {
    // 마켓 코드
    pub market: String,
    // 캔들 시간 (UTC)
    pub candle_date_time_utc: String,
    // 캔들 시간 (KST)
    pub candle_date_time_kst: String,
    // 시가
    pub opening_price: f64,
    // 고가
    pub high_price: f64,
    // 저가
    pub low_price: f64,
    // 종가
    pub trade_price: f64,
    // 타임스탬프
    pub timestamp: u64,
    // 누적 거래 대금
    pub candle_acc_trade_price: f64,
    // 누적 거래량
    pub candle_acc_trade_volume: f64,

    // 전일 종가 (UTC 0시 기준)
    pub prev_closing_price: f64,
    // 변동가
    pub change_price: f64,
    // 변동률
    pub change_rate: f64
}