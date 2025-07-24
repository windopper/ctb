use std::cmp::{max, Ord};

// OHLCV 데이터를 담을 구조체
#[derive(Debug, Clone, Copy)]
pub struct Ohlcv {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// 슈퍼트렌드 결과 값을 담을 구조체
#[derive(Debug, Clone, Copy)]
pub struct SupertrendOutput {
    pub value: f64,
    pub is_uptrend: bool,
}

/// 실제 범위(True Range)를 계산합니다.
fn true_range(current: Ohlcv, previous_close: f64) -> f64 {
    let high_minus_low = current.high - current.low;
    let high_minus_prev_close = (current.high - previous_close).abs();
    let low_minus_prev_close = (current.low - previous_close).abs();
    high_minus_low.max(high_minus_prev_close).max(low_minus_prev_close)
}

/// 평균 실제 범위(Average True Range)를 계산합니다.
fn calculate_atr(data: &[Ohlcv], period: usize) -> Vec<f64> {
    if data.len() < period {
        return vec![];
    }

    let mut trs = Vec::with_capacity(data.len());
    trs.push(data[0].high - data[0].low); // 첫 번째 TR

    for i in 1..data.len() {
        trs.push(true_range(data[i], data[i - 1].close));
    }

    let mut atrs = Vec::with_capacity(data.len());
    let first_atr: f64 = trs.iter().take(period).sum::<f64>() / period as f64;
    atrs.push(first_atr);

    for i in period..data.len() {
        let prev_atr = atrs[atrs.len() - 1];
        let current_atr = (prev_atr * (period - 1) as f64 + trs[i]) / period as f64;
        atrs.push(current_atr);
    }

    atrs
}

/// 슈퍼트렌드 값을 계산하는 함수
///
/// # 인수
/// * `data` - OHLCV 데이터 슬라이스
/// * `period` - ATR 계산 기간
/// * `multiplier` - ATR에 적용할 승수
pub fn calculate_supertrend(data: &[Ohlcv], period: usize, multiplier: f64) -> Vec<SupertrendOutput> {
    if data.len() < period { return vec![]; }
    let atrs = calculate_atr(data, period); // ATR 계산 함수는 별도로 구현 가정
    let mut results: Vec<SupertrendOutput> = Vec::new();

    for i in period..data.len() {
        let src = (data[i].high + data[i].low) / 2.0;
        let atr = atrs[i - period];
        let upper_band = src + multiplier * atr;
        let lower_band = src - multiplier * atr;

        let prev_lower_band = if i == period { lower_band } else { results.last().unwrap().value };
        let prev_upper_band = if i == period { upper_band } else { results.last().unwrap().value };

        let final_lower_band = if lower_band > prev_lower_band || data[i - 1].close < prev_lower_band {
            lower_band
        } else {
            prev_lower_band
        };
        let final_upper_band = if upper_band < prev_upper_band || data[i - 1].close > prev_upper_band {
            upper_band
        } else {
            prev_upper_band
        };

        let (direction, supertrend) = if results.is_empty() {
            (1, final_upper_band) // 초기 방향 1 (상승)
        } else {
            let prev_supertrend = results.last().unwrap().value;
            if prev_supertrend == prev_upper_band {
                if data[i].close > final_upper_band { (-1, final_lower_band) } else { (1, final_upper_band) }
            } else {
                if data[i].close < final_lower_band { (1, final_upper_band) } else { (-1, final_lower_band) }
            }
        };

        let is_uptrend = direction == 1;
        results.push(SupertrendOutput { value: supertrend, is_uptrend });
    }
    results
}