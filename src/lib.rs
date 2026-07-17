pub mod quote;
pub mod protocol;
pub mod tickers;
pub mod error;

pub const PING_INTERVAL_SECONDS: u8 = 2;
pub const PING_TIMEOUT_SECONDS: u8 = 5;
pub const GEN_QUOTE_RANGE_MIN: u16 = 100;
pub const GEN_QUOTE_RANGE_MAX: u16 = 500;