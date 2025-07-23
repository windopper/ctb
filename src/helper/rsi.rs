

/// RSI(Relative Strength Index, 상대강도지수)는 주어진 기간 동안의 가격 변동에서 상승폭과 하락폭의 크기를 비교하여, 
/// 과매수(overbought) 또는 과매도(oversold) 상태를 판단하는 데 사용되는 대표적인 모멘텀 지표입니다.
/// 일반적으로 0~100 사이의 값을 가지며, 70 이상이면 과매수, 30 이하면 과매도로 간주합니다.
pub fn calculate_rsi(prices: &[f64], period: usize) -> Vec<f64> {
    if prices.len() < period {
        return vec![0.0; prices.len()];
    }

    let mut rsi_values = vec![0.0; prices.len()];
    let mut gains = 0.0;
    let mut losses = 0.0;

    for i in 1..period {
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
        rsi_values[period - 1] = 100.0;
    } else {
        let rs = avg_gain / avg_loss;
        rsi_values[period - 1] = 100.0 - (100.0 / (1.0 + rs));
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