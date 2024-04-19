use abstract_sdk::std::ibc_client::state::ACCOUNTS;
use abstract_std::{
    ibc::IbcResponseMsg,
    ibc_client::{
        state::{IBC_INFRA, REVERSE_POLYTONE_NOTE},
        IbcClientCallback,
    },
    objects::chain_name::ChainName,
};
use cosmwasm_std::{from_json, Attribute, DepsMut, Env, MessageInfo};
use polytone::callbacks::{Callback, CallbackMessage};

use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};

/// This is not using IBC endpoints per se but corresponds to a Polytone IBC callback
pub fn receive_action_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    callback: CallbackMessage,
) -> IbcClientResult {
    // 1. First we verify the callback is well formed and sent by the right contract

    // only a note contract can call this endpoint
    let host_chain: ChainName = REVERSE_POLYTONE_NOTE
        .may_load(deps.storage, &info.sender)?
        .ok_or(IbcClientError::Unauthorized {})?;

    // only this account can call actions and have a polytone callback
    if callback.initiator != env.contract.address {
        return Err(IbcClientError::Unauthorized {});
    }

    // 2. From here on, we can trust the message that we are receiving

    let callback_msg: IbcClientCallback = from_json(&callback.initiator_msg)?;

    match callback_msg {
        IbcClientCallback::WhoAmI {} => {
            // This response is used to store the Counterparty proxy address (this is used to whitelist the address on the host side)
            if let Callback::Execute(Ok(response)) = &callback.result {
                IBC_INFRA.update(deps.storage, &host_chain, |c| match c {
                    None => Err(IbcClientError::UnregisteredChain(host_chain.to_string())),
                    Some(mut counterpart) => {
                        counterpart.remote_proxy = Some(response.executed_by.clone());
                        Ok(counterpart)
                    }
                })?;
            } else {
                return Err(IbcClientError::IbcFailed(callback));
            }
            Ok(IbcClientResponse::action("register_remote_proxy")
                .add_attribute("chain", host_chain.to_string()))
        }
        IbcClientCallback::CreateAccount { account_id } => {
            // We need to get the address of the remote proxy from the account creation response
            if let Callback::Execute(Ok(response)) = &callback.result {
                let account_creation_result = response.result[0].clone();

                let wasm_abstract_attributes: Vec<Attribute> = account_creation_result
                    .events
                    .into_iter()
                    .filter(|e| e.ty == "wasm-abstract")
                    .flat_map(|e| e.attributes)
                    .collect();

                let remote_proxy_address = &wasm_abstract_attributes
                    .iter()
                    .find(|e| e.key == "proxy_address")
                    .ok_or(IbcClientError::IbcFailed(callback))?
                    .value;

                // We need to store the account address in the IBC client for interactions that may need it locally
                ACCOUNTS.save(
                    deps.storage,
                    (account_id.trace(), account_id.seq(), &host_chain),
                    remote_proxy_address,
                )?;
            } else {
                return Err(IbcClientError::IbcFailed(callback));
            }
            Ok(
                IbcClientResponse::action("acknowledge_remote_account_registration")
                    .add_attribute("account_id", account_id.to_string())
                    .add_attribute("chain", host_chain.to_string()),
            )
        }
        IbcClientCallback::UserRemoteAction(callback_info) => {
            // Here we transfer the callback back to the module that requested it
            let callback = IbcResponseMsg {
                id: callback_info.id.clone(),
                msg: callback_info.msg,
                result: callback.result,
            };
            Ok(IbcClientResponse::action("user_specific_callback")
                .add_message(callback.into_cosmos_msg(callback_info.receiver)?)
                .add_attribute("chain", host_chain.to_string())
                .add_attribute("callback_id", callback_info.id))
        }
    }
}
