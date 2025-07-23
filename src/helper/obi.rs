use crate::core::orderbook::Orderbook;

/// 호가창 불균형 (Order Book Imbalance, OBI) 계산
/// 지정된 깊이까지의 (매수 잔량 / 매도 잔량) 비율을 계산합니다.
pub fn calculate_obi(orderbook: &Option<Orderbook>, depth: usize) -> f64 {
    if let Some(ob) = orderbook {
        // 매수 잔량 합계
        let bid_sum: f64 = ob.orderbook_units.iter().take(depth).map(|unit| unit.bid_size).sum();

        // 매도 잔량 합계
        let ask_sum: f64 = ob.orderbook_units.iter().take(depth).map(|unit| unit.ask_size).sum();

        if ask_sum > 0.0 {
            bid_sum / ask_sum
        } else {
            f64::INFINITY // 매도 물량이 없으면 무한대로 표현
        }
    } else {
        1.0 // 호가창이 없으면 중립 상태로 간주
    }
}