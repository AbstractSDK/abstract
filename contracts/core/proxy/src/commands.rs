use abstract_os::IBC_CLIENT;
use cosmwasm_std::{
    wasm_execute, CosmosMsg, DepsMut, Empty, MessageInfo, Order, Response, StdError,
};

use crate::contract::ProxyResult;
use crate::error::ProxyError;
use crate::queries::*;
use abstract_os::ibc_client::ExecuteMsg as IbcClientMsg;
use abstract_os::objects::proxy_asset::UncheckedProxyAsset;
use abstract_os::proxy::state::{ADMIN, MEMORY, STATE, VAULT_ASSETS};

const LIST_SIZE_LIMIT: usize = 15;

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
pub fn execute_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    Ok(Response::new().add_messages(msgs))
}

/// Executes IBC actions forwarded by whitelisted contracts
/// Calls the messages on the IBC client (ensuring permission)
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<IbcClientMsg>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotWhitelisted {});
    }
    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ibc_client_address = abstract_os::manager::state::OS_MODULES
        .query(&deps.querier, manager_address, IBC_CLIENT)?
        .ok_or_else(|| StdError::GenericErr {
            msg: format!(
                "ibc_client not found on manager. Add it under the {} name.",
                IBC_CLIENT
            ),
        })?;
    let client_msgs: Result<Vec<_>, _> = msgs
        .into_iter()
        .map(|execute_msg| wasm_execute(&ibc_client_address, &execute_msg, vec![]))
        .collect();
    Ok(Response::new().add_messages(client_msgs?))
}

/// Update the stored vault asset information
pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<UncheckedProxyAsset>,
    to_remove: Vec<String>,
) -> ProxyResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let memory = &MEMORY.load(deps.storage)?;
    // Check the vault size to be within the size limit to prevent running out of gas when doing lookups
    let current_vault_size = VAULT_ASSETS
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let delta: i128 = to_add.len() as i128 - to_remove.len() as i128;
    if current_vault_size as i128 + delta > LIST_SIZE_LIMIT as i128 {
        return Err(ProxyError::AssetsLimitReached {});
    }

    for new_asset in to_add.into_iter() {
        let checked_asset = new_asset.check(deps.as_ref(), memory)?;

        VAULT_ASSETS.save(deps.storage, checked_asset.asset.clone(), &checked_asset)?;
    }

    for asset_id in to_remove {
        VAULT_ASSETS.remove(deps.storage, asset_id.into());
    }

    // Check validity of new configuration
    let validity_result = query_proxy_asset_validity(deps.as_ref())?;
    if validity_result.missing_dependencies.is_some()
        || validity_result.unresolvable_assets.is_some()
    {
        return Err(ProxyError::BadUpdate(format!("{:?}", validity_result)));
    }

    Ok(Response::new().add_attribute("action", "update_proxy_assets"))
}

/// Add a contract to the whitelist
pub fn add_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if state.modules.contains(&deps.api.addr_validate(&module)?) {
        return Err(ProxyError::AlreadyInList {});
    }

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if state.modules.len() >= LIST_SIZE_LIMIT {
        return Err(ProxyError::ModuleLimitReached {});
    }

    // Add contract to whitelist.
    state.modules.push(deps.api.addr_validate(&module)?);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", module))
}

/// Remove a contract from the whitelist
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if !state.modules.contains(&deps.api.addr_validate(&module)?) {
        return Err(ProxyError::NotInList {});
    }

    // Remove contract from whitelist.
    let module_address = deps.api.addr_validate(&module)?;
    state.modules.retain(|addr| *addr != module_address);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", module))
}
