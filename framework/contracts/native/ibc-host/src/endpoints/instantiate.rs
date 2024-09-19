use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    std::ibc_host::InstantiateMsg,
};
use abstract_std::{
    ibc_host::state::{Config, CONFIG},
    IBC_HOST,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw2::set_contract_version;

use crate::contract::{HostResponse, HostResult, CONTRACT_VERSION};

pub fn instantiate(deps: DepsMut, _env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    let ans_host = AnsHost {
        address: deps.api.addr_validate(&msg.ans_host_address)?,
    };
    let config = Config {
        version_control: VersionControlContract::new(
            deps.api.addr_validate(&msg.version_control_address)?,
        ),
        ans_host,
        module_factory_addr: deps.api.addr_validate(&msg.module_factory_address)?,
    };

    set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(HostResponse::action("instantiate"))
}
