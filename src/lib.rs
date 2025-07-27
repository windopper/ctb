use dotenv::dotenv;

pub mod upbit_api;
pub mod utils;
pub mod core;
pub mod strategy;
pub mod helper;
pub mod backtest;
pub mod webhook;

pub fn env_var(key: &str) -> String {
    dotenv().ok();
    dotenv::var(key).unwrap()
}

