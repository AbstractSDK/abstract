pub mod contract;
mod handlers;
mod error;
pub mod msg;
pub mod state;
pub mod queries;

pub use error::SubscriptionError;

/// Duration of subscription in weeks
pub const DURATION_IN_WEEKS: u64 = 4;
pub const WEEK_IN_SECONDS: u64 = 7 * 24 * 60 * 60;

#[cfg(feature = "interface")]
pub mod interface {
    use crate::msg::*;
    use abstract_core::app::{BaseInstantiateMsg, InstantiateMsg as AppInitMsg};
    use abstract_interface::AppDeployer;
    use cosmwasm_std::Decimal;
    use cw_asset::AssetInfoUnchecked;
    use cw_orch::{interface, prelude::*};
    use std::str::FromStr;

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, SubscriptionMigrateMsg)]
    pub struct Subscription;

    impl<Chain: CwEnv> AppDeployer<Chain> for Subscription<Chain> {}

    impl<Chain: CwEnv> Uploadable for Subscription<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
            artifacts_dir_from_workspace!()
                .find_wasm_path("abstract_subscription")
                .unwrap()
        }
    }

    impl<Chain: CwEnv> Subscription<Chain> {
        pub fn init_msg(
            payment_denom: String,
            token_addr: String,
            ans_host_address: String,
            factory_addr: String,
            version_control_addr: String,
        ) -> AppInitMsg<SubscriptionInstantiateMsg> {
            AppInitMsg::<SubscriptionInstantiateMsg> {
                base: BaseInstantiateMsg {
                    ans_host_address,
                    version_control_address: version_control_addr,
                },
                module: SubscriptionInstantiateMsg {
                    factory_addr,
                    payment_asset: AssetInfoUnchecked::native(payment_denom),
                    subscription_cost_per_week: Decimal::from_str("0.000001").unwrap(),
                    subscription_per_week_emissions: crate::state::EmissionType::WeekShared(
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
                },
            }
        }
    }
}
