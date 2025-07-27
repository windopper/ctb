use std::collections::BTreeMap;

use colored::Colorize;

use crate::core::trade::AskBid;

pub struct FootprintTrade {
    pub ask_bid: AskBid,
    pub price: f64,
    pub volume: f64,
}

#[derive(Clone)]
pub struct FootprintValue {
    pub ask_volume: f64,
    pub bid_volume: f64,
}

pub fn footprint(footprint_trades: &Vec<FootprintTrade>) -> BTreeMap<String, FootprintValue> {
    let mut footprints: BTreeMap<String, FootprintValue> = BTreeMap::new();

    for trade in footprint_trades {
        if trade.ask_bid == AskBid::Ask {
            footprints.entry(trade.price.to_string())
            .or_insert(FootprintValue { ask_volume: 0.0, bid_volume: 0.0 }).ask_volume += trade.volume;
        } else {
            footprints.entry(trade.price.to_string())
            .or_insert(FootprintValue { ask_volume: 0.0, bid_volume: 0.0 }).bid_volume += trade.volume;
        }
    }

    footprints
}

pub fn log_footprint(footprint: Vec<(String, FootprintValue)>) {
    let max_ask_len = footprint.iter().map(|(_, f)| format!("{:.6}", f.ask_volume).len()).max().unwrap_or(0);
    let max_bid_len = footprint.iter().map(|(_, f)| format!("{:.6}", f.bid_volume).len()).max().unwrap_or(0);

    for (key, value) in footprint.iter() {
        let ask_vol = value.ask_volume;
        let bid_vol = value.bid_volume;
        let diff = bid_vol - ask_vol; // 매수 - 매도 차이
        let diff_sign = if diff >= 0.0 { "+" } else { "" };
        let total_vol = ask_vol + bid_vol;
        let diff_pct = if total_vol.abs() > 1e-8 {
            (diff / total_vol) * 100.0
        } else {
            0.0
        };
        let diff_str = format!("{}{:.*}", diff_sign, 6, diff);
        let diff_pct_str = format!("({:+.2}%)", diff_pct);
        let colored_diff = if diff >= 0.0 {
            diff_str.green()
        } else {
            diff_str.red()
        };
        let colored_pct = if diff >= 0.0 {
            diff_pct_str.green()
        } else {
            diff_pct_str.red()
        };
        println!("{:<12}: {:<width$} | {:<width$} | {} {}",
            key,
            format!("{:.6}", ask_vol),
            format!("{:.6}", bid_vol),
            colored_diff,
            colored_pct,
            width = max_ask_len.max(max_bid_len)
        );
    }
}