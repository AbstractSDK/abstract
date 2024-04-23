use abstract_core::{
    ibc_host::{
        state::{ActionAfterCreationCache, TEMP_ACTION_AFTER_CREATION},
        HelperAction,
    },
    objects::{chain_name::ChainName, AccountId},
};
use abstract_sdk::core::ibc_host::{HostAction, InternalAction};
use cosmwasm_std::{DepsMut, Env};

use crate::{
    account_commands::{self, receive_dispatch, receive_register, receive_send_all_back},
    contract::HostResult,
};

pub fn client_to_host_account_id(remote_chain: ChainName, account_id: AccountId) -> AccountId {
    let mut account_id = account_id.clone();
    account_id.trace_mut().push_chain(remote_chain);

    account_id
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
    // Get the local account id from the remote account id
    // If the account_id is remote and the last trace matches the current chain, we unpack the chain name
    // Otherwise, we just add the sending chain to the trace
    let account_id = match received_account_id.trace() {
        abstract_core::objects::account::AccountTrace::Local => {
            client_to_host_account_id(client_chain.clone(), received_account_id.clone())
        }
        abstract_core::objects::account::AccountTrace::Remote(trace) => {
            if trace.last() == Some(&ChainName::from_chain_id(&env.block.chain_id)) {
                let mut new_trace = trace.clone();
                new_trace.pop();
                if new_trace.is_empty() {
                    AccountId::local(received_account_id.seq())
                } else {
                    AccountId::remote(received_account_id.seq(), new_trace)?
                }
            } else {
                client_to_host_account_id(client_chain.clone(), received_account_id.clone())
            }
        }
    };

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
