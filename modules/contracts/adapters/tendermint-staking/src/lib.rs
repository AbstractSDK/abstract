pub mod contract;
pub mod error;
pub mod msg;
mod staking;

pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";

pub use contract::interface::TMintStakingAdapter;

pub use msg::{TendermintStakingExecuteMsgFns, TendermintStakingQueryMsgFns};
