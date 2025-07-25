#[derive(Debug, Clone, PartialEq)]
pub struct SignalReason {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Signal {
    Buy {
        reason: String,
        initial_trailing_stop: f64,
        take_profit: f64,
        asset_pct: f64,
    },
    Sell(SignalReason),
    Hold,
    UpdateTrailingStop(f64),
}