mod command;
pub mod error;
pub mod msg;

pub use abstract_adapter_utils::{coins_in_assets, cw_approve_msgs, Identify};
pub use command::CwStakingCommand;
pub use error::CwStakingError;

pub const CW_STAKING_ADAPTER_ID: &str = "abstract:cw-staking";
