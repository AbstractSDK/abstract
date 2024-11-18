use abstract_sdk::std::ibc_host::ExecuteMsg;
use abstract_std::{
    ibc_host::state::{CHAIN_PROXIES, REVERSE_CHAIN_PROXIES},
    objects::TruncatedChainId,
};
use cosmwasm_std::{BankMsg, DepsMut, Env, MessageInfo, Response};

use super::packet::{handle_host_action, handle_module_execute};
use crate::{
    account_commands::{self, receive_register},
    contract::{HostResponse, HostResult},
    HostError,
};

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    match msg {
        ExecuteMsg::RegisterChainProxy { chain, proxy } => {
            register_chain_proxy(deps, info, chain, proxy)
        }
        ExecuteMsg::RemoveChainProxy { chain } => remove_chain_proxy(deps, info, chain),
        ExecuteMsg::Execute {
            account_address,
            account_id,
            action,
        } => {
            // This endpoint retrieves the chain name from the executor of the message
            let src_chain: TruncatedChainId =
                REVERSE_CHAIN_PROXIES.load(deps.storage, &info.sender)?;

            handle_host_action(deps, env, src_chain, account_address, account_id, action)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(HostResponse::action("update_ownership"))
        }
        ExecuteMsg::ModuleExecute {
            msg,
            source_module,
            target_module,
        } => {
            let src_chain: TruncatedChainId =
                REVERSE_CHAIN_PROXIES.load(deps.storage, &info.sender)?;

            handle_module_execute(deps, env, src_chain, source_module, target_module, msg)
        }
        ExecuteMsg::Fund {
            src_account,
            src_chain,
        } => {
            // Push the client chain to the account trace
            let account_id = {
                let mut account_id = src_account.clone();
                account_id.push_chain(src_chain.clone());
                account_id
            };
            if let Ok(account) = account_commands::get_account(deps.as_ref(), &env, &account_id) {
                // Send funds to the account

                Ok(Response::new().add_message(BankMsg::Send {
                    to_address: account.addr().to_string(),
                    amount: info.funds,
                }))
            } else {
                // If no account is created already, we create one and send the funds during instantiation directly
                // The account metadata are not set with this call
                // One will have to change them at a later point if they decide to
                let name = format!(
                    "Remote Abstract Account for {}/{}",
                    src_chain.as_str(),
                    account_id
                );

                receive_register(
                    deps,
                    env,
                    account_id,
                    Some(name),
                    None,
                    None,
                    None,
                    vec![],
                    false,
                    info.funds,
                )
            }
        }
    }
}

/// Register the polytone proxy address for a given chain
/// The polytone proxy will send messages to this address when it needs to execute actions on a local account.
fn register_chain_proxy(
    deps: DepsMut,
    info: MessageInfo,
    chain: TruncatedChainId,
    proxy: String,
) -> HostResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    chain.verify()?;

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

fn remove_chain_proxy(deps: DepsMut, info: MessageInfo, chain: TruncatedChainId) -> HostResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    chain.verify()?;

    if let Some(proxy) = CHAIN_PROXIES.may_load(deps.storage, &chain)? {
        REVERSE_CHAIN_PROXIES.remove(deps.storage, &proxy);
    }

    CHAIN_PROXIES.remove(deps.storage, &chain);
    Ok(HostResponse::action("register_chain_client"))
}
