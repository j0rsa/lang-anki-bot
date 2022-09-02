use std::time::UNIX_EPOCH;

pub(crate) mod errors;
pub mod upstream;
pub mod downstream;
pub(crate) mod db_config;
pub(crate) mod repository;
pub mod models;

pub(crate) fn curr_millis() -> u128 {
    let now = std::time::SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis()
}