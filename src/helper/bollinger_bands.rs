pub struct BollingerBands {
    pub upper: f64,
    pub middle: f64,
    pub lower: f64,
}

/// 볼린저 밴드를 계산합니다.
/// 
/// 주가가 상한선에 닿거나 넘어서면 과매수 상태로, 하한선에 닿거나 아래로 떨어지면 과매도 상태로 해석하여 각각 매수 및 매도 신호로 고려할 수 있음.
///
/// # Arguments
/// * `closes` - 종가 데이터 슬라이스 (최신 데이터가 배열의 끝에 와야 함)
/// * `period` - 이동평균 및 표준편차 계산 기간 (일반적으로 20)
/// * `multiplier` - 표준편차에 곱할 승수 (일반적으로 2.0)
///
/// # Returns
/// * `Vec<BollingerBands>` - 각 시점의 볼린저 밴드 값 벡터
pub fn calculate_bollinger_bands(closes: &Vec<f64>, period: usize, multiplier: f64) -> Vec<BollingerBands> {
    if closes.len() < period {
        return Vec::new();
    }

    let mut bb_values = Vec::with_capacity(closes.len() - period + 1);

    // 슬라이딩 윈도우를 사용하여 기간(period)만큼 데이터를 묶어 계산
    for window in closes.windows(period) {
        // 1. 중간 밴드 (SMA) 계산
        let sum: f64 = window.iter().sum();
        let middle_band = sum / period as f64;

        // 2. 표준편차 계산
        let variance = window.iter()
            .map(|value| {
                let diff = value - middle_band;
                diff * diff
            })
            .sum::<f64>() / period as f64;
        let std_dev = variance.sqrt();

        // 3. 상단 및 하단 밴드 계산
        let upper_band = middle_band + std_dev * multiplier;
        let lower_band = middle_band - std_dev * multiplier;

        bb_values.push(BollingerBands {
            upper: upper_band,
            middle: middle_band,
            lower: lower_band,
        });
    }

    bb_values
}