use crate::{contract::MyStandaloneResult, msg::MyStandaloneMigrateMsg, MY_STANDALONE};

use abstract_standalone::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env};

/// Handle the standalone migrate msg
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MyStandaloneMigrateMsg) -> MyStandaloneResult {
    // The Abstract Standalone object does version checking and
    MY_STANDALONE.migrate(deps)?;
    Ok(MY_STANDALONE.response("migrate"))
}
