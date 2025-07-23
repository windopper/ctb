#[derive(Debug, Clone, Copy)]
pub struct DiCandle {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct DiResult {
    pub plus_di: f64,
    pub minus_di: f64,
}

/// D+ 와 D- 를 계산하는 메인 함수
///
/// # Arguments
///
/// * `data` - `PriceData` 구조체의 슬라이스
/// * `period` - 이동 평균을 계산할 기간
///
/// # Returns
///
/// * `DiResult` 구조체의 벡터
pub fn calculate_di(data: &Vec<DiCandle>, period: usize) -> Vec<DiResult> {
    if data.len() < period {
        return Vec::new();
    }

    let mut results = Vec::with_capacity(data.len() - period + 1);
    let mut prev_data = data[0];

    let mut smoothed_plus_dm = 0.0;
    let mut smoothed_minus_dm = 0.0;
    let mut smoothed_tr = 0.0;

    // 초기 period 기간 동안의 +DM, -DM, TR의 합계를 계산
    for i in 1..period {
        let current_data = data[i];
        let (plus_dm, minus_dm) = calculate_dm(current_data, prev_data);
        let tr = calculate_tr(current_data, prev_data);

        smoothed_plus_dm += plus_dm;
        smoothed_minus_dm += minus_dm;
        smoothed_tr += tr;

        prev_data = current_data;
    }

    // 첫 번째 D+ 와 D- 계산
    let plus_di = (smoothed_plus_dm / smoothed_tr) * 100.0;
    let minus_di = (smoothed_minus_dm / smoothed_tr) * 100.0;
    results.push(DiResult { plus_di, minus_di });

    // 나머지 데이터에 대한 D+ 와 D- 계산 (지수이동평균 사용)
    for i in period..data.len() {
        let current_data = data[i];
        let (plus_dm, minus_dm) = calculate_dm(current_data, prev_data);
        let tr = calculate_tr(current_data, prev_data);

        smoothed_plus_dm = (smoothed_plus_dm - (smoothed_plus_dm / period as f64)) + plus_dm;
        smoothed_minus_dm = (smoothed_minus_dm - (smoothed_minus_dm / period as f64)) + minus_dm;
        smoothed_tr = (smoothed_tr - (smoothed_tr / period as f64)) + tr;

        let plus_di = (smoothed_plus_dm / smoothed_tr) * 100.0;
        let minus_di = (smoothed_minus_dm / smoothed_tr) * 100.0;

        results.push(DiResult { plus_di, minus_di });

        prev_data = current_data;
    }

    results
}

/// 방향성 움직임 (+DM, -DM) 계산 함수
fn calculate_dm(current: DiCandle, previous: DiCandle) -> (f64, f64) {
    let up_move = current.high - previous.high;
    let down_move = previous.low - current.low;

    let plus_dm = if up_move > down_move && up_move > 0.0 { up_move } else { 0.0 };
    let minus_dm = if down_move > up_move && down_move > 0.0 { down_move } else { 0.0 };

    (plus_dm, minus_dm)
}

/// 실질 변동폭 (TR) 계산 함수
fn calculate_tr(current: DiCandle, previous: DiCandle) -> f64 {
    let high_low = current.high - current.low;
    let high_close = (current.high - previous.close).abs();
    let low_close = (current.low - previous.close).abs();

    high_low.max(high_close).max(low_close)
}