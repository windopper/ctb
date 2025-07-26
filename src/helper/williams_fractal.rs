// 캔들의 고가(high)와 저가(low)를 저장하는 구조체
// Debug: println!으로 쉽게 출력하기 위함
// Copy, Clone: 값을 쉽게 복사하기 위함
#[derive(Debug, Copy, Clone)]
pub struct FractalCandle {
    pub high: f64,
    pub low: f64,
}

// 프랙탈의 종류를 나타내는 열거형 (Enum)
// PartialEq, Eq: 값 비교를 위함 (예: assert_eq! 등 테스트에서 유용)
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FractalType {
    Bullish, // 상승 프랙탈 (저점이 5개 캔들 중 가장 낮음)
    Bearish, // 하락 프랙탈 (고점이 5개 캔들 중 가장 높음)
}

/// 윌리엄스 프랙탈을 계산합니다.
///
/// # Arguments
/// * `candles` - 고가와 저가 정보를 담고 있는 캔들 데이터 슬라이스
///
/// # Returns
/// 각 캔들 위치에 프랙탈이 형성되었는지를 나타내는 `Vec<Option<FractalType>>`.
/// 프랙탈이 없으면 `None`, 있으면 `Some(FractalType)`이 됩니다.
/// 결과 벡터의 길이는 입력 `candles` 벡터의 길이와 같습니다.
pub fn calculate_williams_fractals(candles: &Vec<FractalCandle>) -> Vec<Option<FractalType>> {
    let n = candles.len();
    // 결과를 저장할 벡터를 `None`으로 초기화합니다.
    // 이렇게 하면 프랙탈이 형성되지 않은 지점은 자동으로 None이 됩니다.
    let mut fractals: Vec<Option<FractalType>> = vec![None; n];

    // 프랙탈은 5개의 캔들(중심 캔들과 좌우 2개씩)이 필요하므로,
    // 최소 5개의 캔들이 없으면 계산할 수 없습니다.
    if n < 5 {
        return fractals;
    }

    for i in 2..(candles.len() - 2) {
        if candles[i].high > candles[i - 1].high &&
           candles[i].high > candles[i - 2].high &&
           candles[i].high > candles[i + 1].high &&
           candles[i].high > candles[i + 2].high {
            fractals[i] = Some(FractalType::Bearish);
        }

        if candles[i].low < candles[i - 1].low &&
           candles[i].low < candles[i - 2].low &&
           candles[i].low < candles[i + 1].low &&
           candles[i].low < candles[i + 2].low {
            fractals[i] = Some(FractalType::Bullish);
        }
    }

    fractals
}