use crate::error::SubscriptionError;
use crate::handlers;
use crate::msg::{AppMigrateMsg, SubscriptionInstantiateMsg};
use crate::msg::{SubscriptionExecuteMsg, SubscriptionQueryMsg};
use crate::SUBSCRIPTION;
use abstract_app::AppContract;
use cosmwasm_std::Response;
use cw20::Cw20ReceiveMsg;

pub type SubscriptionResult<T = Response> = Result<T, SubscriptionError>;

pub type SubscriptionApp = AppContract<
    SubscriptionError,
    SubscriptionInstantiateMsg,
    SubscriptionExecuteMsg,
    SubscriptionQueryMsg,
    AppMigrateMsg,
    Cw20ReceiveMsg,
>;

pub(crate) const BLOCKS_PER_MONTH: u64 = 10 * 60 * 24 * 30;

const SUBSCRIPTION_MODULE: SubscriptionApp =
    SubscriptionApp::new(SUBSCRIPTION, CONTRACT_VERSION, None)
        .with_execute(handlers::execute_handler)
        .with_instantiate(handlers::instantiate_handler)
        .with_query(handlers::query_handler)
        .with_receive(handlers::receive_cw20);
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// export endpoints
#[cfg(feature = "export")]
abstract_app::export_endpoints!(SUBSCRIPTION_MODULE, SubscriptionApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(SUBSCRIPTION_MODULE, SubscriptionApp, SubscriptionApp);
