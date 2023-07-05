

use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::PoolMetadata;
use abstract_osmosis_adapter::OSMOSIS;
use abstract_core::ans_host::ExecuteMsgFns;
use abstract_interface::AccountFactory;
use abstract_core::adapter;
use abstract_core::objects::AnsAsset;
use abstract_core::MANAGER;
use abstract_core::objects::AssetEntry;
use abstract_dex_adapter_traits::msg::DexAction;
use abstract_dex_adapter_traits::msg::DexExecuteMsg;
use abstract_interface::Manager;
use abstract_interface::AbstractInterfaceError;
use cw_orch::deploy::Deploy;
use cosmwasm_std::Decimal;
use cosmwasm_std::coin;
use abstract_dex_adapter::contract::CONTRACT_VERSION;
use abstract_interface::AbstractAccount;
use abstract_interface::Abstract;
use abstract_dex_adapter::EXCHANGE;
use abstract_dex_adapter::msg::DexInstantiateMsg;
use abstract_interface::AdapterDeployer;
use cw_orch::{prelude::*, interface};
use abstract_dex_adapter::msg::{InstantiateMsg, QueryMsg, ExecuteMsg};
use abstract_core::objects::gov_type::GovernanceDetails;
use cw_orch::test_tube::osmosis_test_tube::{OsmosisTestApp};
use anyhow::Result as AnyResult;

pub fn create_default_account<Chain: CwEnv>(
    factory: &AccountFactory<Chain>,
) -> anyhow::Result<AbstractAccount<Chain>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(factory.get_chain().sender()).to_string(),
    })?;
    Ok(os)
}

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty)]
pub struct OsmosisDexAdapter<Chain>;

// Implement deployer trait
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
        artifacts_dir_from_workspace!()
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

        let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
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
        manager.execute_on_module(EXCHANGE, swap_msg)?;
        Ok(())
    }
}

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    TestTube,
    OsmosisDexAdapter<TestTube>,
    AbstractAccount<TestTube>,
    Abstract<TestTube>,
)> {
    let atom = "uatom";
    let osmo = "uosmo";

    let chain = TestTube::new(vec![
        coin(1_000_000_000_000, osmo),
        coin(1_000_000_000_000, atom)
    ]);

    let deployment = Abstract::deploy_on(chain.clone(), Empty {})?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let dex_adapter = OsmosisDexAdapter::new(EXCHANGE, chain.clone());

    dex_adapter.deploy(
        CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
    )?;




    // We need to register some pairs and assets on the ans host contract

    let pool_id = chain.create_pool(vec![
        coin(1_000_000_000, osmo),
        coin(1_000_000_000, atom)
    ])?;

    deployment.ans_host.update_asset_addresses(
            vec![
                (
                    "atom".to_string(),
                    cw_asset::AssetInfoBase::native(atom),
                ),
                (
                    "osmo".to_string(),
                    cw_asset::AssetInfoBase::native(osmo),
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
            vec![
                (
                    PoolAddressBase::id(pool_id),
                    PoolMetadata::constant_product(OSMOSIS, vec!["osmo".to_string(), "atom".to_string()]),
                )
            ],
            vec![],
        )
        .unwrap();


    let account = create_default_account(&deployment.account_factory)?;

    // install exchange on OS
    account.manager.install_module(EXCHANGE, &Empty {}, None)?;
    // load exchange data into type
    dex_adapter.set_address(&Addr::unchecked(
        account.manager.module_info(EXCHANGE)?.unwrap().address,
    ));

    Ok((chain, dex_adapter, account, deployment))
}


use cosmwasm_std::coins;
use cw_orch::test_tube::TestTube;
#[test]
fn swap() -> AnyResult<()>{

    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;
    chain.bank_send(proxy_addr.to_string(), coins(1_000_000_000, "uatom"))?;

    // swap 100_000 uatom to uosmo
    dex_adapter.swap(("atom", 100_000), "osmo", OSMOSIS.into())?;


    // Assert balances

    Ok(())
}

