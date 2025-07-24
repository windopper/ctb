use crate::helper::rsi::{calculate_rsi};

/// 제공된 `calculate_rsi` 함수를 사용하여 Stochastic RSI의 %K와 %D 라인을 계산합니다.
///
/// # 인자
///
/// * `prices` - 종가 데이터 슬라이스
/// * `stoch_period` - Stochastic 및 RSI 계산 기간 (일반적으로 14)
/// * `d_period` - %D 계산을 위한 이동 평균 기간 (일반적으로 3)
///
/// # 반환값
///
/// * `(Vec<f64>, Vec<f64>)` - (%K 값 벡터, %D 값 벡터) 튜플
pub fn calculate_stochastic_rsi(
    prices: &[f64],
    stoch_period: usize,
    d_period: usize,
) -> (Vec<f64>, Vec<f64>) {
    // 1. 제공된 함수로 RSI 값 계산
    let rsi_values = calculate_rsi(prices, stoch_period);

    // 2. 유효한 RSI 값만 추출합니다.
    // calculate_rsi는 첫 `stoch_period`개의 값을 0으로 채우므로, 그 이후의 값만 사용합니다.
    let valid_rsi_values: Vec<f64> = rsi_values
        .into_iter()
        .collect();

    // StochRSI를 계산하기에 충분한 RSI 값이 있는지 확인합니다.
    if valid_rsi_values.len() < stoch_period {
        return (vec![], vec![]);
    }

    // 3. %K 라인을 계산합니다. `windows`를 사용하면 코드가 간결해집니다.
    let k_values: Vec<f64> = valid_rsi_values
        .windows(stoch_period)
        .map(|window| {
            let current_rsi = window[stoch_period - 1];
            
            // min/max 값을 효율적으로 찾습니다.
            let min_rsi = window.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max_rsi = window.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            if max_rsi == min_rsi {
                // 기간 내 RSI 변동이 없으면 0으로 처리 (분모 0 방지)
                0.0
            } else {
                (current_rsi - min_rsi) / (max_rsi - min_rsi) * 100.0
            }
        })
        .collect();

    // 4. %D 라인을 계산합니다.
    // %D를 계산하기에 충분한 %K 값이 있는지 확인합니다.
    if k_values.len() < d_period {
        // %K 값은 있지만 %D 값은 없는 경우, %K만 반환하고 %D는 비워둡니다.
        return (k_values, vec![]);
    }

    let d_values: Vec<f64> = k_values
        .windows(d_period)
        .map(|window| window.iter().sum::<f64>() / d_period as f64)
        .collect();

    // 5. %D 값과 짝을 맞추기 위해 %K 값의 앞부분을 잘라냅니다.
    // %D는 %K보다 `d_period - 1`개만큼 데이터가 적기 때문입니다.
    let k_values_aligned = k_values[d_period - 1..].to_vec();

    (k_values_aligned, d_values)
}