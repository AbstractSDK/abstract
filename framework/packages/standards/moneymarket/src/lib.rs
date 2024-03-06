mod command;
mod error;

pub mod ans_action;
pub mod msg;
pub mod query;
pub mod raw_action;
#[cfg(feature = "testing")]
pub mod tests;

// Export interface for use in SDK modules
pub use abstract_adapter_utils::{coins_in_assets, cw_approve_msgs, Identify};
pub use command::{Fee, FeeOnInput, MoneyMarketCommand, Return, Spread};
pub use error::MoneyMarketError;

pub const MONEYMARKET_ADAPTER_ID: &str = "abstract:moneymarket";
