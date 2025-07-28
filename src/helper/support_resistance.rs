use std::collections::HashMap;

use linregress::{FormulaRegressionBuilder, RegressionDataBuilder};

// 캔들 데이터를 표현하는 구조체 (기존과 동일)
#[derive(Debug, Clone, Copy)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// 대각선 지지/저항선을 표현하는 구조체 (기존과 동일)
#[derive(Debug, Clone)]
pub struct TrendLine {
    /// 기울기 (price / index)
    pub slope: f64,
    /// y절편
    pub intercept: f64,
}

// Python의 pythag 함수와 동일
fn pythag(pt1: (usize, f64), pt2: (usize, f64)) -> f64 {
    let a_sq = (pt2.0 as f64 - pt1.0 as f64).powi(2);
    let b_sq = (pt2.1 - pt1.1).powi(2);
    (a_sq + b_sq).sqrt()
}

// Python의 local_min_max 함수를 Rust로 구현
fn find_local_min_max(pts: &[f64]) -> (Vec<(usize, f64)>, Vec<(usize, f64)>) {
    if pts.len() < 3 {
        return (Vec::new(), Vec::new());
    }

    let mut local_min = Vec::new();
    let mut local_max = Vec::new();
    
    // 초기 prev_pts 설정
    let mut prev_pts = [(0, pts[0]), (1, pts[1])];

    for i in 1..(pts.len() - 1) {
        let mut append_to: Option<String> = None;
        if pts[i - 1] > pts[i] && pts[i] < pts[i + 1] {
            append_to = Some("min".to_string());
        } else if pts[i - 1] < pts[i] && pts[i] > pts[i + 1] {
            append_to = Some("max".to_string());
        }

        if let Some(label) = append_to {
            let curr_point = (i, pts[i]);
            // 이전 변곡점과의 거리가 충분한지 확인
            let prev_distance = pythag(prev_pts[0], prev_pts[1]) * 0.5;
            let curr_distance = pythag(prev_pts[1], curr_point);

            if curr_distance >= prev_distance {
                if label == "min" {
                    local_min.push(curr_point);
                } else {
                    local_max.push(curr_point);
                }
                // 이전 변곡점 업데이트
                prev_pts[0] = prev_pts[1];
                prev_pts[1] = curr_point;
            }
        }
    }
    (local_min, local_max)
}


/// 선형 회귀 분석을 통해 기울기(slope)와 y절편(intercept)을 계산합니다.
///
/// # Arguments
/// * `pts` - (x, y) 좌표 쌍의 슬라이스. x는 usize, y는 f64입니다.
///
/// # Returns
/// * `Some((slope, intercept))` - 분석에 성공한 경우 (기울기, y절편) 튜플을 반환합니다.
/// * `None` - 데이터 포인트가 2개 미만이거나 분석에 실패한 경우 None을 반환합니다.
fn regression_coef(pts: &[(usize, f64)]) -> Option<(f64, f64)> {
    if pts.len() < 2 {
        return None;
    }

    // 1. 입력 데이터를 x와 y 벡터로 분리합니다.
    let (x_values, y_values): (Vec<f64>, Vec<f64>) =
        pts.iter().map(|(i, v)| (*i as f64, *v)).unzip();

    // 2. `build_from`을 사용하여 회귀 분석용 데이터를 생성합니다.
    // 라이브러리는 (이름, 값 벡터) 형태의 데이터를 요구합니다.
    let data_vec = vec![("X", x_values), ("Y", y_values)];
    let data = RegressionDataBuilder::new().build_from(data_vec).ok()?;

    // 3. R 스타일의 수식을 정의합니다. "Y"를 "X"에 대해 회귀 분석합니다.
    let formula = "Y ~ X";

    // 4. 수식과 데이터를 기반으로 모델을 학습시킵니다.
    let model = FormulaRegressionBuilder::new()
        .data(&data)
        .formula(formula)
        .fit().ok()?;

    // 5. 학습된 모델에서 파라미터를 가져옵니다.
    // `model.parameters()`는 파라미터 값의 슬라이스(&[f64])를 반환합니다.
    // 순서는 [Intercept, Slope] 입니다.
    let params = model.parameters();
    if params.len() < 2 {
        return None; // 안전장치: 파라미터가 2개 미만이면 실패 처리
    }

    let intercept = params[0]; // 첫 번째 값은 y절편(Intercept)
    let slope = params[1];     // 두 번째 값은 기울기(Slope)

    Some((slope, intercept))
}


// 데이터 스무딩을 위한 간단한 이동 평균 함수
fn simple_moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    if window_size == 0 {
        return data.to_vec();
    }
    let mut smoothed = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        let start = i.saturating_sub(window_size / 2);
        let end = (i + window_size / 2 + 1).min(data.len());
        let window = &data[start..end];
        let sum: f64 = window.iter().sum();
        smoothed.push(sum / window.len() as f64);
    }
    smoothed
}


/// Savitzky-Golay 필터와 선형 회귀를 사용하여 지지/저항 추세선을 계산합니다.
///
/// # Returns
///
/// `(Option<TrendLine>, Option<TrendLine>)` - (지지선, 저항선)
pub fn calculate_trend_lines_with_regression(
    candles: &[Candle],
) -> (Option<TrendLine>, Option<TrendLine>) {
    if candles.is_empty() {
        return (None, None);
    }

    let series: Vec<f64> = candles.iter().map(|c| c.close).collect();

    // 1. 데이터 스무딩
    let month_diff = (series.len() / 30).max(1);
    let smooth_window = 2 * month_diff + 3;
    let smoothed_pts = simple_moving_average(&series, smooth_window);
    
    // 2. 지역 최소/최대점 찾기
    let (local_min, local_max) = find_local_min_max(&smoothed_pts);

    // 3. 선형 회귀로 추세선 계산
    let support_line = if let Some((slope, intercept)) = regression_coef(&local_min) {
        Some(TrendLine { slope, intercept })
    } else {
        None
    };

    let resistance_line = if let Some((slope, intercept)) = regression_coef(&local_max) {
        Some(TrendLine { slope, intercept })
    } else {
        None
    };

    (support_line, resistance_line)
}
