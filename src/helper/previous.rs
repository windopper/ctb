/// 데이터 슬라이스에서 `current_index` 이전의 가장 마지막 저점(trough)의
/// 인덱스와 값을 찾습니다. 저점은 양 옆의 값보다 작은 지점으로 정의합니다.
///
/// # 인자
/// * `data` - 가격 또는 RSI 데이터 슬라이스
/// * `current_index` - 탐색을 시작할 기준 인덱스
///
/// # 반환값
/// * `Some((usize, f64))` - (저점 인덱스, 저점 값)
/// * `None` - 이전 저점을 찾지 못한 경우
pub fn find_previous_trough_with_index(
    data: &[f64],
    current_index: usize,
) -> Option<(usize, f64)> {
    // 유효한 저점을 찾으려면 최소 3개의 데이터 포인트(좌, 중앙, 우)가 필요합니다.
    // current_index 바로 이전부터 역순으로 탐색합니다.
    // 루프는 `current_index - 1`에서 시작하여 인덱스 1까지 갑니다.
    // 인덱스 0은 좌측 값이 없어 저점인지 판단할 수 없기 때문입니다.
    if current_index < 2 {
        return None;
    }

    // current_index 바로 이전 지점부터 역순으로 탐색
    for i in (1..current_index - 1).rev() {
        // 저점 조건: data[i]가 양 옆의 값보다 작다.
        if data[i] < data[i - 1] && data[i] < data[i + 1] {
            // 저점을 찾았으므로 즉시 인덱스와 값을 반환합니다.
            // 역순으로 찾았기 때문에 이것이 가장 마지막(최근) 저점입니다.
            return Some((i, data[i]));
        }
    }

    // 루프가 끝날 때까지 저점을 찾지 못함
    None
}

/// 데이터 슬라이스에서 `current_index` 이전의 가장 마지막 고점(peak)의
/// 인덱스와 값을 찾습니다. 고점은 양 옆의 값보다 큰 지점으로 정의합니다.
///
/// # 인자
/// * `data` - 가격 또는 RSI 데이터 슬라이스
/// * `current_index` - 탐색을 시작할 기준 인덱스
///
/// # 반환값
/// * `Some((usize, f64))` - (고점 인덱스, 고점 값)
/// * `None` - 이전 고점을 찾지 못한 경우
pub fn find_previous_peak_with_index(
    data: &[f64],
    current_index: usize,
) -> Option<(usize, f64)> {
    // 유효한 고점을 찾으려면 최소 3개의 데이터 포인트(좌, 중앙, 우)가 필요합니다.
    if current_index < 2 {
        return None;
    }

    // current_index 바로 이전 지점부터 역순으로 탐색합니다 (인덱스 1까지).
    for i in (1..current_index - 1).rev() {
        // 고점 조건: data[i]가 양 옆의 값보다 크다.
        if data[i] > data[i - 1] && data[i] > data[i + 1] {
            // 고점을 찾았으므로 즉시 인덱스와 값을 반환합니다.
            // 역순으로 찾았기 때문에 이것이 가장 마지막(최근) 고점입니다.
            return Some((i, data[i]));
        }
    }

    // 루프가 끝날 때까지 고점을 찾지 못함
    None
}