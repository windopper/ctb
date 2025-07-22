use serde::{Deserialize, Serialize};

use crate::upbit_api::utils::request_upbit_api;

#[derive(Deserialize, Serialize, Debug)]
pub struct Market {
    market: String,
    korean_name: String,
    english_name: String,
    market_event: Option<MarketEvent>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MarketEvent {
    warning: bool,
    caution: MarketCaution,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct MarketCaution {
    PRICE_FLUCTUATIONS: bool,
    TRADING_VOLUME_SOARING: bool,
    DEPOSIT_AMOUNT_SOARING: bool,
    GLOBAL_PRICE_DIFFERENCES: bool,
    CONCENTRATION_OF_SMALL_ACCOUNTS: bool,
}

/// 시장 정보 조회
pub async fn get_market_info() -> Result<Vec<Market>, Box<dyn std::error::Error>> {
    let body = request_upbit_api("/market/all", None).await;
    if let Some(body) = body {
        // eprintln!("{}", body);
        let markets: Vec<Market> = serde_json::from_str(&body)?;
        Ok(markets)
    } else {
        Err("Failed to get market info".into())
    }
}