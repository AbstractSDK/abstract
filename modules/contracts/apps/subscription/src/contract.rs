use abstract_app::AppContract;
use cosmwasm_std::Response;
use cw20::Cw20ReceiveMsg;

use crate::{
    handlers,
    msg::{
        SubscriptionExecuteMsg, SubscriptionInstantiateMsg, SubscriptionMigrateMsg,
        SubscriptionQueryMsg,
    },
    SubscriptionError,
};

pub type SubscriptionResult<T = Response> = Result<T, SubscriptionError>;

pub type SubscriptionApp = AppContract<
    SubscriptionError,
    SubscriptionInstantiateMsg,
    SubscriptionExecuteMsg,
    SubscriptionQueryMsg,
    SubscriptionMigrateMsg,
    Cw20ReceiveMsg,
>;

pub const SUBSCRIPTION_ID: &str = "abstract:subscription";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const SUBSCRIPTION_MODULE: SubscriptionApp =
    SubscriptionApp::new(SUBSCRIPTION_ID, CONTRACT_VERSION, None)
        .with_execute(handlers::execute_handler)
        .with_instantiate(handlers::instantiate_handler)
        .with_query(handlers::query_handler);

// export endpoints
#[cfg(feature = "export")]
abstract_app::export_endpoints!(SUBSCRIPTION_MODULE, SubscriptionApp);

abstract_app::cw_orch_interface!(SUBSCRIPTION_MODULE, SubscriptionApp, SubscriptionInterface);

#[cfg(not(target_arch = "wasm32"))]
impl<Chain: cw_orch::prelude::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for self::interface::SubscriptionInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        to_json_binary, Addr, CosmosMsg, Decimal, SubMsg, WasmMsg,
    };

    use super::*;
    use crate::{
        msg::{HookReceiverExecuteMsg, UnsubscribedHookMsg},
        state::{
            Subscriber, SubscriptionConfig, SubscriptionState, INCOME_TWA, SUBSCRIBERS,
            SUBSCRIPTION_CONFIG, SUBSCRIPTION_STATE,
        },
    };

    #[test]
    fn unsubscribe_no_hook_msg() {
        let mut deps = mock_dependencies();
        let depsmut = deps.as_mut();
        let env = mock_env();
        let app = SUBSCRIPTION_MODULE;

        INCOME_TWA
            .instantiate(depsmut.storage, &env, None, 259200u64)
            .unwrap();
        SUBSCRIPTION_CONFIG
            .save(
                depsmut.storage,
                &SubscriptionConfig {
                    payment_asset: cw_asset::AssetInfoBase::Native("token".to_owned()),
                    subscription_cost_per_second: Decimal::from_str("0.1").unwrap(),
                    subscription_per_second_emissions: crate::state::EmissionType::None,
                    unsubscribe_hook_addr: None,
                },
            )
            .unwrap();
        SUBSCRIBERS
            .save(
                depsmut.storage,
                &Addr::unchecked("bob"),
                &Subscriber {
                    expiration_timestamp: env.block.time,
                    last_emission_claim_timestamp: env.block.time,
                },
            )
            .unwrap();
        SUBSCRIPTION_STATE
            .save(depsmut.storage, &SubscriptionState { active_subs: 1 })
            .unwrap();

        let res =
            handlers::execute::unsubscribe(depsmut, env, app, vec!["bob".to_string()]).unwrap();

        assert!(res.messages.is_empty());
    }

    #[test]
    fn unsubscribe_with_hook_msg() {
        let mut deps = mock_dependencies();
        let depsmut = deps.as_mut();
        let env = mock_env();
        let app = SUBSCRIPTION_MODULE;

        INCOME_TWA
            .instantiate(depsmut.storage, &env, None, 259200u64)
            .unwrap();
        SUBSCRIPTION_CONFIG
            .save(
                depsmut.storage,
                &SubscriptionConfig {
                    payment_asset: cw_asset::AssetInfoBase::Native("token".to_owned()),
                    subscription_cost_per_second: Decimal::from_str("0.1").unwrap(),
                    subscription_per_second_emissions: crate::state::EmissionType::None,
                    unsubscribe_hook_addr: Some(Addr::unchecked("alice")),
                },
            )
            .unwrap();
        SUBSCRIBERS
            .save(
                depsmut.storage,
                &Addr::unchecked("bob"),
                &Subscriber {
                    expiration_timestamp: env.block.time,
                    last_emission_claim_timestamp: env.block.time,
                },
            )
            .unwrap();
        SUBSCRIPTION_STATE
            .save(depsmut.storage, &SubscriptionState { active_subs: 1 })
            .unwrap();

        let res =
            handlers::execute::unsubscribe(depsmut, env, app, vec!["bob".to_string()]).unwrap();

        let expected_msg = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "alice".to_owned(),
            msg: to_json_binary(&HookReceiverExecuteMsg::Unsubscribed(UnsubscribedHookMsg {
                unsubscribed: vec!["bob".to_owned()],
            }))
            .unwrap(),
            funds: vec![],
        }));
        assert_eq!(res.messages, vec![expected_msg]);
    }
}
