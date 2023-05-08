use abstract_core::objects::fee::Fee;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, to_binary, wasm_execute, WasmMsg};
use cw20::MinterResponse;

use crate::contract::{DEFAULT_LP_TOKEN_NAME, DEFAULT_LP_TOKEN_SYMBOL, TemplateApp, TemplateResult};
use crate::msg::{ExecuteMsg, TemplateExecuteMsg, TemplateInstantiateMsg};
use crate::state::{Config, CONFIG};

pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _app: TemplateApp,
    _msg: TemplateInstantiateMsg,
) -> TemplateResult {
    let config: Config = Config {};

    CONFIG.save(deps.storage, &config)?;

    // Example reply that doesn't do anything
    Ok(Response::new().add_submessage(SubMsg::reply_on_success(
        wasm_execute(_env.contract.address, &cosmwasm_std::Empty {}, vec![])?,
        INSTANTIATE_REPLY_ID,
    )))
}
