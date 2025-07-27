use ctb::core::candle::{Candle, CandleBase, CandleTrait};
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

fn main() {
    println!("=== 캔들 패턴 분석 예제 ===\n");
    
    // 이전 캔들들 (비교용)
    let previous_candles = vec![
        create_candle(100.0, 105.0, 98.0, 103.0),
        create_candle(103.0, 107.0, 100.0, 105.0),
        create_candle(105.0, 108.0, 102.0, 106.0),
    ];
    
    // 1. 망치형 캔들 테스트
    let hammer_candle = create_candle(100.0, 105.0, 95.0, 102.0);
    println!("1. 망치형 캔들:");
    println!("   시가: 100.0, 고가: 105.0, 저가: 95.0, 종가: 102.0");
    println!("   망치형 여부: {}", is_hammer(&hammer_candle));
    println!();
    
    // 2. 역망치형 캔들 테스트
    let inverted_hammer_candle = create_candle(100.0, 115.0, 98.0, 102.0);
    println!("2. 역망치형 캔들:");
    println!("   시가: 100.0, 고가: 115.0, 저가: 98.0, 종가: 102.0");
    println!("   역망치형 여부: {}", is_inverted_hammer(&inverted_hammer_candle));
    println!();
    
    // 3. 장대 양봉형 캔들 테스트
    let long_bullish_candle = create_candle(100.0, 120.0, 98.0, 118.0);
    println!("3. 장대 양봉형 캔들:");
    println!("   시가: 100.0, 고가: 120.0, 저가: 98.0, 종가: 118.0");
    println!("   장대 양봉형 여부: {}", is_long_bullish_candle(&long_bullish_candle, &previous_candles));
    println!();
    
    // 4. 장대 음봉형 캔들 테스트
    let long_bearish_candle = create_candle(120.0, 122.0, 100.0, 102.0);
    println!("4. 장대 음봉형 캔들:");
    println!("   시가: 120.0, 고가: 122.0, 저가: 100.0, 종가: 102.0");
    println!("   장대 음봉형 여부: {}", is_long_bearish_candle(&long_bearish_candle, &previous_candles));
    println!();
    
    // 5. 십자형 캔들 테스트
    let doji_candle = create_candle(100.0, 105.0, 95.0, 100.5);
    println!("5. 십자형 캔들:");
    println!("   시가: 100.0, 고가: 105.0, 저가: 95.0, 종가: 100.5");
    println!("   십자형 여부: {}", is_doji(&doji_candle));
    println!();
    
    // 6. 비석 십자형 캔들 테스트
    let gravestone_doji_candle = create_candle(100.0, 115.0, 100.0, 100.0);
    println!("6. 비석 십자형 캔들:");
    println!("   시가: 100.0, 고가: 115.0, 저가: 100.0, 종가: 100.0");
    println!("   비석 십자형 여부: {}", is_gravestone_doji(&gravestone_doji_candle));
    println!();
    
    // 7. 잠자리형 캔들 테스트
    let dragonfly_doji_candle = create_candle(100.0, 100.0, 85.0, 100.0);
    println!("7. 잠자리형 캔들:");
    println!("   시가: 100.0, 고가: 100.0, 저가: 85.0, 종가: 100.0");
    println!("   잠자리형 여부: {}", is_dragonfly_doji(&dragonfly_doji_candle));
    println!();
    
    // 8. 점십자형 캔들 테스트
    let four_price_doji_candle = create_candle(100.0, 100.1, 99.9, 100.0);
    println!("8. 점십자형 캔들:");
    println!("   시가: 100.0, 고가: 100.1, 저가: 99.9, 종가: 100.0");
    println!("   점십자형 여부: {}", is_four_price_doji(&four_price_doji_candle));
    println!();
    
    // 9. 유성형 캔들 테스트
    let shooting_star_candle = create_candle(100.0, 115.0, 98.0, 102.0);
    println!("9. 유성형 캔들:");
    println!("   시가: 100.0, 고가: 115.0, 저가: 98.0, 종가: 102.0");
    println!("   유성형 여부: {}", is_shooting_star(&shooting_star_candle));
    println!();
    
    // 10. 교수형 캔들 테스트
    let hanging_man_candle = create_candle(100.0, 101.0, 85.0, 99.0);
    println!("10. 교수형 캔들:");
    println!("    시가: 100.0, 고가: 101.0, 저가: 85.0, 종가: 99.0");
    println!("    교수형 여부: {}", is_hanging_man(&hanging_man_candle));
    println!();
    
    // 11. 상승 샅바형 캔들 테스트
    let rising_shooting_star_candle = create_candle(100.0, 110.0, 100.0, 105.0);
    println!("11. 상승 샅바형 캔들:");
    println!("    시가: 100.0, 고가: 110.0, 저가: 100.0, 종가: 105.0");
    println!("    상승 샅바형 여부: {}", is_rising_shooting_star(&rising_shooting_star_candle));
    println!();
    
    // 12. 하락 샅바형 캔들 테스트
    let falling_shooting_star_candle = create_candle(110.0, 115.0, 100.0, 100.0);
    println!("12. 하락 샅바형 캔들:");
    println!("    시가: 110.0, 고가: 115.0, 저가: 100.0, 종가: 100.0");
    println!("    하락 샅바형 여부: {}", is_falling_shooting_star(&falling_shooting_star_candle));
    println!();
    
    // 통합 패턴 판단 테스트
    println!("=== 통합 패턴 판단 테스트 ===");
    
    let test_candles = vec![
        ("망치형", create_candle(100.0, 105.0, 95.0, 102.0)),
        ("장대 양봉형", create_candle(100.0, 120.0, 98.0, 118.0)),
        ("십자형", create_candle(100.0, 105.0, 95.0, 100.5)),
        ("일반 캔들", create_candle(100.0, 110.0, 90.0, 108.0)),
    ];
    
    for (name, candle) in test_candles {
        let pattern = identify_candle_pattern(&candle, &previous_candles);
        println!("{}: {:?}", name, pattern);
    }
    
    println!("\n=== 구현된 캔들 패턴 목록 ===");
    println!("1. 망치형 (Hammer)");
    println!("2. 역망치형 (Inverted Hammer)");
    println!("3. 장대 양봉형 (Long Bullish Candle)");
    println!("4. 장대 음봉형 (Long Bearish Candle)");
    println!("5. 십자형 (Doji)");
    println!("6. 비석 십자형 (Gravestone Doji)");
    println!("7. 잠자리형 (Dragonfly Doji)");
    println!("8. 점십자형 (Four Price Doji)");
    println!("9. 유성형 (Shooting Star)");
    println!("10. 교수형 (Hanging Man)");
    println!("11. 상승 샅바형 (Rising Shooting Star)");
    println!("12. 하락 샅바형 (Falling Shooting Star)");
    println!("13. 아래 꼬리 캔들 (Lower Shadow Candle)");
    println!("14. 윗 꼬리 캔들 (Upper Shadow Candle)");
} 