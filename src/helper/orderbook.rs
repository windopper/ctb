use crate::core::orderbook::Orderbook;

pub fn orderbook_helper(orderbook: &Orderbook) {
    // 
}

/// 상위 n개 주문 비율
/// 주문 비율 = 상위 n개 주문 누적 체결량 / 전체 주문 누적 체결량
/// 주문 비율이 0.05 이하면 매도벽 약세, 0.95 이상이면 매수벽 약세
pub fn top_n_orderbook_ratio(orderbook: &Orderbook, n: usize) -> f64 {
    let mut total_ask_size = 0.0;
    let mut total_bid_size = 0.0;

    for i in 0..n {
        total_ask_size += orderbook.orderbook_units[i].ask_size;
        total_bid_size += orderbook.orderbook_units[i].bid_size;
    }

    total_ask_size / (total_ask_size + total_bid_size)
}