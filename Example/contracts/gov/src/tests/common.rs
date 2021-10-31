use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Env, Timestamp};

pub(crate) const VOTING_TOKEN: &str = "voting_token";
pub(crate) const TEST_CREATOR: &str = "creator";
pub(crate) const TEST_VOTER: &str = "voter1";
pub(crate) const TEST_VOTER_2: &str = "voter2";
pub(crate) const TEST_VOTER_3: &str = "voter3";
pub(crate) const DEFAULT_QUORUM: u64 = 30u64;
pub(crate) const DEFAULT_THRESHOLD: u64 = 50u64;
pub(crate) const DEFAULT_VOTING_PERIOD: u64 = 10000u64;
pub(crate) const DEFAULT_FIX_PERIOD: u64 = 10u64;
pub(crate) const DEFAULT_TIMELOCK_PERIOD: u64 = 10000u64;
pub(crate) const DEFAULT_EXPIRATION_PERIOD: u64 = 20000u64;
pub(crate) const DEFAULT_PROPOSAL_DEPOSIT: u128 = 10000000000u128;

/**
 * Mocks the environment with a given height and time.
 */
pub fn mock_env_height(height: u64, time: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    env
}
