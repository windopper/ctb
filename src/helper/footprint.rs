use std::collections::BTreeMap;

use crate::core::trade::AskBid;

pub struct FootprintTrade {
    pub ask_bid: AskBid,
    pub price: f64,
    pub volume: f64,
}

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