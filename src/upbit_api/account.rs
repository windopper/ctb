use serde::Deserialize;

use crate::{utils::str_to_f64, upbit_api::utils::request_upbit_api};

#[derive(Deserialize, Debug)]
pub struct Account {
    pub currency: String,
    #[serde(deserialize_with = "str_to_f64")]
    pub balance: f64,
    #[serde(deserialize_with = "str_to_f64")]
    pub locked: f64,
    #[serde(deserialize_with = "str_to_f64")]
    pub avg_buy_price: f64,
    pub avg_buy_price_modified: bool,
    pub unit_currency: String,
}

pub async fn check_my_account() -> Result<Vec<Account>, Box<dyn std::error::Error>> {
    let body = request_upbit_api("/accounts", None).await;
    if let Some(body) = body {
        let accounts: Vec<Account> = serde_json::from_str(&body)?;
        Ok(accounts)
    } else {
        Err("Failed to get accounts".into())
    }
}