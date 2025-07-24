use ctb::helper::trend::{analyze_trend_moving_average, Trend};

#[test]
pub fn test_analyze_trend_moving_average() {
    let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0];
    let trend = analyze_trend_moving_average(&prices, 3, 7);
    assert_eq!(trend, Some(Trend::Uptrend));
}

#[test]
pub fn test_analyze_trend_moving_average_complex() {
    // 상승 구간
    let prices_up = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 112.0];
    let trend_up = analyze_trend_moving_average(&prices_up, 3, 5);
    assert_eq!(trend_up, Some(Trend::Uptrend));

    // 하락 구간
    let prices_down = vec![120.0, 118.0, 116.0, 114.0, 112.0, 110.0, 108.0];
    let trend_down = analyze_trend_moving_average(&prices_down, 3, 5);
    assert_eq!(trend_down, Some(Trend::Downtrend));

    // 횡보 구간
    let prices_sideways = vec![100.0, 100.5, 99.5, 100.0, 100.2, 99.8, 100.0];
    let trend_sideways = analyze_trend_moving_average(&prices_sideways, 3, 5);
    assert_eq!(trend_sideways, Some(Trend::Sideways));

    // 복합 구간 (상승 후 하락)
    let prices_mixed = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 108.0, 106.0, 104.0, 102.0, 100.0];
    let trend_mixed = analyze_trend_moving_average(&prices_mixed, 3, 7);
    // 마지막 구간이 하락이므로 Downtrend가 나와야 함
    assert_eq!(trend_mixed, Some(Trend::Downtrend));
}

