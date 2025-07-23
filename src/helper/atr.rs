

pub struct AtrCandle {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// 특정 기간 동안 주가가 평균적으로 얼마나 움직였는지 나타냄
/// 
/// ATR 수치가 높으면 시장의 변동성이 크다는 의미, 낮으면 변동성이 작고 시장이 안정적이거나 횡보하고 있음을 의미
/// 
/// TR(True Range) = max(high - low, abs(high - prev_close), abs(low - prev_close))
/// 
/// ATR = TR의 평균
/// 
/// candles는 최신 데이터가 배열의 끝에 와야 함
pub fn calculate_atr(candles: &Vec<AtrCandle>, period: usize) -> Vec<f64> {
    if candles.len() < period {
        return Vec::new();
    }

    let mut atr_values = Vec::with_capacity(candles.len());
    let mut true_ranges = Vec::with_capacity(candles.len());
    
    // 첫 번째 TR 계산
    let first_tr = candles[0].high - candles[0].low;
    true_ranges.push(first_tr);

    // 나머지 TR 계산
    for i in 1..candles.len() {
        let prev_close = candles[i - 1].close;
        let current_high = candles[i].high;
        let current_low = candles[i].low;

        let tr1 = current_high - current_low;
        let tr2 = (current_high - prev_close).abs();
        let tr3 = (current_low - prev_close).abs();

        let true_range = tr1.max(tr2).max(tr3);
        true_ranges.push(true_range);
    }

    // 첫 ATR
    let first_atr = true_ranges.iter().take(period).sum::<f64>() / period as f64;
    atr_values.push(first_atr);

    let mut prev_atr = first_atr;
    for i in period..candles.len() {
        let current_atr = (prev_atr * (period - 1) as f64 + true_ranges[i]) / period as f64;
        atr_values.push(current_atr);
        prev_atr = current_atr;
    }
    
    atr_values
}