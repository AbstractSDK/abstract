use abstract_app::objects::chain_name::ChainName;
use abstract_app::objects::module::ModuleInfo;
use abstract_app::objects::AccountId;
use abstract_app::sdk::IbcInterface;
use abstract_app::std::ibc::Callback;
use abstract_app::std::ibc_client::InstalledModuleIdentification;
use abstract_app::traits::AbstractResponse;
use cosmwasm_std::{ensure, to_json_binary, DepsMut, Env, MessageInfo};

use crate::contract::{App, AppResult};

use crate::error::AppError;
use crate::ibc;
use crate::msg::AppQueryMsg;
use crate::msg::{AppExecuteMsg, PingPongIbcMsg};
use crate::state::PREVIOUS_PING_PONG;

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { host_chain } => ping_pong(deps, host_chain, app),
        AppExecuteMsg::Rematch {
            host_chain,
            account_id,
        } => rematch(deps, host_chain, account_id, app),
    }
}

fn ping_pong(deps: DepsMut, host_chain: ChainName, app: App) -> AppResult {
    _ping_pong(deps, host_chain, app)
}

pub(crate) fn _ping_pong(deps: DepsMut, host_chain: ChainName, app: App) -> AppResult {
    let current_module_info = app.module_info()?;
    let ibc_client = app.ibc_client(deps.as_ref());
    let ibc_action = ibc_client.module_ibc_action(
        host_chain,
        current_module_info,
        &PingPongIbcMsg { pongs },
        None,
    )?;

    Ok(app
        .response("ping_pong")
        .add_attribute("pongs_left", pongs.to_string())
        .add_message(ibc_action))
}

fn rematch(deps: DepsMut, host_chain: ChainName, account_id: AccountId, app: App) -> AppResult {
    let ibc_client = app.ibc_client(deps.as_ref());

    let module_query = ibc_client.module_ibc_query(
        host_chain.clone(),
        InstalledModuleIdentification {
            module_info: app.module_info()?,
            account_id: Some(account_id),
        },
        &crate::msg::QueryMsg::from(AppQueryMsg::PreviousPingPong {}),
        Callback::new(to_json_binary(&ibc::PingPongIbcCallback::Rematch {
            rematch_chain: host_chain,
        })?),
    )?;

    Ok(app.response("rematch").add_message(module_query))
}
