/// Exponential Moving Average
/// 최근 데이터에 더 큰 가중치를 부여하여 이동 평균을 계산하는 기술적 지표
/// 
/// 단순 이동 평균보다 최근 가격 변공에 더 민감하게 반응.
pub fn calculate_ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period {
        return vec![0.0; data.len()];
    }

    let mut ema_values = vec![0.0; data.len()];
    let multiplier = 2.0 / (period as f64 + 1.0);

    // 첫 EMA는 단순이동평균(SMA)으로 계산
    let initial_sma: f64 = data.iter().take(period).sum();
    ema_values[period - 1] = initial_sma / period as f64;

    // 나머지 EMA 계산
    for i in period..data.len() {
        ema_values[i] = (data[i] - ema_values[i - 1]) * multiplier + ema_values[i - 1];
    }

    ema_values
}