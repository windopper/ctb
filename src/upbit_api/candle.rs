use crate::{core::candle::{Candle, DayCandle, MinuteCandle}, upbit_api::utils::request_upbit_api};

pub async fn get_candle_seconds(market: &str, to: Option<&str>, count: u32) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
    let query = if let Some(to) = to 
    { format!("?market={}&to={}&count={}", market, to, count) } else { format!("?market={}&count={}", market, count) };
    
    let body = request_upbit_api("/candles/seconds", Some(query)).await;

    if let Some(body) = body {
        let candles: Vec<Candle> = serde_json::from_str(&body)?;
        Ok(candles)
    } else {
        Err("Failed to get candle info".into())
    }
}

pub async fn get_candle_minutes(market: &str, to: Option<&str>, count: u32, unit: u32) -> Result<Vec<MinuteCandle>, Box<dyn std::error::Error>> {
    let query = if let Some(to) = to 
    { format!("?market={}&to={}&count={}&unit={}", market, to, count, unit) } else { format!("?market={}&count={}&unit={}", market, count, unit) };
    
    let body = request_upbit_api(format!("/candles/minutes/{}", unit).as_str(), Some(query)).await;

    if let Some(body) = body {
        let candles: Vec<MinuteCandle> = serde_json::from_str(&body)?;
        Ok(candles)
    } else {
        Err("Failed to get candle info".into())
    }
}

pub async fn get_candle_days(market: &str, to: Option<&str>, count: u32) -> Result<Vec<DayCandle>, Box<dyn std::error::Error>> {
    let query = if let Some(to) = to 
    { format!("?market={}&to={}&count={}", market, to, count) } else { format!("?market={}&count={}", market, count) };
    
    let body = request_upbit_api("/candles/days", Some(query)).await;

    if let Some(body) = body {
        let candles: Vec<DayCandle> = serde_json::from_str(&body)?;
        Ok(candles)
    } else {
        Err("Failed to get candle info".into())
    }
}