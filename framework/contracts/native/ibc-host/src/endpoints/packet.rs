use crate::{
    account_commands::{self, receive_dispatch, receive_register, receive_send_all_back},
    contract::HostResult,
    error::HostError,
};
use abstract_core::{
    ibc_host::{
        state::{ActionAfterCreationCache, REVERSE_CHAIN_PROXIES, TEMP_ACTION_AFTER_CREATION},
        ExecuteMsg, HelperAction,
    },
    objects::{chain_name::ChainName, AccountId},
};
use abstract_sdk::core::ibc_host::{HostAction, InternalAction};
use cosmwasm_std::{wasm_execute, DepsMut, Env, MessageInfo, Response, StdError, SubMsg};

use super::reply::INIT_BEFORE_ACTION_REPLY_ID;

/// Takes ibc request, matches and executes
pub fn handle_host_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proxy_address: String,
    account_id: AccountId,
    host_action: HostAction,
) -> HostResult {
    // We verify the caller is indeed registered for the calling chain
    let client_chain = REVERSE_CHAIN_PROXIES.load(deps.storage, &info.sender)?;

    // We execute the action
    _handle_host_action(
        deps,
        env,
        client_chain,
        proxy_address,
        account_id,
        host_action,
    )
}

// Internal function non permissioned
// We added this step to be able to execute actions from inside the ibc host
pub(crate) fn _handle_host_action(
    deps: DepsMut,
    env: Env,
    client_chain: ChainName,
    proxy_address: String,
    received_account_id: AccountId,
    host_action: HostAction,
) -> HostResult {
    // Push the client chain to the account trace
    let mut account_id = received_account_id.clone();
    account_id.trace_mut().push_chain(client_chain.clone());

    // get the local account information
    match host_action {
        HostAction::Internal(InternalAction::Register {
            description,
            link,
            name,
        }) => receive_register(deps, env, account_id, name, description, link),

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
                let create_account_message = wasm_execute(
                    env.contract.address,
                    &ExecuteMsg::InternalRegisterAccount {
                        client_chain: client_chain.to_string(),
                        account_id,
                    },
                    vec![],
                )?;

                // We save the action they wanted to dispatch
                TEMP_ACTION_AFTER_CREATION.save(
                    deps.storage,
                    &ActionAfterCreationCache {
                        action,
                        client_proxy_address: proxy_address,
                        account_id: received_account_id,
                        chain_name: client_chain,
                    },
                )?;

                // We add a submessage after account creation to dispatch the action
                let sub_msg =
                    SubMsg::reply_on_success(create_account_message, INIT_BEFORE_ACTION_REPLY_ID);

                Ok(Response::new().add_submessage(sub_msg))
            }
        }
    }
    .map_err(Into::into)
}
