use abstract_macros::abstract_response;
use abstract_sdk::std::ibc_host::{InstantiateMsg, QueryMsg};
use abstract_std::{
    ibc_host::{ExecuteMsg, MigrateMsg},
    objects::module_version::{assert_cw_contract_upgrade, migrate_module_data},
    IBC_HOST,
};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, IbcReceiveResponse, MessageInfo, Reply, Response, StdError,
};
use semver::Version;

use crate::{
    endpoints::{
        self,
        reply::{
            reply_execute_action, reply_forward_response_data, INIT_BEFORE_ACTION_REPLY_ID,
            RESPONSE_REPLY_ID,
        },
    },
    error::HostError,
};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(IBC_HOST)]
pub struct HostResponse;

pub type HostResult<T = Response> = Result<T, HostError>;
pub type IbcHostResult = HostResult<IbcReceiveResponse>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    endpoints::instantiate(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    // will only process base requests as there is no exec handler set.
    endpoints::execute(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> HostResult<Binary> {
    // will only process base requests as there is no exec handler set.
    endpoints::query(deps, env, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply_msg: Reply) -> HostResult {
    if reply_msg.id == INIT_BEFORE_ACTION_REPLY_ID {
        reply_execute_action(deps, env, reply_msg)
    } else if reply_msg.id == RESPONSE_REPLY_ID {
        reply_forward_response_data(reply_msg)
    } else {
        Err(HostError::Std(StdError::generic_err("Not implemented")))
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> HostResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            abstract_sdk::cw_helpers::migrate_instantiate(deps, env, instantiate_msg, instantiate)
        }
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_cw_contract_upgrade(deps.storage, IBC_HOST, to_version)?;
            cw2::set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;
            migrate_module_data(deps.storage, IBC_HOST, CONTRACT_VERSION, None::<String>)?;
            Ok(HostResponse::action("migrate"))
        }
    }
}
