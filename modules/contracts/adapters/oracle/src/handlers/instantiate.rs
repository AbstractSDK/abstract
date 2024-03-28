use crate::{
    contract::{OracleAdapter, OracleResult},
    msg::OracleInstantiateMsg,
    state::{Oracle, ADDRESSES_OF_PROVIDERS},
};

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: OracleAdapter,
    msg: OracleInstantiateMsg,
) -> OracleResult {
    let OracleInstantiateMsg {
        external_age_max,
        providers,
    } = msg;

    // Save config
    let oracle = Oracle::new("");
    oracle.update_config(deps, external_age_max)?;

    // Save addresses of providers
    for (provider, human_addr) in providers {
        let addr = deps.api.addr_validate(&human_addr)?;
        ADDRESSES_OF_PROVIDERS.save(deps.storage, &provider, &addr)?;
    }
    Ok(Response::default())
}
