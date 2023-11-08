use crate::Manager;
pub use abstract_core::proxy::{ExecuteMsgFns as ProxyExecFns, QueryMsgFns as ProxyQueryFns};
use abstract_core::{
    objects::{price_source::UncheckedPriceSource, AssetEntry},
    proxy::*,
    MANAGER, PROXY,
};

use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Proxy<Chain>;

impl<Chain: CwEnv> Uploadable for Proxy<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
        Box::new(
            ContractWrapper::new_with_empty(
                ::proxy::contract::execute,
                ::proxy::contract::instantiate,
                ::proxy::contract::query,
            )
            .with_migrate(::proxy::contract::migrate)
            .with_reply(::proxy::contract::reply),
        )
    }
    fn wasm(&self) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("proxy")
            .unwrap()
    }
}

impl<Chain: CwEnv> Proxy<Chain> {
    pub fn set_proxy_asset(
        &self,
        to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
    ) -> Result<(), crate::AbstractInterfaceError> {
        let manager = Manager::new(MANAGER, self.get_chain().clone());
        manager.execute_on_module(
            PROXY,
            ExecuteMsg::UpdateAssets {
                to_add,
                to_remove: vec![],
            },
        )?;
        Ok(())
    }
    // pub  fn set_vault_assets(&self, path: &str) -> Result<(), crate::AbstractBootError> {
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
    //         None => return Err(CwOrchError::StdErr("network not found".into())),
    //     }
    // }

    // pub  fn fund_os(&self, coin: Coin) -> Result<(), crate::AbstractBootError> {
    //     self.instance()
    //         .sender
    //         .bank_send(self.instance().name, vec![coin])
    //         ?;
    //     Ok(())
    // }
}
