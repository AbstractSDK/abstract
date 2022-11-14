use abstract_os::api::*;
use abstract_os::dex::*;
use abstract_os::middleware;
use abstract_os::objects::AssetEntry;
use abstract_os::EXCHANGE;
use abstract_os::MANAGER;
use boot_core::BootError;
use boot_core::{Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;
use cosmwasm_std::Uint128;

use crate::manager::Manager;
use crate::AbstractOS;

pub type DexApi<Chain> = AbstractOS<
    Chain,
    ExecuteMsg<DexRequestMsg>,
    middleware::InstantiateMsg<BaseInstantiateMsg>,
    abstract_os::api::QueryMsg<abstract_os::dex::DexQueryMsg>,
    Empty,
>;

impl<Chain: TxHandler + Clone> DexApi<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
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

    pub fn swap(
        &self,
        offer_asset: (&str, u128),
        ask_asset: &str,
        dex: String,
    ) -> Result<(), BootError> {
        let manager = Manager::new(MANAGER, &self.chain());
        let asset = AssetEntry::new(offer_asset.0);
        let ask_asset = AssetEntry::new(ask_asset);
        manager.execute_on_module(
            EXCHANGE,
            ExecuteMsg::<DexRequestMsg>::App(ApiRequestMsg {
                proxy_address: None,
                request: DexRequestMsg {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: (asset, Uint128::new(offer_asset.1)),
                        ask_asset,
                        max_spread: None,
                        belief_price: None,
                    },
                },
            }),
        )?;
        Ok(())
    }
}
