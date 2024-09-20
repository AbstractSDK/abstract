use abstract_sdk::std::ibc_host::InstantiateMsg;
use abstract_std::IBC_HOST;
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;

use crate::contract::{HostResponse, HostResult, CONTRACT_VERSION};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> HostResult {
    set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(HostResponse::action("instantiate"))
}
