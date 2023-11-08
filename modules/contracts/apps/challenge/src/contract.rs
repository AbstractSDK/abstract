use crate::{
    error::AppError,
    handlers,
    msg::{ChallengeExecuteMsg, ChallengeInstantiateMsg, ChallengeQueryMsg},
};
use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};

/// The version of your app
pub const CHALLENGE_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// the id of the app
pub const CHALLENGE_APP_ID: &str = "abstract:challenge";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type ChallengeApp =
    AppContract<AppError, ChallengeInstantiateMsg, ChallengeExecuteMsg, ChallengeQueryMsg, Empty>;

const CHALLENGE_APP: ChallengeApp =
    ChallengeApp::new(CHALLENGE_APP_ID, CHALLENGE_APP_VERSION, None)
        .instantiate(handlers::instantiate_handler)
        .execute(handlers::execute_handler)
        .query(handlers::query_handler);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(CHALLENGE_APP, ChallengeApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(CHALLENGE_APP, ChallengeApp, ChallengeApp);
