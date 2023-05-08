use abstract_app::state::AppState;
use abstract_sdk::{
    *, core::objects::deposit_info::DepositInfo,
    core::objects::fee::Fee, core::proxy::AssetsInfoResponse,
    cw_helpers::cosmwasm_std::wasm_smart_query, features::AbstractResponse,
};
use cosmwasm_std::{
    Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, to_binary, Uint128,
    wasm_execute, WasmMsg,
};
use cosmwasm_std::{QuerierWrapper, StdResult};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use cw_asset::{Asset, AssetInfo};

use crate::contract::{TemplateApp, TemplateResult};
use crate::error::TemplateError;
use crate::msg::TemplateExecuteMsg;
use crate::state::{Config, CONFIG};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: TemplateApp,
    msg: TemplateExecuteMsg,
) -> TemplateResult {
    match msg {
        TemplateExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, msg_info: MessageInfo, app: TemplateApp) -> TemplateResult {
    // Only the admin should be able to call this
    app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(app.tag_response(
        Response::default(),
        "update_config",
    ))
}
