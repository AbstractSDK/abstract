

use abstract_proxy::error::ProxyError;
use cosmwasm_std::SubMsg;
use crate::contract::RESPONSE_REPLY_ID;
use abstract_core::proxy::state::STATE;
use abstract_proxy::contract::ProxyResponse;
use cosmwasm_std::Empty;
use abstract_proxy::contract::ProxyResult;
use cosmwasm_std::DepsMut;

use cosmwasm_std::MessageInfo;

use cosmwasm_std::CosmosMsg;

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
/// This function patches the one defined in the original crate to have a different reply id for the module action submsg
pub fn execute_module_action_response(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: CosmosMsg<Empty>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    let submsg = SubMsg::reply_on_success(msg, RESPONSE_REPLY_ID);

    Ok(ProxyResponse::action("execute_module_action_response").add_submessage(submsg))
}