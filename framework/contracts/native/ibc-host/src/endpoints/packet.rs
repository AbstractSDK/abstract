use abstract_core::{
    ibc::ModuleIbcMsg,
    ibc_client::InstalledModuleIdentification,
    ibc_host::{
        state::{ActionAfterCreationCache, CONFIG, TEMP_ACTION_AFTER_CREATION},
        HelperAction,
    },
    objects::{
        chain_name::ChainName, module::ModuleInfo, module_reference::ModuleReference, AccountId,
    },
};
use abstract_sdk::core::ibc_host::{HostAction, InternalAction};
use cosmwasm_std::{wasm_execute, Binary, DepsMut, Empty, Env, Response};

use crate::{
    account_commands::{self, receive_dispatch, receive_register, receive_send_all_back},
    contract::HostResult,
    HostError,
};

use abstract_core::base::ExecuteMsg as MiddlewareExecMsg;

pub fn remote_account_id(remote_chain: ChainName, account_id: AccountId) -> HostResult<AccountId> {
    let mut account_id = account_id.clone();
    account_id.trace_mut().push_chain(remote_chain.clone());

    Ok(account_id)
}

/// Handle actions that are passed to the IBC host contract
/// This function is not permissioned and access control needs to be handled outside of it
/// Usually the `client_chain` argument needs to be derived from the message sender
pub fn handle_host_action(
    deps: DepsMut,
    env: Env,
    client_chain: ChainName,
    proxy_address: String,
    received_account_id: AccountId,
    host_action: HostAction,
) -> HostResult {
    // Push the client chain to the account trace
    let account_id = remote_account_id(client_chain.clone(), received_account_id.clone())?;

    // get the local account information
    match host_action {
        HostAction::Internal(InternalAction::Register {
            description,
            link,
            name,
            base_asset,
            namespace,
            install_modules,
        }) => receive_register(
            deps,
            env,
            account_id,
            name,
            description,
            link,
            base_asset,
            namespace,
            install_modules,
            false,
        ),

        action => {
            // If this account already exists, we can propagate the action
            if let Ok(account) = account_commands::get_account(deps.as_ref(), &account_id) {
                match action {
                    HostAction::Dispatch { manager_msg } => {
                        receive_dispatch(deps, account, manager_msg)
                    }
                    HostAction::Helpers(helper_action) => match helper_action {
                        HelperAction::SendAllBack => {
                            receive_send_all_back(deps, env, account, proxy_address, client_chain)
                        }
                        _ => unimplemented!(""),
                    },
                    HostAction::Internal(InternalAction::Register { .. }) => {
                        unreachable!("This action is handled above")
                    }
                    _ => unimplemented!(""),
                }
            } else {
                // If no account is created already, we create one and execute the action on reply
                // The account metadata are not set with this call
                // One will have to change them at a later point if they decide to
                let name = format!(
                    "Remote Abstract Account for {}/{}",
                    client_chain.as_str(),
                    account_id
                );

                // We save the action they wanted to dispatch for the reply triggered by the receive_register function
                TEMP_ACTION_AFTER_CREATION.save(
                    deps.storage,
                    &ActionAfterCreationCache {
                        action,
                        client_proxy_address: proxy_address,
                        account_id: received_account_id,
                        chain_name: client_chain,
                    },
                )?;
                receive_register(
                    deps,
                    env,
                    account_id,
                    name,
                    None,
                    None,
                    None,
                    None,
                    vec![],
                    true,
                )
            }
        }
    }
    .map_err(Into::into)
}

/// Handle actions that are passed to the IBC host contract and originate from a registered module
pub fn handle_host_module_action(
    deps: DepsMut,
    client_chain: ChainName,
    source_module: InstalledModuleIdentification,
    target_module: ModuleInfo,
    msg: Binary,
) -> HostResult {
    // We resolve the target module
    let vc = CONFIG.load(deps.storage)?.version_control;
    let target_module_resolved = vc.query_module(target_module.clone(), &deps.querier)?;

    // We can't send module actions to accounts base and native apps
    let target_addr = match target_module_resolved.reference {
        ModuleReference::AccountBase(_) | ModuleReference::Native(_) => {
            return Err(HostError::WrongModuleAction(target_module.to_string()))
        }
        ModuleReference::Adapter(addr) => addr,
        ModuleReference::App(_) | ModuleReference::Standalone(_) => {
            let installed_module = InstalledModuleIdentification {
                module_info: target_module,
                account_id: source_module
                    .account_id
                    .clone()
                    .map(|a| remote_account_id(client_chain.clone(), a))
                    .transpose()?,
            };
            installed_module.addr(deps.as_ref(), vc)?
        }
        _ => unimplemented!(),
    };

    // We pass the message on to the module
    let msg = wasm_execute(
        target_addr,
        &MiddlewareExecMsg::ModuleIbc::<Empty, Empty>(ModuleIbcMsg {
            client_chain,
            source_module,
            msg,
        }),
        vec![],
    )?;

    Ok(Response::new()
        .add_attribute("action", "module-ibc-call")
        .add_message(msg))
}
