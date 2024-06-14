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
use crate::state::{CURRENT_PONGS, PREVIOUS_PING_PONG};

pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::PingPong { pongs, host_chain } => ping_pong(deps, pongs, host_chain, app),
        AppExecuteMsg::Rematch {
            host_chain,
            account_id,
        } => rematch(deps, host_chain, account_id, app),
    }
}

fn ping_pong(deps: DepsMut, pongs: u32, host_chain: ChainName, app: App) -> AppResult {
    ensure!(pongs > 0, AppError::ZeroPongs {});
    PREVIOUS_PING_PONG.save(deps.storage, &(pongs, host_chain.clone()))?;
    _ping_pong(deps, pongs, host_chain, app)
}

pub(crate) fn _ping_pong(deps: DepsMut, pongs: u32, host_chain: ChainName, app: App) -> AppResult {
    if pongs == 1 {
        // If we have 1 pong it means we send last pong, let's assume it succeeded
        CURRENT_PONGS.save(deps.storage, &0)?;
    } else {
        CURRENT_PONGS.save(deps.storage, &pongs)?;
    }

    let current_module_info = ModuleInfo::from_id(app.module_id(), app.version().into())?;
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
        Callback::new(to_json_binary(&ibc::PingPongIbcCallbacks::Rematch {
            rematch_chain: host_chain,
        })?),
    )?;

    Ok(app.response("rematch").add_message(module_query))
}
