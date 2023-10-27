#![allow(unused)]
mod common;
// Keep this until https://github.com/osmosis-labs/test-tube/issues/28 is fixed!
#[cfg(feature = "osmosis-test")]
mod osmosis_test {

    use std::path::PathBuf;

    use abstract_core::adapter;
    use abstract_core::ans_host::ExecuteMsgFns;
    use abstract_core::objects::pool_id::PoolAddressBase;
    use abstract_core::objects::PoolMetadata;
    use abstract_core::objects::ABSTRACT_ACCOUNT_ID;
    use abstract_core::MANAGER;
    use abstract_dex_adapter::contract::CONTRACT_VERSION;
    use abstract_dex_adapter::msg::DexQueryMsgFns;
    use abstract_dex_standard::msg::DexInstantiateMsg;
    use abstract_dex_standard::DexError;
    use abstract_interface::Abstract;
    use abstract_interface::AbstractAccount;
    use abstract_interface::AbstractInterfaceError;
    use abstract_interface::AdapterDeployer;
    use abstract_interface::DeployStrategy;
    use abstract_interface::Manager;
    use cosmwasm_std::coins;
    use cosmwasm_std::Decimal;

    use abstract_dex_adapter::msg::{
        DexAction, DexExecuteMsg, ExecuteMsg, InstantiateMsg, QueryMsg,
    };

    use abstract_core::objects::{AnsAsset, AssetEntry};
    use cw_orch::deploy::Deploy;

    use cosmwasm_std::{coin, Addr, Empty, Uint128};
    use cw_asset::AssetInfoBase;
    use cw_orch::interface;
    use cw_orch::osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::lockup::AccountLockedCoinsRequest;
    use cw_orch::osmosis_test_tube::osmosis_test_tube::osmosis_std::types::osmosis::lockup::AccountLockedCoinsResponse;
    use cw_orch::osmosis_test_tube::osmosis_test_tube::Runner;
    use cw_orch::prelude::*;
    use speculoos::prelude::*;

    const OSMOSIS: &str = "osmosis";
    const DENOM: &str = "uosmo";

    const UOSMO: &str = DENOM;
    const UATOM: &str = "uatom";

    pub const LP: &str = "osmosis/osmo,atom";

    use crate::common::create_default_account;
    use abstract_dex_adapter::DEX_ADAPTER_ID;

    fn get_pool_token(id: u64) -> String {
        format!("gamm/pool/{}", id)
    }

    #[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
    pub struct OsmosisDexAdapter<Chain>;

    impl<Chain: CwEnv> AdapterDeployer<Chain, DexInstantiateMsg> for OsmosisDexAdapter<Chain> {}

    impl<Chain: CwEnv> Uploadable for OsmosisDexAdapter<Chain> {
        fn wrapper(&self) -> <Mock as TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                abstract_dex_adapter::contract::execute,
                abstract_dex_adapter::contract::instantiate,
                abstract_dex_adapter::contract::query,
            ))
        }
        fn wasm(&self) -> WasmPath {
            let mut artifacts_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            artifacts_path.push("../../../artifacts");

            ArtifactsDir::new(artifacts_path)
                .find_wasm_path("abstract_dex_adapter-osmosis")
                .unwrap()
        }
    }

    impl<Chain: CwEnv> OsmosisDexAdapter<Chain> {
        /// Swap using Abstract's OS (registered in daemon_state).
        pub fn swap(
            &self,
            offer_asset: (&str, u128),
            ask_asset: &str,
            dex: String,
        ) -> Result<(), AbstractInterfaceError> {
            let manager = Manager::new(MANAGER, self.get_chain().clone());
            let asset = AssetEntry::new(offer_asset.0);
            let ask_asset = AssetEntry::new(ask_asset);

            let swap_msg = ExecuteMsg::Module(adapter::AdapterRequestMsg {
                proxy_address: None,
                request: DexExecuteMsg::Action {
                    dex,
                    action: DexAction::Swap {
                        offer_asset: AnsAsset::new(asset, offer_asset.1),
                        ask_asset,
                        max_spread: Some(Decimal::percent(30)),
                        belief_price: None,
                    },
                },
            });
            manager.execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
            Ok(())
        }
    }

    fn setup_osmosis() -> anyhow::Result<(
        OsmosisTestTube,
        u64,
        OsmosisDexAdapter<OsmosisTestTube>,
        AbstractAccount<OsmosisTestTube>,
    )> {
        let tube = OsmosisTestTube::new(vec![
            coin(1_000_000_000_000, UOSMO),
            coin(1_000_000_000_000, UATOM),
        ]);

        let deployment = Abstract::deploy_on(tube.clone(), tube.sender().to_string())?;

        let _root_os = create_default_account(&deployment.account_factory)?;
        let dex: OsmosisDexAdapter<OsmosisTestTube> =
            OsmosisDexAdapter::new(DEX_ADAPTER_ID, tube.clone());

        dex.deploy(
            CONTRACT_VERSION.parse()?,
            DexInstantiateMsg {
                swap_fee: Decimal::percent(1),
                recipient_account: ABSTRACT_ACCOUNT_ID.seq(),
            },
            DeployStrategy::Error,
        )?;

        let os = create_default_account(&deployment.account_factory)?;
        // let proxy_addr = os.proxy.address()?;
        let _manager_addr = os.manager.address()?;

        // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
        let pool_id = tube.create_pool(vec![coin(1_000, UOSMO), coin(1_000, UATOM)])?;

        deployment
            .ans_host
            .update_asset_addresses(
                vec![
                    ("osmo".to_string(), cw_asset::AssetInfoBase::native(UOSMO)),
                    ("atom".to_string(), cw_asset::AssetInfoBase::native(UATOM)),
                    (
                        LP.to_string(),
                        cw_asset::AssetInfoBase::native(get_pool_token(pool_id)),
                    ),
                ],
                vec![],
            )
            .unwrap();

        deployment
            .ans_host
            .update_dexes(vec![OSMOSIS.into()], vec![])
            .unwrap();

        deployment
            .ans_host
            .update_pools(
                vec![(
                    PoolAddressBase::id(pool_id),
                    PoolMetadata::constant_product(
                        OSMOSIS,
                        vec!["osmo".to_string(), "atom".to_string()],
                    ),
                )],
                vec![],
            )
            .unwrap();

        // install exchange on AbstractAccount
        os.manager.install_module(DEX_ADAPTER_ID, &Empty {}, None)?;
        // load exchange data into type
        dex.set_address(&Addr::unchecked(
            os.manager.module_info(DEX_ADAPTER_ID)?.unwrap().address,
        ));

        tube.bank_send(os.proxy.addr_str()?, coins(10_000u128, UOSMO))?;

        Ok((tube, pool_id, dex, os))
    }

    #[test]
    fn swap_native() -> anyhow::Result<()> {
        let (chain, pool_id, dex_adapter, os) = setup_osmosis()?;
        let proxy_addr = os.proxy.addr_str()?;

        dex_adapter.swap(("osmo", 100), "atom", OSMOSIS.to_owned())?;

        // check balances
        let uosmo_balance = chain.query_balance(&proxy_addr, UOSMO)?;
        assert_that!(uosmo_balance.u128()).is_equal_to(9_900);

        let uatom_balance = chain.query_balance(&proxy_addr, UATOM)?;
        assert_that!(uatom_balance.u128()).is_equal_to(89);

        Ok(())
    }
}
