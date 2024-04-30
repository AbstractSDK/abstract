pub mod contract;
pub mod error;
pub mod msg;
mod staking;

pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";

#[cfg(feature = "interface")]
pub use contract::interface::TMintStakingAdapter;

#[cfg(feature = "interface")]
pub use msg::{TendermintStakingExecuteMsgFns, TendermintStakingQueryMsgFns};
