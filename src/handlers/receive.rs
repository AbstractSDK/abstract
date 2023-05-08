use crate::{
    contract::{TemplateApp, TemplateResult},
    msg::Cw20HookMsg,
};
use abstract_sdk::AbstractResponse;
use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_handler(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: TemplateApp,
    cw20_msg: Cw20ReceiveMsg,
) -> TemplateResult {
    match from_binary(&cw20_msg.msg)? {
        Cw20HookMsg::Deposit {} => {
            // Do nothing, just return
            Ok(app.custom_tag_response(
                Response::default(),
                "receive_cw20",
                vec![("method", "deposit")],
            ))
        }
    }
}
