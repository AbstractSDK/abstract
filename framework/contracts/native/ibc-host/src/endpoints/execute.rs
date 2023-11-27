use std::str::FromStr;

use crate::{
    contract::{HostResponse, HostResult},
    HostError,
};
use abstract_core::{
    ibc_host::state::{CHAIN_PROXIES, CONFIG, REVERSE_CHAIN_PROXIES},
    objects::chain_name::ChainName,
};
use abstract_sdk::{core::ibc_host::ExecuteMsg, feature_objects::VersionControlContract};
use cosmwasm_std::{DepsMut, Env, MessageInfo};

use super::packet::handle_host_action;

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    match msg {
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
        ExecuteMsg::RegisterChainProxy { chain, proxy } => {
            register_chain_proxy(deps, info, chain, proxy)
        }
        ExecuteMsg::RemoveChainProxy { chain } => remove_chain_proxy(deps, info, chain),
        ExecuteMsg::Execute {
            proxy_address,
            account_id,
            action,
        } => {
            // This endpoint retrieves the chain name from the executor of the message
            let client_chain: ChainName = REVERSE_CHAIN_PROXIES.load(deps.storage, &info.sender)?;

            handle_host_action(deps, env, client_chain, proxy_address, account_id, action)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(HostResponse::action("update_ownership"))
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

    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    if let Some(ans_host_address) = ans_host_address {
        // validate address format
        config.ans_host.address = deps.api.addr_validate(&ans_host_address)?;
    }

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control =
            VersionControlContract::new(deps.api.addr_validate(&version_control_address)?);
    }

    if let Some(account_factory_address) = account_factory_address {
        // validate address format
        config.account_factory = deps.api.addr_validate(&account_factory_address)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(HostResponse::action("update_config"))
}

/// Register the polytone proxy address for a given chain
/// The polytone proxy will send messages to this address when it needs to execute actions on a local account.
fn register_chain_proxy(
    deps: DepsMut,
    info: MessageInfo,
    chain: String,
    proxy: String,
) -> HostResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let chain = ChainName::from_str(&chain)?;

    // We validate the proxy address, because this is the Polytone counterpart on the local chain
    let proxy = deps.api.addr_validate(&proxy)?;
    // Can't register if it already exists
    if CHAIN_PROXIES.has(deps.storage, &chain) || REVERSE_CHAIN_PROXIES.has(deps.storage, &proxy) {
        return Err(HostError::ProxyAddressExists {});
    }

    CHAIN_PROXIES.save(deps.storage, &chain, &proxy)?;
    REVERSE_CHAIN_PROXIES.save(deps.storage, &proxy, &chain)?;
    Ok(HostResponse::action("register_chain_client"))
}

fn remove_chain_proxy(deps: DepsMut, info: MessageInfo, chain: String) -> HostResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let chain = ChainName::from_str(&chain)?;

    if let Some(proxy) = CHAIN_PROXIES.may_load(deps.storage, &chain)? {
        REVERSE_CHAIN_PROXIES.remove(deps.storage, &proxy);
    }

    CHAIN_PROXIES.remove(deps.storage, &chain);
    Ok(HostResponse::action("register_chain_client"))
}
