#[derive(Debug, PartialEq)]
pub enum Trend {
    Uptrend,   // 상승 추세
    Downtrend, // 하락 추세
    Sideways,  // 횡보
}

/// 간단한 이동 평균(SMA)을 계산하는 헬퍼 함수
fn calculate_sma(data: &[f64], period: usize) -> Option<f64> {
    if data.len() < period {
        return None;
    }
    // 데이터의 마지막 'period'개 만큼을 사용
    let sum: f64 = data.iter().rev().take(period).sum();
    Some(sum / period as f64)
}


/// 이동 평균 교차를 사용하여 값의 추세를 분석합니다.
///
/// # Arguments
/// * `prices` - 분석할 가격 데이터 슬라이스.
/// * `short_period` - 단기 이동 평균 기간 (예: 20).
/// * `long_period` - 장기 이동 평균 기간 (예: 50).
///
/// # Returns
/// * `Some(Trend)` - 데이터가 충분할 경우 추세 분석 결과 반환.
/// * `None` - 데이터가 부족하거나 기간 설정이 잘못된 경우.
pub fn analyze_trend_moving_average(values: &[f64], short_period: usize, long_period: usize) -> Option<Trend> {
    if long_period <= short_period || values.len() < long_period {
        return None; // 기간 설정이 잘못되었거나 데이터가 부족한 경우
    }

    // 마지막 값을 기준으로 단기/장기 이동 평균 계산
    let short_ma = calculate_sma(values, short_period)?;
    let long_ma = calculate_sma(values, long_period)?;

    // 두 MA의 차이를 백분율로 계산하여 횡보 구간 판단
    let difference_ratio = (short_ma - long_ma) / long_ma;

    // 횡보로 판단할 임계값
    const SIDEWAYS_THRESHOLD_RATIO: f64 = 0.01;

    if difference_ratio > SIDEWAYS_THRESHOLD_RATIO {
        Some(Trend::Uptrend)
    } else if difference_ratio < -SIDEWAYS_THRESHOLD_RATIO {
        Some(Trend::Downtrend)
    } else {
        Some(Trend::Sideways)
    }
}