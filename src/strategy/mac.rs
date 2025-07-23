use std::error::Error;

use crate::{core::signal::Signal};

fn calculate_moving_average(closing_prices: &[f64], window: usize) -> Option<f64> {
    if closing_prices.len() < window {
        return None;
    }
    let sum: f64 = closing_prices.iter().take(window).sum();
    Some(sum / window as f64)
}

pub async fn run(closing_prices: Vec<f64>) -> Result<Signal, Box<dyn Error>> {
    let short_ma = calculate_moving_average(&closing_prices, 5);
    let long_ma = calculate_moving_average(&closing_prices, 20);

    let prev_short_ma = calculate_moving_average(&closing_prices[1..], 5);
    let prev_long_ma = calculate_moving_average(&closing_prices[1..], 20);

    println!("----- latest data -----");
    println!("----- 5 ma: {:?} -----", short_ma.unwrap());
    println!("----- 20 ma: {:?} -----", long_ma.unwrap());
    println!("----- 5 ma change: {:?} -----", short_ma.unwrap() - prev_short_ma.unwrap());
    println!("----- 20 ma change: {:?} -----", long_ma.unwrap() - prev_long_ma.unwrap());
    println!("--------------------------------");

    if let (Some(short_ma), Some(long_ma), Some(prev_short_ma), Some(prev_long_ma)) = (short_ma, long_ma, prev_short_ma, prev_long_ma) {
        if short_ma > long_ma && prev_short_ma < prev_long_ma {
            return Ok(Signal::Buy);
        } else if short_ma < long_ma && prev_short_ma > prev_long_ma {
            return Ok(Signal::Sell);
        } else {
            return Ok(Signal::Hold);
        }
    } else {
        println!("data is not enough");
    }

    Ok(Signal::Hold)
}