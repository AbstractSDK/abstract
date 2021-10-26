use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Env, Timestamp};

pub(crate) const LP_TOKEN: &str = "lp_token";
pub(crate) const VAULT_CONTRACT: &str = "vault_contract_address";
pub(crate) const TEST_CREATOR: &str = "creator";

/**
 * Mocks the environment with a given height and time.
 */
pub fn mock_env_height(height: u64, time: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    env
}
