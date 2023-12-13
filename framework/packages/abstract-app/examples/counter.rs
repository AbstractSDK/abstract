pub use abstract_core::app;

pub use cosmwasm_std::testing::*;
use cosmwasm_std::{Response, StdError};

pub type CounterResult<T = ()> = Result<T, CounterError>;

#[cosmwasm_schema::cw_serde]
pub struct CounterInitMsg;

#[cosmwasm_schema::cw_serde]
pub enum CounterExecMsg {
    UpdateConfig {},
}

#[cosmwasm_schema::cw_serde]
pub struct CounterQueryMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterMigrateMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterReceiveMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterSudoMsg;

abstract_app::app_msg_types!(CounterApp, CounterExecMsg, CounterQueryMsg);

use abstract_app::{AppContract, AppError};

use abstract_sdk::AbstractSdkError;

use cw_controllers::AdminError;
use thiserror::Error;

// ANCHOR: error
#[derive(Error, Debug, PartialEq)]
pub enum CounterError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DappError(#[from] AppError),

    #[error("{0}")]
    Abstract(#[from] abstract_core::AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Unauthorized(#[from] AdminError),
}
// ANCHOR_END: error

// ANCHOR: counter_app
pub type CounterApp = AppContract<
    CounterError,
    CounterInitMsg,
    CounterExecMsg,
    CounterQueryMsg,
    CounterMigrateMsg,
    CounterReceiveMsg,
    CounterSudoMsg,
>;
// ANCHOR_END: counter_app

const COUNTER_ID: &str = "example:counter";
const APP_VERSION: &str = "1.0.0";

// ANCHOR: handlers
// ANCHOR: new
pub fn counter_app() -> CounterApp {
    CounterApp::new(COUNTER_ID, APP_VERSION, None)
        // ANCHOR_END: new
        .with_instantiate(handlers::instantiate)
        .with_execute(handlers::execute)
        .with_query(handlers::query)
        .with_sudo(handlers::sudo)
        .with_receive(handlers::receive)
        .with_replies(&[(1u64, handlers::reply)])
        .with_migrate(handlers::migrate)
}
// ANCHOR_END: handlers

// ANCHOR: export
abstract_app::export_endpoints!(counter_app, CounterApp);
// ANCHOR_END: export

// ANCHOR: interface
abstract_app::cw_orch_interface!(counter_app, CounterApp, CounterAppInterface);
// ANCHOR_END: interface

mod handlers {
    #![allow(non_upper_case_globals)]
    use abstract_sdk::{base::*, features::CustomData, AbstractResponse};
    use cosmwasm_std::*;

    use super::*;

    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        app: &mut CounterApp,
        _msg: CounterInitMsg,
    ) -> CounterResult<()> {
        app.set_data("counter_init".as_bytes());
        Ok(())
    }
    pub fn query(
        deps: Deps,
        env: Env,
        _app: &CounterApp,
        _msg: CounterQueryMsg,
    ) -> CounterResult<Binary> {
        to_json_binary("counter_query").map_err(Into::into)
    }
    pub fn sudo(
        deps: DepsMut,
        env: Env,
        app: &mut CounterApp,
        _msg: CounterSudoMsg,
    ) -> CounterResult<()> {
        app.set_data("counter_sudo".as_bytes());
        Ok(())
    }
    pub fn receive(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        app: &mut CounterApp,
        _msg: CounterReceiveMsg,
    ) -> CounterResult<()> {
        app.set_data("counter_receive".as_bytes());
        Ok(())
    }
    pub fn reply(deps: DepsMut, env: Env, app: &mut CounterApp, msg: Reply) -> CounterResult<()> {
        app.set_data(msg.result.unwrap().data.unwrap());
        Ok(())
    }
    pub fn migrate(
        deps: DepsMut,
        env: Env,
        app: &mut CounterApp,
        _msg: CounterMigrateMsg,
    ) -> CounterResult<()> {
        app.set_data("counter_migrate".as_bytes());
        Ok(())
    }
    // ANCHOR: execute
    pub fn execute(
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        app: &mut CounterApp, // <-- Notice how the `CounterApp` is available here
        msg: CounterExecMsg,
    ) -> CounterResult<()> {
        match msg {
            CounterExecMsg::UpdateConfig {} => update_config(deps, info, app),
        }
    }

    /// Update the configuration of the app
    fn update_config(deps: DepsMut, msg_info: MessageInfo, app: &mut CounterApp) -> CounterResult {
        // Only the admin should be able to call this
        app.admin.assert_admin(deps.as_ref(), &msg_info.sender)?;

        app.set_data("counter_exec".as_bytes());
        app.tag_response("update_config");

        Ok(())
    }
    // ANCHOR_END: execute
}

fn main() {}
