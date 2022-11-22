use std::str::FromStr;

use abstract_sdk::os::{
    app::BaseInstantiateMsg, app::InstantiateMsg as AppInitMsg, subscription::*,
};
use boot_core::interface::BootExecute;
use boot_core::prelude::boot_contract;
use boot_core::{BootEnvironment, Contract};
use cosmwasm_std::{Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Subscription<Chain>;

impl<Chain: BootEnvironment> Subscription<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("subscription"), // .with_mock(Box::new(
                                                                       //     ContractWrapper::new_with_empty(
                                                                       //         ::contract::execute,
                                                                       //         ::contract::instantiate,
                                                                       //         ::contract::query,
                                                                       //     ),
                                                                       // ))
        )
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
            app: InstantiateMsg {
                subscription: abstract_sdk::os::subscription::SubscriptionInstantiateMsg {
                    factory_addr,
                    payment_asset: AssetInfoUnchecked::native(payment_denom),
                    subscription_cost_per_block: Decimal::from_str("0.000001").unwrap(),
                    version_control_addr,
                    subscription_per_block_emissions: state::UncheckedEmissionType::IncomeBased(
                        AssetInfoUnchecked::cw20(token_addr.clone()),
                    ),
                },
                contribution: Some(abstract_sdk::os::subscription::ContributionInstantiateMsg {
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

    // pub  fn pay_subscription(&self, os_id: u32, manager: Manager<'_>) -> Result<CosmTxResponse, BootError> {
    //     let result: SubscriptionFeeResponse = self.query(QueryMsg::Fee {  })?;

    //     let asset = result.fee;
    //     let msg = CosmosMsg::Wasm(WasmMsg::Execute { contract_addr: (), msg: (), funds: () });
    //     manager.execute(&ManagerExec::ConfigureModule { module_name: PROXY, config_msg: () }, coins);

    //     self.execute(&ExecuteMsg::Pay {  os_id: os_id }, Some(&[Coin::create("uusd", asset.amount.u128().into())]))
    // }

    pub fn claim_contribution(&self, os_id: u32) -> anyhow::Result<()> {
        self.execute(&ExecuteMsg::ClaimCompensation { os_id }, None)?;
        Ok(())
    }

    pub fn claim_emissions(&self, os_id: u32) -> anyhow::Result<()> {
        self.execute(&ExecuteMsg::ClaimEmissions { os_id }, None)?;
        Ok(())
    }
}
