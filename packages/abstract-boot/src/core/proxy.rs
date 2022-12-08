use abstract_sdk::os::{objects::proxy_asset::UncheckedProxyAsset, proxy::*, MANAGER, PROXY};

use crate::manager::Manager;
use boot_core::interface::ContractInstance;
use boot_core::prelude::boot_contract;
use boot_core::{BootEnvironment, BootError, Contract};

#[boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Proxy<Chain>;

impl<Chain: BootEnvironment> Proxy<Chain> {
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("proxy"), // .with_mock(Box::new(
                                                                //     ContractWrapper::new_with_empty(
                                                                //         ::contract::execute,
                                                                //         ::contract::instantiate,
                                                                //         ::contract::query,
                                                                //     ),
                                                                // ))
        )
    }

    pub fn set_proxy_asset(&self, to_add: Vec<UncheckedProxyAsset>) -> Result<(), BootError> {
        let manager = Manager::new(MANAGER, &self.get_chain());
        manager.execute_on_module(
            PROXY,
            ExecuteMsg::UpdateAssets {
                to_add,
                to_remove: vec![],
            },
        )?;
        Ok(())
    }
    // pub  fn set_vault_assets(&self, path: &str) -> Result<(), BootError> {
    //     let file = File::open(path).expect(&format!("file should be present at {}", path));
    //     let json: serde_json::Value = from_reader(file)?;
    //     let maybe_assets = json.get(self.instance().deployment.network.chain.chain_id.clone());
    //     match maybe_assets {
    //         Some(assets_value) => {
    //             let to_add: Vec<UncheckedProxyAsset> =
    //                 serde_json::from_value(assets_value.clone())?;
    //             let mut i = 0;
    //             while i != to_add.len() - 1 {
    //                 let chunk = to_add.get(i..min(i + 10, to_add.len() - 1)).unwrap();
    //                 i += chunk.len();
    //                 self.0
    //                     .execute(
    //                         &ExecuteMsg::UpdateAssets {
    //                             to_add: chunk.to_vec(),
    //                             to_remove: vec![],
    //                         },
    //                         &vec![],
    //                     )
    //                     ?;
    //             }

    //             return Ok(());
    //         }
    //         None => return Err(BootError::StdErr("network not found".into())),
    //     }
    // }

    // pub  fn fund_os(&self, coin: Coin) -> Result<(), BootError> {
    //     self.instance()
    //         .sender
    //         .bank_send(self.instance().name, vec![coin])
    //         ?;
    //     Ok(())
    // }
}
