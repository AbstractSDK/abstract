mod commands;
pub mod contract;
pub mod error;
pub mod msg;
pub mod queries;
pub mod state;

pub const SUBSCRIPTION: &str = "abstract:subscription";

#[cfg(feature = "boot")]
pub mod boot {
    use crate::msg::*;
    use abstract_boot::AppDeployer;
    use abstract_core::app::{BaseInstantiateMsg, InstantiateMsg as AppInitMsg};
    use boot_core::ContractWrapper;
    use boot_core::{boot_contract, BootEnvironment, Contract};
    use cosmwasm_std::{Decimal, Uint128};
    use cw_asset::AssetInfoUnchecked;
    use std::str::FromStr;

    #[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
    pub struct Subscription<Chain>;

    impl<Chain: BootEnvironment> AppDeployer<Chain> for Subscription<Chain> {}

    impl<Chain: BootEnvironment> Subscription<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            let mut contract = Contract::new(name, chain);
            contract = contract.with_wasm_path("subscription").with_mock(Box::new(
                ContractWrapper::new_with_empty(
                    crate::contract::execute,
                    crate::contract::instantiate,
                    crate::contract::query,
                ),
            ));
            Self(contract)
        }
        pub fn init_msg(
            payment_denom: String,
            token_addr: String,
            ans_host_address: String,
            factory_addr: String,
            version_control_addr: String,
        ) -> AppInitMsg<InstantiateMsg> {
            AppInitMsg::<InstantiateMsg> {
                base: BaseInstantiateMsg { ans_host_address },
                module: InstantiateMsg {
                    subscription: SubscriptionInstantiateMsg {
                        factory_addr,
                        payment_asset: AssetInfoUnchecked::native(payment_denom),
                        subscription_cost_per_block: Decimal::from_str("0.000001").unwrap(),
                        version_control_addr,
                        subscription_per_block_emissions:
                            crate::state::UncheckedEmissionType::IncomeBased(
                                AssetInfoUnchecked::cw20(token_addr.clone()),
                            ),
                    },
                    contribution: Some(ContributionInstantiateMsg {
                        protocol_income_share: Decimal::percent(10),
                        emission_user_share: Decimal::percent(50),
                        max_emissions_multiple: Decimal::from_ratio(2u128, 1u128),
                        token_info: AssetInfoUnchecked::cw20(token_addr),
                        emissions_amp_factor: Uint128::new(680000),
                        emissions_offset: Uint128::new(5200),
                        // 3 days
                        income_averaging_period: 259200u64.into(),
                    }),
                },
            }
        }
    }
}
