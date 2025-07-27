use crate::core::candle::{Candle, CandleTrait};

/// 망치형 캔들인지 확인하는 함수
/// 
/// 망치형 캔들의 특징:
/// - 작은 몸통 (시가와 종가의 차이가 작음)
/// - 긴 아래 그림자 (저가가 시가/종가보다 훨씬 낮음)
/// - 짧은 위 그림자 (고가가 시가/종가와 비슷함)
pub fn is_hammer(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    
    // 전체 캔들 크기
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 망치형 조건 (조정됨):
    // 1. 아래 그림자가 전체 범위의 40% 이상
    // 2. 위 그림자가 전체 범위의 35% 이하
    // 3. 몸통이 전체 범위의 35% 이하
    lower_shadow_size >= total_range * 0.4 &&
    upper_shadow_size <= total_range * 0.35 &&
    body_size <= total_range * 0.35
}

/// 역망치형 캔들인지 확인하는 함수
/// 
/// 역망치형 캔들의 특징:
/// - 작은 몸통 (시가와 종가의 차이가 작음)
/// - 긴 위 그림자 (고가가 시가/종가보다 훨씬 높음)
/// - 짧은 아래 그림자 (저가가 시가/종가와 비슷함)
pub fn is_inverted_hammer(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    
    // 전체 캔들 크기
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 역망치형 조건 (조정됨):
    // 1. 위 그림자가 전체 범위의 40% 이상
    // 2. 아래 그림자가 전체 범위의 35% 이하
    // 3. 몸통이 전체 범위의 35% 이하
    upper_shadow_size >= total_range * 0.4 &&
    lower_shadow_size <= total_range * 0.35 &&
    body_size <= total_range * 0.35
}

/// 아래 꼬리 캔들인지 확인하는 함수
/// 
/// 아래 꼬리 캔들의 특징:
/// - 작은 몸통 (시가와 종가의 차이가 작음)
/// - 긴 아래 그림자 (저가가 시가/종가보다 훨씬 낮음)
/// - 짧은 위 그림자 (고가가 시가/종가와 비슷함)
/// - 망치형과 유사하지만 더 엄격한 조건
pub fn is_lower_shadow_candle(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    
    // 전체 캔들 크기
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 아래 꼬리 캔들 조건:
    // 1. 아래 그림자가 전체 범위의 50% 이상
    // 2. 위 그림자가 전체 범위의 15% 이하
    // 3. 몸통이 전체 범위의 35% 이하
    lower_shadow_size >= total_range * 0.5 &&
    upper_shadow_size <= total_range * 0.15 &&
    body_size <= total_range * 0.35
}

/// 윗 꼬리 캔들인지 확인하는 함수
/// 
/// 윗 꼬리 캔들의 특징:
/// - 작은 몸통 (시가와 종가의 차이가 작음)
/// - 긴 위 그림자 (고가가 시가/종가보다 훨씬 높음)
/// - 짧은 아래 그림자 (저가가 시가/종가와 비슷함)
/// - 역망치형과 유사하지만 더 엄격한 조건
pub fn is_upper_shadow_candle(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    
    // 전체 캔들 크기
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 윗 꼬리 캔들 조건:
    // 1. 위 그림자가 전체 범위의 50% 이상
    // 2. 아래 그림자가 전체 범위의 15% 이하
    // 3. 몸통이 전체 범위의 35% 이하
    upper_shadow_size >= total_range * 0.5 &&
    lower_shadow_size <= total_range * 0.15 &&
    body_size <= total_range * 0.35
}

/// 장대 양봉형 캔들인지 확인하는 함수
/// 
/// 장대 양봉형 캔들의 특징:
/// - 큰 몸통 (시가와 종가의 차이가 큼, 종가가 시가보다 높음)
/// - 이전 봉들에 비해 크기가 매우 큼
/// - 강한 매수세를 나타냄
pub fn is_long_bullish_candle(candle: &Candle, previous_candles: &[Candle]) -> bool {
    let body_size = candle.get_trade_price() - candle.get_opening_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 양봉이어야 함 (종가가 시가보다 높음)
    if body_size <= 0.0 {
        return false;
    }
    
    // 몸통이 전체 범위의 70% 이상이어야 함
    if body_size < total_range * 0.7 {
        return false;
    }
    
    // 이전 캔들들과 비교하여 크기가 큰지 확인
    if !previous_candles.is_empty() {
        let avg_body_size: f64 = previous_candles.iter()
            .map(|c| (c.get_trade_price() - c.get_opening_price()).abs())
            .sum::<f64>() / previous_candles.len() as f64;
        
        // 현재 몸통이 평균의 2배 이상이어야 함
        if body_size < avg_body_size * 2.0 {
            return false;
        }
    }
    
    true
}

/// 장대 음봉형 캔들인지 확인하는 함수
/// 
/// 장대 음봉형 캔들의 특징:
/// - 큰 몸통 (시가와 종가의 차이가 큼, 종가가 시가보다 낮음)
/// - 이전 봉들에 비해 크기가 매우 큼
/// - 강한 매도세를 나타냄
pub fn is_long_bearish_candle(candle: &Candle, previous_candles: &[Candle]) -> bool {
    let body_size = candle.get_opening_price() - candle.get_trade_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 음봉이어야 함 (종가가 시가보다 낮음)
    if body_size <= 0.0 {
        return false;
    }
    
    // 몸통이 전체 범위의 70% 이상이어야 함
    if body_size < total_range * 0.7 {
        return false;
    }
    
    // 이전 캔들들과 비교하여 크기가 큰지 확인
    if !previous_candles.is_empty() {
        let avg_body_size: f64 = previous_candles.iter()
            .map(|c| (c.get_trade_price() - c.get_opening_price()).abs())
            .sum::<f64>() / previous_candles.len() as f64;
        
        // 현재 몸통이 평균의 2배 이상이어야 함
        if body_size < avg_body_size * 2.0 {
            return false;
        }
    }
    
    true
}

/// 상승 샅바형 캔들인지 확인하는 함수
/// 
/// 상승 샅바형 캔들의 특징:
/// - 시가가 장중 최저가
/// - 장중에서 주가가 상승하면서 윗꼬리양봉 형태
/// - 하락추세에서 나타나면 기술적 반등 예상
pub fn is_rising_shooting_star(candle: &Candle) -> bool {
    let body_size = candle.get_trade_price() - candle.get_opening_price();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price();
    let lower_shadow_size = candle.get_opening_price() - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 양봉이어야 함
    if body_size <= 0.0 {
        return false;
    }
    
    // 시가가 저가와 거의 같아야 함 (상승 샅바형의 특징)
    if lower_shadow_size > total_range * 0.1 {
        return false;
    }
    
    // 위 그림자가 있어야 함
    if upper_shadow_size < total_range * 0.2 {
        return false;
    }
    
    true
}

/// 하락 샅바형 캔들인지 확인하는 함수
/// 
/// 하락 샅바형 캔들의 특징:
/// - 종가가 장중 최저가
/// - 장중에서 주가가 하락하면서 밑꼬리음봉 형태
/// - 상승추세에서 나타나면 기술적 하락 예상
pub fn is_falling_shooting_star(candle: &Candle) -> bool {
    let body_size = candle.get_opening_price() - candle.get_trade_price();
    let upper_shadow_size = candle.get_high_price() - candle.get_opening_price();
    let lower_shadow_size = candle.get_trade_price() - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 음봉이어야 함
    if body_size <= 0.0 {
        return false;
    }
    
    // 종가가 저가와 거의 같아야 함 (하락 샅바형의 특징)
    if lower_shadow_size > total_range * 0.1 {
        return false;
    }
    
    // 위 그림자가 있어야 함
    if upper_shadow_size < total_range * 0.2 {
        return false;
    }
    
    true
}

/// 교수형 캔들인지 확인하는 함수
/// 
/// 교수형 캔들의 특징:
/// - 작은 몸통
/// - 긴 아래 그림자
/// - 상승추세의 고점에서 나타나면 하락 전환 신호
pub fn is_hanging_man(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 교수형 조건 (조정됨):
    // 1. 아래 그림자가 전체 범위의 60% 이상
    // 2. 위 그림자가 전체 범위의 10% 이하
    // 3. 몸통이 전체 범위의 30% 이하
    lower_shadow_size >= total_range * 0.6 &&
    upper_shadow_size <= total_range * 0.1 &&
    body_size <= total_range * 0.3
}

/// 유성형 캔들인지 확인하는 함수
/// 
/// 유성형 캔들의 특징:
/// - 작은 몸통
/// - 긴 위 그림자
/// - 상승추세의 고점에서 나타나면 하락 전환 신호
pub fn is_shooting_star(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 유성형 조건 (역망치형과 유사하지만 더 엄격):
    // 1. 위 그림자가 전체 범위의 70% 이상
    // 2. 아래 그림자가 전체 범위의 5% 이하
    // 3. 몸통이 전체 범위의 25% 이하
    upper_shadow_size >= total_range * 0.7 &&
    lower_shadow_size <= total_range * 0.05 &&
    body_size <= total_range * 0.25
}

/// 십자형 캔들인지 확인하는 함수
/// 
/// 십자형 캔들의 특징:
/// - 시가와 종가가 거의 같음
/// - 위아래 그림자가 있음
/// - 추세 전환 신호
pub fn is_doji(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 몸통이 전체 범위의 10% 이하
    body_size <= total_range * 0.1
}

/// 비석 십자형 캔들인지 확인하는 함수
/// 
/// 비석 십자형 캔들의 특징:
/// - 시가, 종가, 저가가 거의 같음
/// - 긴 위 그림자
/// - 상승추세 고점에서 하락 전환 신호
pub fn is_gravestone_doji(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 비석 십자형 조건:
    // 1. 몸통이 매우 작음 (5% 이하)
    // 2. 위 그림자가 전체 범위의 60% 이상
    // 3. 아래 그림자가 거의 없음 (5% 이하)
    body_size <= total_range * 0.05 &&
    upper_shadow_size >= total_range * 0.6 &&
    lower_shadow_size <= total_range * 0.05
}

/// 잠자리형 캔들인지 확인하는 함수
/// 
/// 잠자리형 캔들의 특징:
/// - 시가, 종가, 고가가 거의 같음
/// - 긴 아래 그림자
/// - 하락추세에서 상승 반전 신호
pub fn is_dragonfly_doji(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 잠자리형 조건 (조정됨):
    // 1. 몸통이 매우 작음 (10% 이하)
    // 2. 아래 그림자가 전체 범위의 50% 이상
    // 3. 위 그림자가 거의 없음 (10% 이하)
    body_size <= total_range * 0.1 &&
    lower_shadow_size >= total_range * 0.5 &&
    upper_shadow_size <= total_range * 0.1
}

/// 점십자형 캔들인지 확인하는 함수
/// 
/// 점십자형 캔들의 특징:
/// - 시가, 고가, 저가, 종가가 모두 거의 같음
/// - 매우 작은 몸통과 그림자
/// - 예측 불가능한 패턴
pub fn is_four_price_doji(candle: &Candle) -> bool {
    let body_size = (candle.get_trade_price() - candle.get_opening_price()).abs();
    let upper_shadow_size = candle.get_high_price() - candle.get_trade_price().max(candle.get_opening_price());
    let lower_shadow_size = candle.get_opening_price().min(candle.get_trade_price()) - candle.get_low_price();
    let total_range = candle.get_high_price() - candle.get_low_price();
    
    // 점십자형 조건 (조정됨):
    // 1. 몸통이 매우 작음 (5% 이하)
    // 2. 위 그림자가 매우 작음 (5% 이하)
    // 3. 아래 그림자가 매우 작음 (5% 이하)
    body_size <= total_range * 0.05 &&
    upper_shadow_size <= total_range * 0.05 &&
    lower_shadow_size <= total_range * 0.05
}

/// 캔들 패턴을 판단하는 통합 함수
/// 
/// 주어진 캔들이 어떤 패턴에 해당하는지 판단하여 패턴 이름을 반환합니다.
pub fn identify_candle_pattern(candle: &Candle, previous_candles: &[Candle]) -> Option<&'static str> {
    if is_four_price_doji(candle) {
        Some("점십자형")
    } else if is_gravestone_doji(candle) {
        Some("비석 십자형")
    } else if is_dragonfly_doji(candle) {
        Some("잠자리형")
    } else if is_doji(candle) {
        Some("십자형")
    } else if is_hanging_man(candle) {
        Some("교수형")
    } else if is_shooting_star(candle) {
        Some("유성형")
    } else if is_hammer(candle) {
        Some("망치형")
    } else if is_inverted_hammer(candle) {
        Some("역망치형")
    } else if is_lower_shadow_candle(candle) {
        Some("아래 꼬리 캔들")
    } else if is_upper_shadow_candle(candle) {
        Some("윗 꼬리 캔들")
    } else if is_rising_shooting_star(candle) {
        Some("상승 샅바형")
    } else if is_falling_shooting_star(candle) {
        Some("하락 샅바형")
    } else if is_long_bullish_candle(candle, previous_candles) {
        Some("장대 양봉형")
    } else if is_long_bearish_candle(candle, previous_candles) {
        Some("장대 음봉형")
    } else {
        None
    }
}