// 캔들 또는 시계열 데이터를 표현하기 위한 구조체
#[derive(Debug, Clone, Copy)]
pub struct VWMACandle {
    pub close: f64,
    pub volume: f64,
}

/// 거래량 가중 이동 평균(VWMA)을 계산합니다.
///
/// # Arguments
/// * `data` - 종가(`close`)와 거래량(`volume`)을 포함하는 데이터 포인트의 슬라이스
/// * `period` - 이동 평균을 계산할 기간 (예: 5, 20)
///
/// # Returns
/// * `Vec<Option<f64>>` - 각 데이터 포인트에 대한 VWMA 값.
///   기간보다 데이터가 적은 초기 구간은 `None`으로 채워집니다.
pub fn calculate_vwma(data: &Vec<VWMACandle>, period: usize) -> Vec<Option<f64>> {
    // 기간이 0이거나 데이터가 비어있으면 빈 벡터를 반환
    if period == 0 || data.is_empty() {
        return vec![];
    }

    let mut results: Vec<Option<f64>> = Vec::with_capacity(data.len());

    for i in 0..data.len() {
        // 현재 인덱스까지의 데이터 포인트 수가 period보다 작은 경우
        if i + 1 < period {
            results.push(None);
        } else {
            // 현재 인덱스를 포함하여 period만큼의 데이터를 슬라이싱
            let window = &data[(i + 1 - period)..=i];

            let mut sum_price_volume = 0.0;
            let mut sum_volume = 0.0;

            for point in window {
                sum_price_volume += point.close * point.volume;
                sum_volume += point.volume;
            }

            // 거래량의 합이 0보다 큰 경우에만 계산 (0으로 나누기 방지)
            if sum_volume > 0.0 {
                let vwma_value = sum_price_volume / sum_volume;
                results.push(Some(vwma_value));
            } else {
                // 해당 기간의 거래량이 모두 0인 경우, VWMA는 정의되지 않음
                results.push(None);
            }
        }
    }

    results
}