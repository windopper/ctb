use crate::{core::{candle::CandleTrait, signal::Signal}, helper::rsi::calculate_rsi};

pub fn generate_signals(rsi_values: &[f64], buy_threshold: f64, sell_threshold: f64) -> Vec<Signal> {
    rsi_values.iter().map(|&rsi| {
        if rsi > 0.0 && rsi < buy_threshold {
            Signal::Buy
        } else if rsi > sell_threshold {
            Signal::Sell
        } else {
            Signal::Hold
        }
    }).collect()
}

pub async fn run(closing_prices: &Vec<f64>) -> (f64, Signal) {
    const RSI_PERIOD: usize = 14;
    const BUY_THRESHOLD: f64 = 30.0;
    const SELL_THRESHOLD: f64 = 70.0;

    let rsi_results = calculate_rsi(&closing_prices, RSI_PERIOD);

    let signals = generate_signals(&rsi_results, BUY_THRESHOLD, SELL_THRESHOLD);

    // println!("{:<5} | {:<5} | {:<5} | {:<10}", "인덱스", "종가", "RSI", "신호");
    // println!("--------------------------------");

    // for i in 0..closing_prices.len() {
    //     let rsi_val = rsi_results.get(i).cloned().unwrap_or(0.0);
    //     let signal = signals.get(i).unwrap_or(&Signal::Hold);

    //     if i >= RSI_PERIOD {
    //         println!(
    //            "{:<5} | {:<10.2} | {:<15.2} | {:?}",
    //            i + 1,
    //            closing_prices[i],
    //            rsi_val,
    //            signal
    //         );
    //     } else {
    //        println!(
    //            "{:<5} | {:<10.2} | {:<15} | {:?}",
    //            i + 1,
    //            closing_prices[i],
    //            "계산 중...",
    //            signal
    //        );
    //     }
    // }

    let latest_signal = signals.last().unwrap();
    let latest_rsi = rsi_results.last().unwrap();
    (latest_rsi.clone(), latest_signal.clone())
}