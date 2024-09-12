use abstract_std::{
    ibc::{
        polytone_callbacks::{Callback as PolytoneCallback, CallbackMessage},
        IbcResponseMsg, IbcResult,
    },
    ibc_client::{
        state::{ACCOUNTS, IBC_INFRA, REVERSE_POLYTONE_NOTE},
        IbcClientCallback,
    },
    objects::TruncatedChainId,
};
use cosmwasm_std::{from_json, Attribute, DepsMut, Env, MessageInfo};

use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};

/// This is not using IBC endpoints per se but corresponds to a Polytone IBC callback
pub fn receive_action_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    polytone_callback: CallbackMessage,
) -> IbcClientResult {
    // 1. First we verify the callback is well formed and sent by the right contract

    // only a note contract can call this endpoint
    let host_chain: TruncatedChainId = REVERSE_POLYTONE_NOTE
        .may_load(deps.storage, &info.sender)?
        .ok_or(IbcClientError::Unauthorized {})?;

    // only this account can call actions and have a polytone callback
    if polytone_callback.initiator != env.contract.address {
        return Err(IbcClientError::Unauthorized {});
    }

    // 2. From here on, we can trust the message that we are receiving

    let callback_msg: IbcClientCallback = from_json(&polytone_callback.initiator_msg)?;

    match callback_msg {
        IbcClientCallback::WhoAmI {} => {
            // This response is used to store the Counterparty proxy address (this is used to whitelist the address on the host side)
            if let PolytoneCallback::Execute(Ok(response)) = &polytone_callback.result {
                IBC_INFRA.update(deps.storage, &host_chain, |c| match c {
                    None => Err(IbcClientError::UnregisteredChain(host_chain.to_string())),
                    Some(mut counterpart) => {
                        counterpart.remote_proxy = Some(response.executed_by.clone());
                        Ok(counterpart)
                    }
                })?;
            } else {
                return Err(IbcClientError::IbcFailed(polytone_callback));
            }
            Ok(IbcClientResponse::action("register_remote_proxy")
                .add_attribute("chain", host_chain.to_string()))
        }
        IbcClientCallback::CreateAccount { account_id } => {
            // We need to get the address of the remote proxy from the account creation response
            if let PolytoneCallback::Execute(Ok(response)) = &polytone_callback.result {
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
                    .ok_or(IbcClientError::IbcFailed(polytone_callback))?
                    .value;

                // We need to store the account address in the IBC client for interactions that may need it locally
                ACCOUNTS.save(
                    deps.storage,
                    (account_id.trace(), account_id.seq(), &host_chain),
                    remote_proxy_address,
                )?;
            } else {
                return Err(IbcClientError::IbcFailed(polytone_callback));
            }
            Ok(
                IbcClientResponse::action("acknowledge_remote_account_registration")
                    .add_attribute("account_id", account_id.to_string())
                    .add_attribute("chain", host_chain.to_string()),
            )
        }
        IbcClientCallback::ModuleRemoteAction {
            callback,
            sender_address,
            initiator_msg,
        } => {
            let resp_msg = IbcResponseMsg {
                callback,
                result: IbcResult::from_execute(polytone_callback.result, initiator_msg)?,
            };
            Ok(IbcClientResponse::action("module_action_ibc_callback")
                .add_message(resp_msg.into_cosmos_msg(sender_address)?)
                .add_attribute("chain", host_chain.to_string()))
        }
        IbcClientCallback::ModuleRemoteQuery {
            sender_address,
            callback,
            queries,
        } => {
            let reps_msg = IbcResponseMsg {
                callback,
                result: IbcResult::from_query(polytone_callback.result, queries)?,
            };
            Ok(IbcClientResponse::action("module_query_ibc_callback")
                .add_message(reps_msg.into_cosmos_msg(sender_address)?)
                .add_attribute("chain", host_chain.to_string()))
        }
    }
}
