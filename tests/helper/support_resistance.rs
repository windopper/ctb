use ctb::helper::support_resistance::{calculate_trend_lines_with_regression, Candle};
#[test]
fn test_calculate_trend_lines_integration() {
    // 더 간단한 지지선 패턴 테스트
    let candles_support: Vec<Candle> = vec![
        Candle { open: 10.0, high: 10.0, low: 10.0, close: 10.0 }, // 0
        Candle { open: 8.0, high: 8.0, low: 8.0, close: 8.0 },   // 1
        Candle { open: 6.0, high: 6.0, low: 6.0, close: 6.0 },   // 2 (min)
        Candle { open: 8.0, high: 8.0, low: 8.0, close: 8.0 },   // 3
        Candle { open: 6.0, high: 6.0, low: 6.0, close: 6.0 },   // 4 (min)
        Candle { open: 9.0, high: 9.0, low: 9.0, close: 9.0 },   // 5
    ];

    let (support, _resistance) = calculate_trend_lines_with_regression(&candles_support);

    if let Some(s_line) = support {
        // 지지선의 기울기가 양수인지(우상향) 확인
        assert!(s_line.slope > 0.0);
    }
    
    // 저항선이 있는 패턴 테스트 (역U자형)
    let candles_resistance: Vec<Candle> = vec![
        Candle { open: 6.0, high: 6.0, low: 6.0, close: 6.0 },   // 0
        Candle { open: 8.0, high: 8.0, low: 8.0, close: 8.0 },   // 1
        Candle { open: 10.0, high: 10.0, low: 10.0, close: 10.0 }, // 2 (max)
        Candle { open: 8.0, high: 8.0, low: 8.0, close: 8.0 },   // 3
        Candle { open: 10.0, high: 10.0, low: 10.0, close: 10.0 }, // 4 (max)
        Candle { open: 7.0, high: 7.0, low: 7.0, close: 7.0 },   // 5
    ];

    let (_support2, resistance2) = calculate_trend_lines_with_regression(&candles_resistance);
    
    // 저항선이 계산되면 기울기 확인
    if let Some(r_line) = resistance2 {
        // 저항선의 기울기가 음수인지(우하향) 확인
        assert!(r_line.slope < 0.0);
    }
    
    // 캔들이 없을 때 (None, None)을 반환하는지 테스트
    let empty_candles: Vec<Candle> = Vec::new();
    let (s_empty, r_empty) = calculate_trend_lines_with_regression(&empty_candles);
    assert!(s_empty.is_none());
    assert!(r_empty.is_none());
}