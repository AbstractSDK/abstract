pub use abstract_core::app;

pub use cosmwasm_std::testing::*;
use cosmwasm_std::StdError;

pub type CounterResult = Result<(), CounterError>;

#[cosmwasm_schema::cw_serde]
pub struct CounterInitMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterExecMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterQueryMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterMigrateMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterReceiveMsg;

#[cosmwasm_schema::cw_serde]
pub struct CounterSudoMsg;

abstract_app::app_messages!(CounterApp, CounterExecMsg, CounterQueryMsg);

use abstract_app::{AppContract, AppError};

use abstract_sdk::AbstractSdkError;

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
pub const COUNTER_APP: CounterApp = CounterApp::new(COUNTER_ID, APP_VERSION, None)
    // ANCHOR_END: new
    .with_instantiate(handlers::instantiate)
    .with_execute(handlers::execute)
    .with_query(handlers::query)
    .with_sudo(handlers::sudo)
    .with_receive(handlers::receive)
    .with_replies(&[(1u64, handlers::reply)])
    .with_migrate(handlers::migrate);
// ANCHOR_END: handlers

// ANCHOR: export
abstract_app::export_endpoints!(COUNTER_APP, CounterApp);
// ANCHOR_END: export

// ANCHOR: interface
abstract_app::create_interface!(COUNTER_APP, CounterApp);
// ANCHOR_END: interface

mod handlers {
    #![allow(non_upper_case_globals)]
    use abstract_sdk::base::*;
    use cosmwasm_std::{to_binary, Response};

    use super::*;

    pub const instantiate: InstantiateHandlerFn<CounterApp, CounterInitMsg, CounterError> =
        |_, _, _, _, _| Ok(Response::new().set_data("counter_init".as_bytes()));
    pub const execute: ExecuteHandlerFn<CounterApp, CounterExecMsg, CounterError> =
        |_, _, _, _, _| Ok(Response::new().set_data("counter_exec".as_bytes()));
    pub const query: QueryHandlerFn<CounterApp, CounterQueryMsg, CounterError> =
        |_, _, _, _| to_binary("counter_query").map_err(Into::into);
    pub const sudo: SudoHandlerFn<CounterApp, CounterSudoMsg, CounterError> =
        |_, _, _, _| Ok(Response::new().set_data("counter_sudo".as_bytes()));
    pub const receive: ReceiveHandlerFn<CounterApp, CounterReceiveMsg, CounterError> =
        |_, _, _, _, _| Ok(Response::new().set_data("counter_receive".as_bytes()));
    pub const reply: ReplyHandlerFn<CounterApp, CounterError> =
        |_, _, _, msg| Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()));
    pub const migrate: MigrateHandlerFn<CounterApp, CounterMigrateMsg, CounterError> =
        |_, _, _, _| Ok(Response::new().set_data("counter_migrate".as_bytes()));
}

fn main() {}
