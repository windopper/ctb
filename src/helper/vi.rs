#[derive(Debug, Clone, Copy)]
pub struct ViCandle {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct VortexIndicator {
    pub vi_plus: f64,
    pub vi_minus: f64,
}

/// 볼텍스 지표를 계산합니다.
///
/// # Arguments
///
/// * `prices` - 고가, 저가, 종가를 포함하는 `PriceData` 구조체의 슬라이스
/// * `period` - 볼텍스 지표를 계산할 기간
///
/// # Returns
///
/// * `Option<Vec<VortexIndicator>>` - 계산된 볼텍스 지표 값의 벡터.
///   입력 데이터가 충분하지 않으면 `None`을 반환합니다.
pub fn calculate_vortex_indicator(prices: &Vec<ViCandle>, period: usize) -> Option<Vec<VortexIndicator>> {
    if prices.len() < period {
        return None;
    }

    let mut results = Vec::new();
    let mut sum_tr = 0.0;
    let mut sum_vm_plus = 0.0;
    let mut sum_vm_minus = 0.0;

    // 초기 기간(period) 동안의 합계 계산
    for i in 1..period {
        let current_high = prices[i].high;
        let current_low = prices[i].low;
        let prev_high = prices[i - 1].high;
        let prev_low = prices[i - 1].low;
        let prev_close = prices[i - 1].close;

        let tr = (current_high - current_low)
            .max((current_high - prev_close).abs())
            .max((current_low - prev_close).abs());
        sum_tr += tr;

        let vm_plus = (current_high - prev_low).abs();
        sum_vm_plus += vm_plus;

        let vm_minus = (current_low - prev_high).abs();
        sum_vm_minus += vm_minus;
    }

    // 첫 번째 볼텍스 지표 계산
    if sum_tr > 0.0 {
        results.push(VortexIndicator {
            vi_plus: sum_vm_plus / sum_tr,
            vi_minus: sum_vm_minus / sum_tr,
        });
    } else {
        results.push(VortexIndicator {
            vi_plus: 0.0,
            vi_minus: 0.0,
        });
    }

    // 나머지 기간에 대한 볼텍스 지표 계산 (이동 합계)
    for i in period..prices.len() {
        let current_high = prices[i].high;
        let current_low = prices[i].low;
        let prev_high = prices[i - 1].high;
        let prev_low = prices[i - 1].low;
        let prev_close = prices[i - 1].close;

        let tr = (current_high - current_low)
            .max((current_high - prev_close).abs())
            .max((current_low - prev_close).abs());
        sum_tr += tr;

        let vm_plus = (current_high - prev_low).abs();
        sum_vm_plus += vm_plus;

        let vm_minus = (current_low - prev_high).abs();
        sum_vm_minus += vm_minus;

        // 가장 오래된 데이터 제거
        let oldest_high = prices[i - period].high;
        let oldest_low = prices[i - period].low;
        let oldest_prev_close = if i - period > 0 {
            prices[i - period - 1].close
        } else {
            // 가장 오래된 데이터의 이전 종가가 없는 경우 처리
            prices[i-period].close
        };
        let oldest_prev_high = if i - period > 0 {
            prices[i - period -1].high
        } else {
            prices[i-period].high
        };
        let oldest_prev_low = if i-period > 0 {
            prices[i-period-1].low
        } else {
            prices[i-period].low
        };


        let oldest_tr = (oldest_high - oldest_low)
            .max((oldest_high - oldest_prev_close).abs())
            .max((oldest_low - oldest_prev_close).abs());
        sum_tr -= oldest_tr;

        let oldest_vm_plus = (oldest_high - oldest_prev_low).abs();
        sum_vm_plus -= oldest_vm_plus;

        let oldest_vm_minus = (oldest_low - oldest_prev_high).abs();
        sum_vm_minus -= oldest_vm_minus;

        if sum_tr > 0.0 {
            results.push(VortexIndicator {
                vi_plus: sum_vm_plus / sum_tr,
                vi_minus: sum_vm_minus / sum_tr,
            });
        } else {
            results.push(VortexIndicator {
                vi_plus: 0.0,
                vi_minus: 0.0,
            });
        }
    }

    Some(results)
}