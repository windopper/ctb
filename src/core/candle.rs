use serde::{Deserialize, Serialize, Deserializer};

/// 공통 캔들 정보 구조체
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CandleBase {
    /// 마켓 코드
    #[serde(deserialize_with = "null_to_empty_string")]
    pub market: String,
    /// 캔들 시간 (UTC)
    #[serde(deserialize_with = "null_to_empty_string")]
    pub candle_date_time_utc: String,
    /// 캔들 시간 (KST)
    #[serde(deserialize_with = "null_to_empty_string")]
    pub candle_date_time_kst: String,
    /// 시가
    pub opening_price: f64,
    /// 고가
    pub high_price: f64,
    /// 저가
    pub low_price: f64,
    /// 종가
    pub trade_price: f64,
    /// 타임스탬프
    pub timestamp: u64,
    /// 누적 거래 대금
    pub candle_acc_trade_price: f64,
    /// 누적 거래량
    pub candle_acc_trade_volume: f64,
}

/// 일반 캔들
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Candle {
    #[serde(flatten)]
    pub base: CandleBase,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MinuteCandle {
    #[serde(flatten)]
    pub base: CandleBase,
    /// 분 단위
    pub unit: u32,
}

/// 일봉 캔들
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DayCandle {
    #[serde(flatten)]
    pub base: CandleBase,
    /// 전일 종가 (UTC 0시 기준)
    #[serde(deserialize_with = "null_to_zero_f64")]
    pub prev_closing_price: f64,
    /// 변동가
    #[serde(deserialize_with = "null_to_zero_f64")]
    pub change_price: f64,
    /// 변동률
    #[serde(deserialize_with = "null_to_zero_f64")]
    pub change_rate: f64,   
}

fn null_to_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>

where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?.unwrap_or("".to_string()))
}

fn null_to_zero_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<f64>::deserialize(deserializer)?.unwrap_or(0.0))
}

pub trait CandleTrait {
    fn get_market(&self) -> &str;
    fn get_candle_date_time_utc(&self) -> &str;
    fn get_candle_date_time_kst(&self) -> &str;
    fn get_opening_price(&self) -> f64;
    fn get_high_price(&self) -> f64;
    fn get_low_price(&self) -> f64;
    fn get_trade_price(&self) -> f64;
    fn get_timestamp(&self) -> u64;
    fn get_candle_acc_trade_price(&self) -> f64;
    fn get_candle_acc_trade_volume(&self) -> f64;
}

impl CandleTrait for CandleBase {
    fn get_market(&self) -> &str {
        &self.market
    }
    fn get_candle_date_time_utc(&self) -> &str {
        &self.candle_date_time_utc
    }
    fn get_candle_date_time_kst(&self) -> &str {
        &self.candle_date_time_kst
    }
    fn get_opening_price(&self) -> f64 {
        self.opening_price
    }
    fn get_high_price(&self) -> f64 {
        self.high_price
    }
    fn get_low_price(&self) -> f64 {
        self.low_price
    }
    fn get_trade_price(&self) -> f64 {
        self.trade_price
    }
    fn get_timestamp(&self) -> u64 {
        self.timestamp
    }
    fn get_candle_acc_trade_price(&self) -> f64 {
        self.candle_acc_trade_price
    }
    fn get_candle_acc_trade_volume(&self) -> f64 {
        self.candle_acc_trade_volume
    }
}

impl CandleTrait for Candle {
    fn get_market(&self) -> &str {
        self.base.get_market()
    }
    fn get_candle_date_time_utc(&self) -> &str {
        self.base.get_candle_date_time_utc()
    }
    fn get_candle_date_time_kst(&self) -> &str {
        self.base.get_candle_date_time_kst()
    }
    fn get_opening_price(&self) -> f64 {
        self.base.get_opening_price()
    }
    fn get_high_price(&self) -> f64 {
        self.base.get_high_price()
    }
    fn get_low_price(&self) -> f64 {
        self.base.get_low_price()
    }
    fn get_trade_price(&self) -> f64 {
        self.base.get_trade_price()
    }
    fn get_timestamp(&self) -> u64 {
        self.base.get_timestamp()
    }
    fn get_candle_acc_trade_price(&self) -> f64 {
        self.base.get_candle_acc_trade_price()
    }
    fn get_candle_acc_trade_volume(&self) -> f64 {
        self.base.get_candle_acc_trade_volume()
    }
}

impl CandleTrait for MinuteCandle {
    fn get_market(&self) -> &str {
        self.base.get_market()
    }
    fn get_candle_date_time_utc(&self) -> &str {
        self.base.get_candle_date_time_utc()
    }
    fn get_candle_date_time_kst(&self) -> &str {
        self.base.get_candle_date_time_kst()
    }
    fn get_opening_price(&self) -> f64 {
        self.base.get_opening_price()
    }
    fn get_high_price(&self) -> f64 {
        self.base.get_high_price()
    }
    fn get_low_price(&self) -> f64 {
        self.base.get_low_price()
    }
    fn get_trade_price(&self) -> f64 {
        self.base.get_trade_price()
    }
    fn get_timestamp(&self) -> u64 {
        self.base.get_timestamp()
    }
    fn get_candle_acc_trade_price(&self) -> f64 {
        self.base.get_candle_acc_trade_price()
    }
    fn get_candle_acc_trade_volume(&self) -> f64 {
        self.base.get_candle_acc_trade_volume()
    }
}

impl CandleTrait for DayCandle {
    fn get_market(&self) -> &str {
        self.base.get_market()
    }
    fn get_candle_date_time_utc(&self) -> &str {
        self.base.get_candle_date_time_utc()
    }
    fn get_candle_date_time_kst(&self) -> &str {
        self.base.get_candle_date_time_kst()
    }
    fn get_opening_price(&self) -> f64 {
        self.base.get_opening_price()
    }
    fn get_high_price(&self) -> f64 {
        self.base.get_high_price()
    }
    fn get_low_price(&self) -> f64 {
        self.base.get_low_price()
    }
    fn get_trade_price(&self) -> f64 {
        self.base.get_trade_price()
    }
    fn get_timestamp(&self) -> u64 {
        self.base.get_timestamp()
    }
    fn get_candle_acc_trade_price(&self) -> f64 {
        self.base.get_candle_acc_trade_price()
    }
    fn get_candle_acc_trade_volume(&self) -> f64 {
        self.base.get_candle_acc_trade_volume()
    }
}