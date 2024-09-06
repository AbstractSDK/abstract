pub use abstract_std::proxy::{ExecuteMsgFns as ProxyExecFns, QueryMsgFns as ProxyQueryFns};
use abstract_std::{objects::AccountId, proxy::*, ACCOUNT};
use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
pub struct Proxy<Chain>;

impl<Chain: CwEnv> Proxy<Chain> {
    pub(crate) fn new_from_id(account_id: &AccountId, chain: Chain) -> Self {
        let proxy_id = format!("{ACCOUNT}-{account_id}");
        Self::new(proxy_id, chain)
    }
}

impl<Chain: CwEnv> Uploadable for Proxy<Chain> {
    #[cfg(feature = "integration")]
    fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
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
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("proxy")
            .unwrap()
    }
}

impl<Chain: CwEnv> Proxy<Chain> {
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
