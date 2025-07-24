use crate::helper::ema::calculate_ema;

/// MACD(Moving Average Convergence Divergence)
pub fn calculate_macd(closing_prices: &[f64], period1: usize, period2: usize, period3: usize) -> Option<f64> {
    let ema_12 = calculate_ema(&closing_prices, period1);
    let ema_26 = calculate_ema(&closing_prices, period2);

    let mut macd_line = vec![0.0; closing_prices.len()];
    for i in 25..closing_prices.len() { // 26주기 EMA가 계산된 이후부터
        macd_line[i] = ema_12[i] - ema_26[i];
    }

    let signal_line = calculate_ema(&macd_line, period3);

    let mut histogram = vec![0.0; closing_prices.len()];
    for i in 25..closing_prices.len() { // 유효한 데이터 범위
        histogram[i] = macd_line[i] - signal_line[i];
    }

    Some(histogram[histogram.len() - 1])
}


