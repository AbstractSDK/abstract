use abstract_sdk::std::{
    account::state::{ADMIN, WHITELISTED_MODULES},
    ibc_client::ExecuteMsg as IbcClientMsg,
    IBC_CLIENT,
};
use abstract_std::ICA_CLIENT;
use cosmwasm_std::{
    wasm_execute, Binary, CosmosMsg, DepsMut, Empty, MessageInfo, StdError, SubMsg, WasmQuery,
};

use crate::{
    contract::{AccountResponse, AccountResult, RESPONSE_REPLY_ID},
    error::AccountError,
};

/// Executes `CosmosMsg` on the proxy and forwards its response.
/// Permission: Module
pub fn execute_module_action_response(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: CosmosMsg<Empty>,
) -> AccountResult {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if !whitelisted_modules.0.contains(&msg_info.sender) {
        return Err(AccountError::SenderNotWhitelisted {});
    }

    let submsg = SubMsg::reply_on_success(msg, RESPONSE_REPLY_ID);

    Ok(AccountResponse::action("execute_module_action_response").add_submessage(submsg))
}

/// Executes `Vec<CosmosMsg>` on the proxy.
/// Permission: Module
pub fn execute_module_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> AccountResult {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if !whitelisted_modules.0.contains(&msg_info.sender) {
        return Err(AccountError::SenderNotWhitelisted {});
    }

    Ok(AccountResponse::action("execute_module_action").add_messages(msgs))
}

/// Executes IBC actions on the IBC client.
/// Permission: Module
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: IbcClientMsg,
) -> AccountResult {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if !whitelisted_modules.0.contains(&msg_info.sender) {
        return Err(AccountError::SenderNotWhitelisted {});
    }
    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ibc_client_address = abstract_sdk::std::account::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address, IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on manager. Add it under the {IBC_CLIENT} name."
            ))
        })?;

    let funds_to_send = if let IbcClientMsg::SendFunds { funds, .. } = &msg {
        funds.clone()
    } else {
        vec![]
    };
    let client_msg = wasm_execute(ibc_client_address, &msg, funds_to_send)?;

    Ok(AccountResponse::action("execute_ibc_action").add_message(client_msg))
}

/// Execute an action on an ICA.
/// Permission: Module
///
/// This function queries the `abstract:ica-client` contract from the account's manager.
/// It then fires a smart-query on that address of type [`QueryMsg::IcaAction`](abstract_ica::msg::QueryMsg).
///
/// The resulting `Vec<CosmosMsg>` are then executed on the proxy contract.
pub fn ica_action(deps: DepsMut, msg_info: MessageInfo, action_query: Binary) -> AccountResult {
    let whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;
    if !whitelisted_modules.0.contains(&msg_info.sender) {
        return Err(AccountError::SenderNotWhitelisted {});
    }

    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ica_client_address = abstract_sdk::std::account::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address, ICA_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ica_client not found on manager. Add it under the {ICA_CLIENT} name."
            ))
        })?;

    let res: abstract_ica::msg::IcaActionResult = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: ica_client_address.into(),
            msg: action_query,
        }
        .into(),
    )?;

    Ok(AccountResponse::action("ica_action").add_messages(res.msgs))
}
