// 거래 유형을 나타내는 열거형 (Enum)
pub enum TradeSide {
    Buy,
    Sell,
}

// 단일 거래 데이터를 담는 구조체
pub struct CvdTrade {
    pub volume: f64,
    pub side: TradeSide,
}

/// 누적 거래량 델타(CVD)를 계산합니다.
///
/// # Arguments
///
/// * `trades` - 거래 데이터 슬라이스
///
/// # Returns
///
/// * 누적 거래량 델타, 거래량 차이 비율
pub fn calculate_cvd(trades: &Vec<CvdTrade>) -> (f64, f64) {
    let mut cumulative_delta = 0.0;
    let mut buy_volume = 0.0;
    let mut sell_volume = 0.0;

    for trade in trades {
        let delta = match trade.side {
            TradeSide::Buy => trade.volume,
            TradeSide::Sell => -trade.volume,
        };
        cumulative_delta += delta;

        if let TradeSide::Buy = trade.side {
            buy_volume += trade.volume;
        } else if let TradeSide::Sell = trade.side {
            sell_volume += trade.volume;
        }
    }

    let total_volume = buy_volume + sell_volume;

    (cumulative_delta, (cumulative_delta / total_volume) * 100.0)
}