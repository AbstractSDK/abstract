use crate::Manager;
use abstract_os::dex::{DexAction, DexExecuteMsg};
use abstract_os::{
    api::{self},
    dex::{ExecuteMsg, InstantiateMsg, QueryMsg},
    objects::{AnsAsset, AssetEntry},
    EXCHANGE, MANAGER,
};
use boot_core::{interface::ContractInstance, prelude::boot_contract, BootEnvironment, Contract};
use cosmwasm_std::{Decimal, Empty};
use log::info;

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct DexApi<Chain>;

impl<Chain: BootEnvironment> DexApi<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("dex"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }

    /// Swap using Abstract's OS (registered in daemon_state).
    pub fn swap(
        &self,
        offer_asset: (&str, u128),
        ask_asset: &str,
        dex: String,
    ) -> Result<(), crate::AbstractBootError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        let asset = AssetEntry::new(offer_asset.0);
        let ask_asset = AssetEntry::new(ask_asset);

        let swap_msg = abstract_os::dex::ExecuteMsg::App(api::ApiRequestMsg {
            proxy_address: None,
            request: DexExecuteMsg {
                dex,
                action: DexAction::Swap {
                    offer_asset: AnsAsset::new(asset, offer_asset.1),
                    ask_asset,
                    max_spread: Some(Decimal::percent(30)),
                    belief_price: None,
                },
            }
            .into(),
        });

        info!("Swap msg: {:?}", serde_json::to_string(&swap_msg)?);
        manager.execute_on_module(EXCHANGE, swap_msg)?;
        Ok(())
    }
}
