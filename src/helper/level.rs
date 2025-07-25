use crate::core::candle::CandleTrait;

/// 가격 목록을 받아 근사 레벨(지지/저항)을 찾는 헬퍼 함수
pub fn find_levels(prices: &[f64], tolerance_percent: f64, min_touches: usize) -> Vec<f64> {
    if prices.is_empty() {
        return vec![];
    }

    let mut sorted_prices = prices.to_vec();
    sorted_prices.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut clusters: Vec<Vec<f64>> = Vec::new();
    if let Some(&first_price) = sorted_prices.first() {
        clusters.push(vec![first_price]);
    }

    for &price in sorted_prices.iter().skip(1) {
        let last_cluster_avg = clusters.last().unwrap().iter().sum::<f64>() / clusters.last().unwrap().len() as f64;
        
        if (price - last_cluster_avg).abs() <= last_cluster_avg * tolerance_percent {
            clusters.last_mut().unwrap().push(price);
        } else {
            clusters.push(vec![price]);
        }
    }

    clusters.into_iter()
        .filter(|cluster| cluster.len() >= min_touches)
        .map(|cluster| cluster.iter().sum::<f64>() / cluster.len() as f64)
        .collect()
}

/// 캔들 목록을 받아 지지선과 저항선을 계산합니다.
///
/// # Arguments
/// * `candles` - 캔들 데이터 벡터
/// * `tolerance_percent` - 가격 차이 허용 오차 (예: 0.005 = 0.5%)
/// * `min_touches` - 유의미한 레벨로 간주되기 위한 최소 터치 횟수
/// * `recent_candles` - 분석에 사용할 최근 캔들 수
///
/// # Returns
/// 튜플: `(support_levels, resistance_levels)`
pub fn find_support_resistance(
    candles: Vec<&Box<dyn CandleTrait>>,
    tolerance_percent: f64,
    min_touches: usize,
    recent_candles: usize, // 최근 캔들 수를 지정하는 파라미터 추가
) -> (Vec<f64>, Vec<f64>) {
    // 최근 N개의 캔들만 선택
    let recent_candles = if recent_candles > candles.len() {
        candles.len()
    } else {
        recent_candles
    };
    let recent_candles = &candles[candles.len() - recent_candles..];

    // 최근 캔들의 저가와 고가 추출
    let lows: Vec<f64> = recent_candles.iter().map(|c| c.get_low_price()).collect();
    let highs: Vec<f64> = recent_candles.iter().map(|c| c.get_high_price()).collect();

    let support_levels = find_levels(&lows, tolerance_percent, min_touches);
    let resistance_levels = find_levels(&highs, tolerance_percent, min_touches);

    (support_levels, resistance_levels)
}