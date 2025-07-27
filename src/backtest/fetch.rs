use std::time::Duration;

use crate::{core::candle::CandleTrait, upbit_api::candle::{get_candle_days, get_candle_minutes, get_candle_seconds}};

pub async fn fetch_n_seconds_candles(market: &str, mut count: u32, to: &str) 
-> Result<Vec<Box<dyn CandleTrait>>, Box<dyn std::error::Error>> {
    let mut candles = Vec::new();
    let mut to = to.to_string();
    while count >= 200 {
        let new_candles = get_candle_seconds(market, Some(&to), count).await?;
        // 0.15초 대기
        tokio::time::sleep(Duration::from_millis(120)).await;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
        to = new_candles.last().ok_or("No candles")?.get_candle_date_time_utc().to_string();
        count -= 200;
    }

    if count > 0 && count < 200 {
        let new_candles = get_candle_seconds(market, Some(&to), count).await?;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
    }
    
    println!("{}개 캔들 가져옴", candles.len());
    Ok(candles)
}

/// upbit의 경우 최대 200개의 데이터만 가져올 수 있음
/// 따라서 200개 이상의 데이터를 가져오기 위해서는 여러 번 호출해야 함
pub async fn fetch_n_minute_candles(market: &str, mut count: u32, to: &str, unit: u32) 
-> Result<Vec<Box<dyn CandleTrait>>, Box<dyn std::error::Error>> {
    let mut candles = Vec::new();
    let mut to = to.to_string();
    while count >= 200 {
        let new_candles = get_candle_minutes(market, Some(&to), count, unit).await?;
        // 0.15초 대기
        tokio::time::sleep(Duration::from_millis(120)).await;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
        to = new_candles.last().ok_or("No candles")?.get_candle_date_time_utc().to_string();
        count -= 200;
    }

    if count > 0 && count < 200 {
        let new_candles = get_candle_minutes(market, Some(&to), count, unit).await?;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
    }
    Ok(candles)
} 

pub async fn fetch_n_day_candles(market: &str, mut count: u32, to: &str) 
-> Result<Vec<Box<dyn CandleTrait>>, Box<dyn std::error::Error>> {
    let mut candles = Vec::new();
    let mut to = to.to_string();
    while count >= 200 {
        let new_candles = get_candle_days(market, Some(&to), count).await?;
        // 0.15초 대기
        tokio::time::sleep(Duration::from_millis(120)).await;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
        to = new_candles.last().ok_or("No candles")?.get_candle_date_time_utc().to_string();
        count -= 200;
    }

    if count > 0 && count < 200 {
        let new_candles = get_candle_days(market, Some(&to), count).await?;
        candles.extend(new_candles.clone().into_iter().map(|c| Box::new(c) as Box<dyn CandleTrait>));
    }

    println!("{}개 캔들 가져옴", candles.len());
    Ok(candles)
}