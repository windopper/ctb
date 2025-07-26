pub struct VwapCandle {
    pub close: f64,
    pub volume: f64,
    pub high: f64,
    pub low: f64,
}

pub struct VwapBand {
    pub vwap: f64,
    pub upper_band: f64,
    pub lower_band: f64,
}

/// 주어진 기간과 승수를 사용하여 VWAP 밴드를 계산합니다.
///
/// # Arguments
///
/// * `candles` - OHLCV 데이터 슬라이스
/// * `period` - VWAP 및 표준편차 계산에 사용할 기간
/// * `multiplier` - 밴드 폭을 결정하기 위한 표준편차 승수
///
/// # Returns
///
/// * 각 캔들에 대한 `VwapBand` 결과를 담은 벡터
pub fn calculate_vwap_bands(candles: &Vec<VwapCandle>, period: usize, multiplier: f64) -> Vec<Option<VwapBand>> {
    let mut results = Vec::with_capacity(candles.len());

    // 기간보다 데이터가 적으면 계산 불가
    if candles.len() < period {
        for _ in 0..candles.len() {
            results.push(None);
        }
        return results;
    }

    // 초기값 채우기
    for _ in 0..period - 1 {
        results.push(None);
    }

    // 롤링 윈도우를 사용하여 계산
    for i in (period - 1)..candles.len() {
        let window = &candles[(i + 1 - period)..=i];

        let mut typical_price_volume_sum = 0.0;
        let mut volume_sum = 0.0;

        for candle in window {
            let typical_price = (candle.high + candle.low + candle.close) / 3.0;
            typical_price_volume_sum += typical_price * candle.volume;
            volume_sum += candle.volume;
        }

        if volume_sum == 0.0 {
            results.push(None);
            continue;
        }

        let vwap = typical_price_volume_sum / volume_sum;

        let mut variance_sum = 0.0;
        for candle in window {
            let typical_price = (candle.high + candle.low + candle.close) / 3.0;
            let deviation = typical_price - vwap;
            variance_sum += deviation.powi(2) * candle.volume;
        }

        let variance = variance_sum / volume_sum;
        let std_dev = variance.sqrt();

        let upper_band = vwap + (std_dev * multiplier);
        let lower_band = vwap - (std_dev * multiplier);

        results.push(Some(VwapBand {
            vwap,
            upper_band,
            lower_band,
        }));
    }

    results
}