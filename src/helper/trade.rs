use crate::core::trade::{AskBid, Trade};

pub struct TradeState {
    pub latest_n_volume_diff: f64,
    pub latest_n_volume_total: f64,
    pub trades: Vec<Trade>,
}

pub fn latest_n_ask_bid_volume_ratio(trade: &Trade, state: &mut TradeState) -> f64 {
    state.trades.push(trade.clone());
    if state.trades.len() > 40 {
        let removed = state.trades.remove(0);
        if removed.ask_bid == AskBid::Ask {
            state.latest_n_volume_diff += removed.trade_volume;
            state.latest_n_volume_total -= removed.trade_volume;
        } else {
            state.latest_n_volume_diff -= removed.trade_volume;
            state.latest_n_volume_total -= removed.trade_volume;
        }
    }

    if trade.ask_bid == AskBid::Ask {
        state.latest_n_volume_diff -= trade.trade_volume;
        state.latest_n_volume_total += trade.trade_volume;
    } else {
        state.latest_n_volume_diff += trade.trade_volume;
        state.latest_n_volume_total += trade.trade_volume;
    }

    state.latest_n_volume_diff / state.latest_n_volume_total
}