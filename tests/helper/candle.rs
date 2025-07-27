use ctb::core::candle::{Candle, CandleBase};
use ctb::helper::candle::*;

fn create_candle(opening: f64, high: f64, low: f64, trade: f64) -> Candle {
    Candle {
        base: CandleBase {
            market: "KRW-BTC".to_string(),
            candle_date_time_utc: "2024-01-01T00:00:00".to_string(),
            candle_date_time_kst: "2024-01-01T09:00:00".to_string(),
            opening_price: opening,
            high_price: high,
            low_price: low,
            trade_price: trade,
            timestamp: 0,
            candle_acc_trade_price: 1000000.0,
            candle_acc_trade_volume: 1000.0,
        }
    }
}

#[test]
fn test_hammer_pattern() {
    // 망치형 캔들 테스트
    let hammer_candle = create_candle(100.0, 105.0, 95.0, 102.0);
    assert!(is_hammer(&hammer_candle));
    
    // 망치형이 아닌 캔들 테스트
    let normal_candle = create_candle(100.0, 110.0, 90.0, 108.0);
    assert!(!is_hammer(&normal_candle));
}

#[test]
fn test_inverted_hammer_pattern() {
    // 역망치형 캔들 테스트
    let inverted_hammer_candle = create_candle(100.0, 115.0, 98.0, 102.0);
    assert!(is_inverted_hammer(&inverted_hammer_candle));
}

#[test]
fn test_long_bullish_candle() {
    // 장대 양봉형 캔들 테스트
    let long_bullish_candle = create_candle(100.0, 120.0, 98.0, 118.0);
    
    let previous_candles = vec![
        create_candle(100.0, 105.0, 98.0, 103.0),
        create_candle(103.0, 107.0, 100.0, 105.0),
        create_candle(105.0, 108.0, 102.0, 106.0),
    ];
    
    assert!(is_long_bullish_candle(&long_bullish_candle, &previous_candles));
}

#[test]
fn test_long_bearish_candle() {
    // 장대 음봉형 캔들 테스트
    let long_bearish_candle = create_candle(120.0, 122.0, 100.0, 102.0);
    
    let previous_candles = vec![
        create_candle(100.0, 105.0, 98.0, 103.0),
        create_candle(103.0, 107.0, 100.0, 105.0),
        create_candle(105.0, 108.0, 102.0, 106.0),
    ];
    
    assert!(is_long_bearish_candle(&long_bearish_candle, &previous_candles));
}

#[test]
fn test_doji_patterns() {
    // 일반 십자형 캔들 테스트
    let doji_candle = create_candle(100.0, 105.0, 95.0, 100.5);
    assert!(is_doji(&doji_candle));
    
    // 비석 십자형 캔들 테스트
    let gravestone_doji_candle = create_candle(100.0, 115.0, 100.0, 100.0);
    assert!(is_gravestone_doji(&gravestone_doji_candle));
    
    // 잠자리형 캔들 테스트
    let dragonfly_doji_candle = create_candle(100.0, 100.0, 85.0, 100.0);
    assert!(is_dragonfly_doji(&dragonfly_doji_candle));
    
    // 점십자형 캔들 테스트
    let four_price_doji_candle = create_candle(100.0, 100.1, 99.9, 100.0);
    assert!(is_four_price_doji(&four_price_doji_candle));
}

#[test]
fn test_shooting_star_patterns() {
    // 유성형 캔들 테스트
    let shooting_star_candle = create_candle(100.0, 115.0, 98.0, 102.0);
    assert!(is_shooting_star(&shooting_star_candle));
    
    // 상승 샅바형 캔들 테스트
    let rising_shooting_star_candle = create_candle(100.0, 110.0, 100.0, 105.0);
    assert!(is_rising_shooting_star(&rising_shooting_star_candle));
    
    // 하락 샅바형 캔들 테스트
    let falling_shooting_star_candle = create_candle(110.0, 115.0, 100.0, 100.0);
    assert!(is_falling_shooting_star(&falling_shooting_star_candle));
}

#[test]
fn test_hanging_man_pattern() {
    // 교수형 캔들 테스트
    let hanging_man_candle = create_candle(100.0, 101.0, 85.0, 99.0);
    assert!(is_hanging_man(&hanging_man_candle));
}

#[test]
fn test_identify_candle_pattern() {
    let previous_candles = vec![
        create_candle(100.0, 105.0, 98.0, 103.0),
        create_candle(103.0, 107.0, 100.0, 105.0),
    ];
    
    // 망치형 테스트
    let hammer_candle = create_candle(100.0, 105.0, 95.0, 102.0);
    assert_eq!(identify_candle_pattern(&hammer_candle, &previous_candles), Some("망치형"));
    
    // 장대 양봉형 테스트
    let long_bullish_candle = create_candle(100.0, 120.0, 98.0, 118.0);
    assert_eq!(identify_candle_pattern(&long_bullish_candle, &previous_candles), Some("장대 양봉형"));
    
    // 십자형 테스트
    let doji_candle = create_candle(100.0, 105.0, 95.0, 100.5);
    assert_eq!(identify_candle_pattern(&doji_candle, &previous_candles), Some("십자형"));
    
    // 패턴이 없는 일반 캔들 테스트
    let normal_candle = create_candle(100.0, 110.0, 90.0, 108.0);
    assert_eq!(identify_candle_pattern(&normal_candle, &previous_candles), None);
} 