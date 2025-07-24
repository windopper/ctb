use ctb::helper::supertrend::{calculate_supertrend, Ohlcv};

#[test]
fn test_supertrend() {
    let sample_data = vec![
        Ohlcv { open: 44.98, high: 45.02, low: 44.95, close: 45.00, volume: 1000.0 },
        Ohlcv { open: 45.00, high: 45.10, low: 44.98, close: 45.08, volume: 1200.0 },
        Ohlcv { open: 45.08, high: 45.15, low: 45.05, close: 45.12, volume: 1100.0 },
        Ohlcv { open: 45.12, high: 45.20, low: 45.09, close: 45.18, volume: 1500.0 },
        Ohlcv { open: 45.18, high: 45.25, low: 45.15, close: 45.22, volume: 1300.0 },
        Ohlcv { open: 45.22, high: 45.28, low: 45.19, close: 45.25, volume: 1400.0 },
        Ohlcv { open: 45.25, high: 45.30, low: 45.22, close: 45.28, volume: 1600.0 },
        Ohlcv { open: 45.28, high: 45.28, low: 45.15, close: 45.18, volume: 1800.0 },
        Ohlcv { open: 45.18, high: 45.20, low: 45.05, close: 45.08, volume: 2000.0 },
        Ohlcv { open: 45.08, high: 45.12, low: 44.90, close: 44.95, volume: 2200.0 },
        Ohlcv { open: 44.95, high: 45.00, low: 44.85, close: 44.90, volume: 2500.0 },  
    ];

    let period = 7;
    let multiplier = 3.0;

    let supertrend_values = calculate_supertrend(&sample_data, period, multiplier);

    for (i, result) in supertrend_values.iter().enumerate() {
        println!(
            "[{}] Close: {}, Supertrend: {:.4}, Uptrend: {}",
            i + period,
            sample_data[i + period].close,
            result.value,
            result.is_uptrend
        );
    }
}