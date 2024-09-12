use abstract_std::{
    base::ExecuteMsg as MiddlewareExecMsg,
    ibc::{ModuleIbcInfo, ModuleIbcMsg},
    ibc_client::InstalledModuleIdentification,
    ibc_host::{
        state::{ActionAfterCreationCache, CONFIG, TEMP_ACTION_AFTER_CREATION},
        HelperAction, HostAction, InternalAction,
    },
    objects::{
        account::AccountTrace, module::ModuleInfo, module_reference::ModuleReference, AccountId,
        TruncatedChainId,
    },
};
use cosmwasm_std::{
    to_json_vec, wasm_execute, Binary, ContractResult, Deps, DepsMut, Empty, Env, QueryRequest,
    Response, StdError, SystemResult, WasmQuery,
};

use crate::{
    account_commands::{self, receive_dispatch, receive_register, receive_send_all_back},
    contract::HostResult,
    HostError,
};

/// Handle actions that are passed to the IBC host contract
/// This function is not permissioned and access control needs to be handled outside of it
/// Usually the `src_chain` argument needs to be derived from the message sender
pub fn handle_host_action(
    deps: DepsMut,
    env: Env,
    src_chain: TruncatedChainId,
    proxy_address: String,
    received_account_id: AccountId,
    host_action: HostAction,
) -> HostResult {
    // Push the client chain to the account trace
    let account_id = {
        let mut account_id = received_account_id.clone();
        account_id.push_chain(src_chain.clone());
        account_id
    };

    // get the local account information
    match host_action {
        HostAction::Internal(InternalAction::Register {
            description,
            link,
            name,
            namespace,
            install_modules,
        }) => receive_register(
            deps,
            env,
            account_id,
            name,
            description,
            link,
            namespace,
            install_modules,
            false,
        ),

        action => {
            // If this account already exists, we can propagate the action
            if let Ok(account) = account_commands::get_account(deps.as_ref(), &account_id) {
                match action {
                    HostAction::Dispatch { account_msgs } => {
                        receive_dispatch(deps, account, account_msgs)
                    }
                    HostAction::Helpers(helper_action) => match helper_action {
                        HelperAction::SendAllBack => {
                            receive_send_all_back(deps, env, account, proxy_address, src_chain)
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
                    src_chain.as_str(),
                    account_id
                );

                // We save the action they wanted to dispatch for the reply triggered by the receive_register function
                TEMP_ACTION_AFTER_CREATION.save(
                    deps.storage,
                    &ActionAfterCreationCache {
                        action,
                        client_proxy_address: proxy_address,
                        account_id: received_account_id,
                        chain_name: src_chain,
                    },
                )?;
                receive_register(deps, env, account_id, name, None, None, None, vec![], true)
            }
        }
    }
    .map_err(Into::into)
}

/// Handle actions that are passed to the IBC host contract and originate from a registered module
pub fn handle_module_execute(
    deps: DepsMut,
    env: Env,
    src_chain: TruncatedChainId,
    source_module: InstalledModuleIdentification,
    target_module: ModuleInfo,
    msg: Binary,
) -> HostResult {
    // We resolve the target module
    let vc = CONFIG.load(deps.storage)?.version_control;

    let target_module = InstalledModuleIdentification {
        module_info: target_module,
        // Account can only call modules that are installed on its ICAA.
        // If the calling module is account-specific then we map the calling account-id to the host.
        account_id: source_module
            .account_id
            .map(|a| client_to_host_module_account_id(&env, src_chain.clone(), a)),
    };

    let target_module_resolved = target_module.addr(deps.as_ref(), vc)?;

    match target_module_resolved.reference {
        ModuleReference::Account(_) | ModuleReference::Native(_) | ModuleReference::Service(_) => {
            return Err(HostError::WrongModuleAction(
                "Can't send module-to-module message to an account, service or a native module"
                    .to_string(),
            ))
        }
        _ => {}
    }

    let response = Response::new().add_attribute("action", "module-ibc-call");
    // We pass the message on to the module
    let msg = wasm_execute(
        target_module_resolved.address,
        &MiddlewareExecMsg::ModuleIbc::<Empty, Empty>(ModuleIbcMsg {
            src_module_info: ModuleIbcInfo {
                chain: src_chain,
                module: source_module.module_info,
            },
            msg,
        }),
        vec![],
    )?;

    Ok(response.add_message(msg))
}

/// Handle actions that are passed to the IBC host contract and originate from a registered module
pub fn handle_host_module_query(
    deps: Deps,
    target_module: InstalledModuleIdentification,
    msg: Binary,
) -> HostResult<Binary> {
    // We resolve the target module
    let vc = CONFIG.load(deps.storage)?.version_control;

    let target_module_resolved = target_module.addr(deps, vc)?;

    let query = QueryRequest::<Empty>::from(WasmQuery::Smart {
        contract_addr: target_module_resolved.address.into_string(),
        msg,
    });
    let bin = match deps.querier.raw_query(&to_json_vec(&query)?) {
        SystemResult::Err(system_err) => Err(StdError::generic_err(format!(
            "Querier system error: {system_err}"
        ))),
        SystemResult::Ok(ContractResult::Err(contract_err)) => Err(StdError::generic_err(format!(
            "Querier contract error: {contract_err}"
        ))),
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }?;
    Ok(bin)
}

/// We need to figure what trace module is implying here
pub fn client_to_host_module_account_id(
    env: &Env,
    remote_chain: TruncatedChainId,
    mut account_id: AccountId,
) -> AccountId {
    let account_trace = account_id.trace_mut();
    match account_trace {
        AccountTrace::Local => account_trace.push_chain(remote_chain),
        AccountTrace::Remote(trace) => {
            let current_chain_name = TruncatedChainId::from_chain_id(&env.block.chain_id);
            // If current chain_name == last trace in account_id it means we got response back from remote chain
            if current_chain_name.eq(trace.last().unwrap()) {
                trace.pop();
                if trace.is_empty() {
                    *account_trace = AccountTrace::Local;
                }
            } else {
                trace.push(remote_chain);
            }
        }
    };
    account_id
}
