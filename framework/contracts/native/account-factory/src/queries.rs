use crate::state::*;

use abstract_sdk::core::account_factory::*;
use cosmwasm_std::{Deps, StdResult};

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let _admin = cw_ownable::get_ownership(deps.storage)?;
    let resp = ConfigResponse {
        version_control_contract: state.version_control_contract,
        ans_host_contract: state.ans_host_contract,
        module_factory_address: state.module_factory_address,
        local_account_sequence: LOCAL_ACCOUNT_SEQUENCE.may_load(deps.storage)?.unwrap_or(0),
        ibc_host: state.ibc_host,
    };

    Ok(resp)
}
