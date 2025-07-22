use std::error::Error;

use crate::{core::signal::Signal, upbit_api::candle::get_candle_days};

fn calculate_moving_average(candles: &[f64], window: usize) -> Option<f64> {
    if candles.len() < window {
        return None;
    }
    let sum: f64 = candles.iter().take(window).sum();
    Some(sum / window as f64)
}

pub async fn run() -> Result<Signal, Box<dyn Error>> {
    let candles = get_candle_days("KRW-BTC", None, 200).await?;
    let closing_prices = candles.iter().map(|c| c.trade_price).collect::<Vec<f64>>();

    let short_ma = calculate_moving_average(&closing_prices, 5);
    let long_ma = calculate_moving_average(&closing_prices, 20);

    let prev_short_ma = calculate_moving_average(&closing_prices[1..], 5);
    let prev_long_ma = calculate_moving_average(&closing_prices[1..], 20);

    println!("----- 최신 데이터 -----");
    println!("----- 5일 이동평균: {:?} -----", short_ma.unwrap());
    println!("----- 20일 이동평균: {:?} -----", long_ma.unwrap());
    println!("----- 5일 이동평균 변동: {:?} -----", short_ma.unwrap() - prev_short_ma.unwrap());
    println!("----- 20일 이동평균 변동: {:?} -----", long_ma.unwrap() - prev_long_ma.unwrap());
    println!("--------------------------------");
    println!("날짜: {}", candles[0].candle_date_time_kst);
    println!("종가: {}", candles[0].trade_price);

    if let (Some(short_ma), Some(long_ma), Some(prev_short_ma), Some(prev_long_ma)) = (short_ma, long_ma, prev_short_ma, prev_long_ma) {
        if short_ma > long_ma && prev_short_ma < prev_long_ma {
            return Ok(Signal::Buy);
        } else if short_ma < long_ma && prev_short_ma > prev_long_ma {
            return Ok(Signal::Sell);
        } else {
            return Ok(Signal::Hold);
        }
    } else {
        println!("데이터 부족");
    }

    Ok(Signal::Hold)
}