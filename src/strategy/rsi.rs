use crate::core::signal::Signal;

pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![0.0; prices.len()];
    }

    let mut rsi_values = vec![0.0; prices.len()];
    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..=period {
        let change = prices[i]  - prices[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses += change.abs();
        }
    }

    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        rsi_values[period] = 100.0;
    } else {
        let rs = avg_gain / avg_loss;
        rsi_values[period] = 100.0 - (100.0 / (1.0 + rs));
    }

    for i in (period + 1)..prices.len() {
        let change = prices[i] - prices[i - 1];
        let (current_gain, current_loss) = if change > 0.0 {
            (change, 0.0)
        } else {
            (0.0, -change)
        };

        avg_gain = (avg_gain * (period - 1) as f64 + current_gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + current_loss) / period as f64;

        if avg_loss == 0.0 {
            rsi_values[i] = 100.0;
        } else {
            let rs = avg_gain / avg_loss;
            rsi_values[i] = 100.0 - (100.0 / (1.0 + rs));
        }
    }

    rsi_values
}

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

pub fn run() {
    const RSI_PERIOD: usize = 14;
    const BUY_THRESHOLD: f64 = 30.0;
    const SELL_THRESHOLD: f64 = 70.0;

    let close_prices = vec![
        44.34, 44.09, 44.15, 43.61, 44.33, 44.83, 45.10, 45.42, 45.84, 46.08,
        45.89, 46.03, 45.61, 46.28, 46.28, 46.00, 46.03, 46.41, 46.22, 45.64,
        46.21, 46.25, 45.71, 46.45, 45.78, 45.35, 44.03, 44.18, 44.22, 44.57,
        43.42, 42.66, 43.13, 43.43, 43.79, 44.47, 44.85, 45.09, 45.11, 44.62,
        45.12, 45.55, 46.43, 46.44, 46.45, 47.21, 47.45, 48.01, 48.21, 47.55,
    ];

    let rsi_results = calculate_rsi(&close_prices, RSI_PERIOD);

    let signals = generate_signals(&rsi_results, BUY_THRESHOLD, SELL_THRESHOLD);

    println!("{:<5} | {:<5} | {:<5} | {:<10}", "인덱스", "종가", "RSI", "신호");
    println!("--------------------------------");

    for i in 0..close_prices.len() {
        let rsi_val = rsi_results.get(i).cloned().unwrap_or(0.0);
        let signal = signals.get(i).unwrap_or(&Signal::Hold);

        if i >= RSI_PERIOD {
            println!(
               "{:<5} | {:<10.2} | {:<15.2} | {:?}",
               i + 1,
               close_prices[i],
               rsi_val,
               signal
           );
        } else {
           println!(
               "{:<5} | {:<10.2} | {:<15} | {:?}",
               i + 1,
               close_prices[i],
               "계산 중...",
               signal
           );
        }
    }
}