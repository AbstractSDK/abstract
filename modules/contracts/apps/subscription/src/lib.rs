pub mod contract;
mod error;
mod handlers;
pub mod msg;
pub mod state;

pub use error::SubscriptionError;

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use std::str::FromStr;

    use abstract_app::abstract_interface::AppDeployer;
    use cosmwasm_std::Decimal;
    use cw_asset::AssetInfoUnchecked;
    use cw_orch::{interface, prelude::*};

    use crate::msg::*;

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, SubscriptionMigrateMsg)]
    pub struct Subscription;

    impl<Chain: CwEnv> AppDeployer<Chain> for Subscription<Chain> {}

    impl<Chain: CwEnv> Uploadable for Subscription<Chain> {
        fn wrapper() -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("abstract_subscription")
                .unwrap()
        }
    }

    impl<Chain: CwEnv> Subscription<Chain> {
        pub fn init_msg(payment_denom: String, token_addr: String) -> SubscriptionInstantiateMsg {
            SubscriptionInstantiateMsg {
                payment_asset: AssetInfoUnchecked::native(payment_denom),
                subscription_cost_per_second: Decimal::from_str("0.000001").unwrap(),
                subscription_per_second_emissions: crate::state::EmissionType::SecondShared(
                    Decimal::from_str("0.000001").unwrap(),
                    AssetInfoUnchecked::cw20(token_addr.clone()),
                ),
                // crate::state::EmissionType::IncomeBased(
                //     AssetInfoUnchecked::cw20(token_addr.clone()),
                // ),
                // 3 days
                income_averaging_period: 259200u64.into(),
                // contributors: Some(ContributorsInstantiateMsg {
                //     protocol_income_share: Decimal::percent(10),
                //     emission_user_share: Decimal::percent(50),
                //     max_emissions_multiple: Decimal::from_ratio(2u128, 1u128),
                //     token_info: AssetInfoUnchecked::cw20(token_addr),
                //     emissions_amp_factor: Uint128::new(680000),
                //     emissions_offset: Uint128::new(5200),
                // }),
                unsubscribe_hook_addr: None,
            }
        }
    }
}
