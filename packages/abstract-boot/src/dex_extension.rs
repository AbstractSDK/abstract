use abstract_sdk::os::{
    base,
    dex::*,
    extension::*,
    objects::{AnsAsset, AssetEntry},
    EXCHANGE, MANAGER,
};
use boot_core::{BootError, Contract, IndexResponse, TxHandler, TxResponse};
use cosmwasm_std::Empty;

use crate::{manager::Manager, AbstractOS};

pub type DexExtension<Chain> = AbstractOS<
    Chain,
    ExecuteMsg<DexRequestMsg>,
    base::InstantiateMsg<BaseInstantiateMsg>,
    abstract_sdk::os::extension::QueryMsg<abstract_sdk::os::dex::DexQueryMsg>,
    Empty,
>;

impl<Chain: TxHandler + Clone> DexExtension<Chain>
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
            ExecuteMsg::<DexRequestMsg>::App(ExtensionRequestMsg {
                proxy_address: None,
                request: DexRequestMsg {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
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
