mod command;
mod error;

pub mod msg;
#[cfg(feature = "testing")]
pub mod tests;

// Export interface for use in SDK modules
pub use abstract_adapter_utils::{coins_in_assets, cw_approve_msgs, Identify};
pub use command::OracleCommand;
pub use error::OracleError;

pub const ORACLE_ADAPTER_ID: &str = "abstract:oracle";
