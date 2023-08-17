use crate::{
    contract::{HostResponse, HostResult},
    state::{CHAIN_PROXYS, CONFIG, REVERSE_CHAIN_PROXYS},
    HostError,
};
use abstract_core::{objects::chain_name::ChainName, proxy::state::ADMIN};
use abstract_sdk::core::ibc_host::ExecuteMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use super::packet::handle_host_action;

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host_address,
            account_factory_address,
            version_control_address,
        } => update_config(
            deps,
            info,
            ans_host_address,
            version_control_address,
            account_factory_address,
        ),
        ExecuteMsg::RegisterChainProxy { chain_id, proxy } => {
            register_chain_proxy(deps, info, chain_id, proxy)
        }
        ExecuteMsg::RemoveChainProxy { chain_id } => remove_chain_proxy(deps, info, chain_id),
        ExecuteMsg::RecoverAccount {
            closed_channel: _,
            account_id: _,
            msgs: _,
        } => {
            cw_ownable::assert_owner(deps.storage, &info.sender).unwrap();
            // TODO:
            todo!()
        }
        ExecuteMsg::Execute { account_id, action } => {
            handle_host_action(deps, env, info, account_id, action)
        }
    }
}

/// Updates the host's configuration
fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    ans_host_address: Option<String>,
    version_control_address: Option<String>,
    account_factory_address: Option<String>,
) -> HostResult {
    let mut config = CONFIG.load(deps.storage)?;

    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(ans_host_address) = ans_host_address {
        // validate address format
        config.ans_host.address = deps.api.addr_validate(&ans_host_address)?;
    }

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control = deps.api.addr_validate(&version_control_address)?;
    }

    if let Some(account_factory_address) = account_factory_address {
        // validate address format
        config.account_factory = deps.api.addr_validate(&account_factory_address)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(HostResponse::action("update_config"))
}

fn register_chain_proxy(
    deps: DepsMut,
    info: MessageInfo,
    chain_id: ChainName,
    proxy: String,
) -> HostResult {
    cw_ownable::is_owner(deps.storage, &info.sender)?;

    // We validate the proxy address, because this is the Polytone counterpart on the local chain
    let proxy = deps.api.addr_validate(&proxy)?;
    // Can't register if it already exists
    if CHAIN_PROXYS.has(deps.storage, &chain_id) || REVERSE_CHAIN_PROXYS.has(deps.storage, &proxy) {
        return Err(HostError::ProxyAddressExists);
    }

    CHAIN_PROXYS.save(deps.storage, &chain_id, &proxy)?;
    REVERSE_CHAIN_PROXYS.save(deps.storage, &proxy, &chain_id)?;
    Ok(HostResponse::action("register_chain_client"))
}

fn remove_chain_proxy(deps: DepsMut, info: MessageInfo, chain_id: ChainName) -> HostResult {
    cw_ownable::is_owner(deps.storage, &info.sender)?;

    if let Some(proxy) = CHAIN_PROXYS.may_load(deps.storage, &chain_id)? {
        REVERSE_CHAIN_PROXYS.remove(deps.storage, &proxy);
    }

    CHAIN_PROXYS.remove(deps.storage, &chain_id);
    Ok(HostResponse::action("register_chain_client"))
}
