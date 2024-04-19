use abstract_sdk::std::ibc_host::{HostAction, InternalAction};
use abstract_std::{
    ibc_host::{
        state::{ActionAfterCreationCache, TEMP_ACTION_AFTER_CREATION},
        HelperAction,
    },
    objects::{chain_name::ChainName, AccountId},
};
use cosmwasm_std::{DepsMut, Env};

use crate::{
    account_commands::{self, receive_dispatch, receive_register, receive_send_all_back},
    contract::HostResult,
};

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
    let mut account_id = received_account_id.clone();
    account_id.trace_mut().push_chain(client_chain.clone());

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
