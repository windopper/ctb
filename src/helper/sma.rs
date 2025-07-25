
pub fn calculate_sma(data: &[f64], period: usize) -> Option<f64> {
    if data.len() < period {
        None
    } else {
        Some(data.iter().rev().take(period).sum::<f64>() / period as f64)
    }
}