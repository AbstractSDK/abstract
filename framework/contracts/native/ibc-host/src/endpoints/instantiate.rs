use crate::contract::{HostResponse, HostResult, CONTRACT_VERSION};
use abstract_core::{
    ibc_host::state::{Config, CONFIG},
    IBC_HOST,
};
use abstract_sdk::{
    core::ibc_host::InstantiateMsg,
    feature_objects::{AnsHost, VersionControlContract},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;

pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    let ans_host = AnsHost {
        address: deps.api.addr_validate(&msg.ans_host_address)?,
    };
    let config = Config {
        version_control: VersionControlContract::new(
            deps.api.addr_validate(&msg.version_control_address)?,
        ),
        ans_host,
        account_factory: deps.api.addr_validate(&msg.account_factory_address)?,
    };

    set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(HostResponse::action("instantiate"))
}
