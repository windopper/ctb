use serde::{Deserialize, Serialize};

pub trait TickerTrait {
}

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
pub enum MarketState {
    #[serde(rename = "PREVIEW")] Preview,
    #[serde(rename = "ACTIVE")] Active,
    #[serde(rename = "DELISTED")] Delisted,
    #[serde(other)] Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MarketWarning {
    #[serde(rename = "NONE")] None,
    #[serde(rename = "CAUTION")] Caution,
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
pub struct Ticker {
    #[serde(rename = "ty")]
    pub ticker: String,
    #[serde(rename = "cd")]
    pub code: String,
    #[serde(rename = "op")]
    pub opening_price: f64,
    #[serde(rename = "hp")]
    pub high_price: f64,
    #[serde(rename = "lp")]
    pub low_price: f64,
    #[serde(rename = "tp")]
    pub trade_price: f64,
    #[serde(rename = "pcp")]
    pub prev_closing_price: f64,
    #[serde(rename = "c")]
    pub change: Change,
    #[serde(rename = "cp")]
    pub change_price: f64,
    #[serde(rename = "scp")]
    pub signed_change_price: f64,
    #[serde(rename = "cr")]
    pub change_rate: f64,
    #[serde(rename = "scr")]
    pub signed_change_rate: f64,
    #[serde(rename = "tv")]
    pub trade_volume: f64,
    #[serde(rename = "atv")]
    pub acc_trade_volume: f64,
    #[serde(rename = "atv24h")]
    pub acc_trade_volume_24h: f64,
    #[serde(rename = "atp")]
    pub acc_trade_price: f64,
    #[serde(rename = "atp24h")]
    pub acc_trade_price_24h: f64,
    #[serde(rename = "tdt")]
    pub trade_date: String,
    #[serde(rename = "ttm")]
    pub trade_time: String,
    #[serde(rename = "ttms")]
    pub trade_timestamp: i64,
    #[serde(rename = "ab")]
    pub ask_bid: AskBid,
    #[serde(rename = "aav")]
    pub acc_ask_volume: f64,
    #[serde(rename = "abv")]
    pub acc_bid_volume: f64,
    #[serde(rename = "h52wp")]
    pub highest_52_week_price: f64,
    #[serde(rename = "h52wdt")]
    pub highest_52_week_date: String,
    #[serde(rename = "l52wp")]
    pub lowest_52_week_price: f64,
    #[serde(rename = "l52wdt")]
    pub lowest_52_week_date: String,
    #[serde(rename = "ts")]
    pub trade_status: Option<String>, // Deprecated
    #[serde(rename = "ms")]
    pub market_state: MarketState,
    #[serde(rename = "msfi")]
    pub market_state_for_ios: Option<String>, // Deprecated
    #[serde(rename = "its")]
    pub is_trading_suspended: Option<bool>, // Deprecated
    #[serde(rename = "dd")]
    pub delisting_date: Option<String>,
    #[serde(rename = "mw")]
    pub market_warning: MarketWarning,
    #[serde(rename = "tms")]
    pub timestamp: i64,
    #[serde(rename = "st")]
    pub stream_type: StreamType,
}