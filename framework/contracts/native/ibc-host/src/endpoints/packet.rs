use crate::{
    account_commands::{self, receive_dispatch, receive_send_all_back},
    contract::HostResult,
    error::HostError,
    ibc::receive_register,
};
use abstract_core::{
    objects::AccountId,
    ibc_host::state::{CLIENT_PROXY, REVERSE_CHAIN_PROXYS}
};
use abstract_sdk::core::ibc_host::{HostAction, InternalAction};
use cosmwasm_std::{DepsMut, Env, MessageInfo, StdError};

/// Takes ibc request, matches and executes
/// This fn is the only way to get an Host instance.
pub fn handle_host_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut account_id: AccountId,
    host_action: HostAction,
) -> HostResult {
    // We verify the caller is indeed registered for the calling chain
    let client_chain = REVERSE_CHAIN_PROXYS.load(deps.storage, &info.sender)?;

    // push the client chain to the account trace
    account_id.trace_mut().push_chain(client_chain.clone());

    // get the local account information
    match host_action {
        HostAction::Internal(InternalAction::Register {
            description,
            link,
            name,
            account_proxy_address,
        }) => receive_register(
            deps,
            env,
            account_id,
            account_proxy_address,
            name,
            description,
            link,
        ),

        action => {
            let account = account_commands::get_account(deps.as_ref(), &account_id)?;
            match action {
                HostAction::Dispatch { manager_msg } => {
                    receive_dispatch(deps, account, manager_msg)
                }
                HostAction::SendAllBack {} => {
                    // address of the proxy on the client chain
                    let client_proxy_address = CLIENT_PROXY.load(deps.storage, &account_id)?;
                    receive_send_all_back(deps, env, account, client_proxy_address, client_chain)
                }
                HostAction::Internal(InternalAction::Register { .. }) => {
                    Err(HostError::Std(StdError::generic_err("Unreachable")))
                }
                HostAction::App { msg: _ } => todo!(),
            }
        }
    }
    .map_err(Into::into)
}
