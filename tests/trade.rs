use ctb::core::trade::{filter_trades_by_same_minute, is_trade_time_previous_minute, AskBid, Change, StreamType, Trade};

fn get_default_trade(trade_date: &str, trade_time: &str) -> Trade {
    Trade {
        trade_date: trade_date.to_string(),
        trade_time: trade_time.to_string(),
        trade_type: "".to_string(),
        code: "".to_string(),
        trade_price: 0.0,
        trade_volume: 0.0,
        ask_bid: AskBid::Ask,
        prev_closing_price: 0.0,
        change: Change::Even,
        change_price: 0.0,
        trade_timestamp: 0,
        timestamp: 0,
        sequential_id: 0,
        best_ask_price: 0.0,
        best_ask_size: 0.0,
        best_bid_price: 0.0,
        best_bid_size: 0.0,
        stream_type: StreamType::Snapshot,
    }
}

#[test]
pub fn test_filter_trades_by_previous_minute() {
    let trades = vec![
        get_default_trade("2024-01-01", "12:00:00"),
        get_default_trade("2024-01-01", "12:01:00"),
        get_default_trade("2024-01-01", "12:02:00"),
    ];

    let filtered_trades = filter_trades_by_same_minute(&trades, "2024-01-01T12:01:00");
    let filtered_trades_2 = filter_trades_by_same_minute(&trades, "2024-01-01T21:01:00");
    let filtered_trades_3 = filter_trades_by_same_minute(&trades, "2024-01-01T21:02:00");

    assert_eq!(filtered_trades.len(), 0);
    assert_eq!(filtered_trades_2.len(), 1);
    assert_eq!(filtered_trades_3.len(), 1);
}

#[test]
pub fn test_is_trade_time_previous_minute() {
    let trades = vec![
        get_default_trade("2024-01-01", "12:00:00"),
        get_default_trade("2024-01-01", "12:01:00"),
        get_default_trade("2024-01-01", "12:02:00"),
    ];
    let is_previous_minute = is_trade_time_previous_minute(&trades[0], "2024-01-01T21:01:00");
    assert_eq!(is_previous_minute, true);
    let is_previous_minute = is_trade_time_previous_minute(&trades[1], "2024-01-01T21:01:00");
    assert_eq!(is_previous_minute, false);
    let is_previous_minute = is_trade_time_previous_minute(&trades[2], "2024-01-01T21:01:00");
    assert_eq!(is_previous_minute, false);
}