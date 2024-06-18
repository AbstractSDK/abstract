pub use abstract_std::standalone;
pub use cosmwasm_std::testing::*;
use cosmwasm_std::{Response, StdError};

pub type CounterResult<T = Response> = Result<T, CounterError>;

#[cosmwasm_schema::cw_serde]
pub struct CounterInitMsg {
    pub base: standalone::StandaloneInstantiateMsg,
}

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

use abstract_sdk::AbstractSdkError;
use abstract_standalone::StandaloneContract;
use cw_controllers::AdminError;
use thiserror::Error;

// ANCHOR: error
#[derive(Error, Debug, PartialEq)]
pub enum CounterError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Abstract(#[from] abstract_std::AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Unauthorized(#[from] AdminError),
}
// ANCHOR_END: error

// ANCHOR: counter_app
pub type CounterApp = StandaloneContract;
// ANCHOR_END: counter_app

const COUNTER_ID: &str = "example:counter";
const APP_VERSION: &str = "1.0.0";

// ANCHOR: new
pub const COUNTER_APP: CounterApp = CounterApp::new(COUNTER_ID, APP_VERSION, None);
// ANCHOR_END: new

mod handlers {
    #![allow(non_upper_case_globals)]
    use abstract_sdk::AbstractResponse;
    use cosmwasm_std::*;

    use super::*;

    #[allow(unused)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: CounterInitMsg,
    ) -> Result<Response, CounterError> {
        COUNTER_APP.instantiate(deps, &env, info, msg.base, true)?;
        Ok(COUNTER_APP.response("instantiate"))
    }
}

fn main() {}
