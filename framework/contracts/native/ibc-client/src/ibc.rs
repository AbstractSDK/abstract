use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};
use abstract_core::{
    ibc::IbcResponseMsg,
    ibc_client::{
        state::{REMOTE_PROXY, REVERSE_POLYTONE_NOTE},
        IbcClientCallback,
    },
};
use abstract_sdk::core::ibc_client::state::ACCOUNTS;
use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo};

use polytone::callbacks::{Callback, CallbackMessage};

/// This is not using IBC endpoints per se but corresponds to a Polytone IBC callback
pub fn receive_action_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: CallbackMessage,
) -> IbcClientResult {
    // 1. First we verify the callback is well formed and sent by the right contract

    // only a note contract can call this endpoint
    let host_chain = REVERSE_POLYTONE_NOTE.load(deps.storage, &info.sender)?;

    // only this account can call actions and have a polytone callback
    if callback.initiator != env.contract.address {
        return Err(IbcClientError::Unauthorized {});
    }

    // 2. From here on, we can trust the message that we are receiving

    let callback_msg: IbcClientCallback = from_binary(&callback.initiator_msg)?;

    let msg = match callback_msg {
        IbcClientCallback::WhoAmI {} => {
            // This response is used to store the Counterparty proxy address (this is used to whitelist the address on the host side)
            if let Callback::Execute(Ok(response)) = &callback.result {
                REMOTE_PROXY.save(deps.storage, &host_chain, &response.executed_by)?;
            } else {
                return Err(IbcClientError::IbcFailed(callback));
            }
            None
        }
        IbcClientCallback::CreateAccount { account_id } => {
            // We need to get the address of the remote proxy from the account creation response
            if let Callback::Execute(Ok(response)) = &callback.result {
                let account_creation_result = response.result[0].clone();

                let remote_proxy_address = account_creation_result
                    .events
                    .into_iter()
                    .find(|e| e.ty == "wasm")
                    .ok_or(IbcClientError::IbcFailed(callback.clone()))?
                    .attributes
                    .into_iter()
                    // We need to skip until we get to the actual account creation part
                    .skip_while(|e| !(e.key.eq("action") && e.value.eq("create_proxy")))
                    .find(|e| e.key == "proxy_address")
                    .ok_or(IbcClientError::IbcFailed(callback))?
                    .value;

                // We need to store the account address in the IBC client for interactions that may need it locally
                ACCOUNTS.save(
                    deps.storage,
                    (&account_id, &host_chain),
                    &remote_proxy_address,
                )?;
            } else {
                return Err(IbcClientError::IbcFailed(callback));
            }
            None
        }
        IbcClientCallback::UserRemoteAction(callback_info) => {
            // Here we transfer the callback back to the module that requested it
            let callback = IbcResponseMsg {
                id: callback_info.id,
                result: callback.result,
            };
            Some(callback.into_cosmos_account_msg(callback_info.receiver))
        }
    }
    .transpose()?;
    Ok(IbcClientResponse::action("acknowledge_register").add_messages(msg))
}
