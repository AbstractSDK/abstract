#![cfg(feature = "osmosis-test")]

use std::format;

use abstract_core::{
    adapter,
    ans_host::ExecuteMsgFns,
    objects::{
        gov_type::GovernanceDetails, pool_id::PoolAddressBase, AnsAsset, AssetEntry, PoolMetadata,
    },
};
use abstract_dex_adapter::{
    contract::CONTRACT_VERSION,
    msg::{DexInstantiateMsg, ExecuteMsg, InstantiateMsg, QueryMsg},
    DEX_ADAPTER_ID,
};
use abstract_dex_standard::msg::{DexAction, DexExecuteMsg};
use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountFactory, AdapterDeployer,
    DeployStrategy,
};
use abstract_osmosis_adapter::OSMOSIS;
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_orch::{deploy::Deploy, interface, osmosis_test_tube::OsmosisTestTube, prelude::*};

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
        account: &AbstractAccount<Chain>,
    ) -> Result<(), AbstractInterfaceError> {
        let asset = AssetEntry::new(offer_asset.0);
        let ask_asset = AssetEntry::new(ask_asset);

        let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: DexExecuteMsg::Action {
                dex,
                action: DexAction::Swap {
                    offer_asset: AnsAsset::new(asset, offer_asset.1).into(),
                    ask_asset: ask_asset.into(),
                    max_spread: Some(Decimal::percent(30)),
                    belief_price: None,
                },
                pool: None,
            },
        });
        account
            .manager
            .execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
        Ok(())
    }
    /// Provide liquidity using Abstract's OS (registered in daemon_state).
    pub fn provide(
        &self,
        asset1: (&str, u128),
        asset2: (&str, u128),
        dex: String,
        account: &AbstractAccount<Chain>,
    ) -> Result<(), AbstractInterfaceError> {
        let asset_entry1 = AssetEntry::new(asset1.0);
        let asset_entry2 = AssetEntry::new(asset2.0);

        let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: DexExecuteMsg::Action {
                dex,
                action: DexAction::ProvideLiquidity {
                    assets: vec![
                        AnsAsset::new(asset_entry1, asset1.1).into(),
                        AnsAsset::new(asset_entry2, asset2.1).into(),
                    ],
                    max_spread: Some(Decimal::percent(30)),
                },
                pool: None,
            },
        });
        account
            .manager
            .execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
        Ok(())
    }

    /// Withdraw liquidity using Abstract's OS (registered in daemon_state).
    pub fn withdraw(
        &self,
        lp_token: &str,
        amount: impl Into<Uint128>,
        dex: String,
        account: &AbstractAccount<Chain>,
    ) -> Result<(), AbstractInterfaceError> {
        let lp_token = AnsAsset::new(lp_token, amount.into()).into();

        let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: DexExecuteMsg::Action {
                dex,
                action: DexAction::WithdrawLiquidity { lp_token },
                pool: None,
            },
        });
        account
            .manager
            .execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
        Ok(())
    }
}

fn get_pool_token(id: u64) -> String {
    format!("gamm/pool/{}", id)
}

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    OsmosisTestTube,
    OsmosisDexAdapter<OsmosisTestTube>,
    AbstractAccount<OsmosisTestTube>,
    Abstract<OsmosisTestTube>,
    u64,
)> {
    let atom = "uatom";
    let osmo = "uosmo";

    let chain = OsmosisTestTube::new(vec![
        coin(1_000_000_000_000, osmo),
        coin(1_000_000_000_000, atom),
    ]);

    let deployment = Abstract::deploy_on(chain.clone(), chain.sender().to_string())?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let dex_adapter = OsmosisDexAdapter::new(DEX_ADAPTER_ID, chain.clone());

    dex_adapter.deploy(
        CONTRACT_VERSION.parse()?,
        DexInstantiateMsg {
            swap_fee: Decimal::percent(1),
            recipient_account: 0,
        },
        DeployStrategy::Try,
    )?;

    // We need to register some pairs and assets on the ans host contract

    let pool_id =
        chain.create_pool(vec![coin(10_000_000_000, osmo), coin(10_000_000_000, atom)])?;

    deployment
        .ans_host
        .update_asset_addresses(
            vec![
                ("atom".to_string(), cw_asset::AssetInfoBase::native(atom)),
                ("osmo".to_string(), cw_asset::AssetInfoBase::native(osmo)),
                (
                    "osmosis/atom,osmo".to_string(),
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

    let account = create_default_account(&deployment.account_factory)?;

    // install DEX_ADAPTER_ID on OS
    account.install_adapter(&dex_adapter, None)?;

    Ok((chain, dex_adapter, account, deployment, pool_id))
}

#[test]
fn swap() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, _pool_id) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(proxy_addr.to_string(), coins(swap_value, "uatom"))?;

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances, coins(swap_value, "uatom"));
    // swap 100_000 uatom to uosmo
    dex_adapter.swap(("atom", swap_value), "osmo", OSMOSIS.into(), &os)?;

    // Assert balances
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(proxy_addr.as_ref(), "uosmo")?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn swap_concentrated_liquidity() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, deployment, _pool_id) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(proxy_addr.to_string(), coins(swap_value, "uatom"))?;

    let lp = "osmosis/osmo2,atom2";
    let pool_id = chain.create_pool(vec![coin(1_000, "uosmo"), coin(1_000, "uatom")])?;

    deployment
        .ans_host
        .update_asset_addresses(
            vec![
                (
                    "osmo2".to_string(),
                    cw_asset::AssetInfoBase::native("uosmo"),
                ),
                (
                    "atom2".to_string(),
                    cw_asset::AssetInfoBase::native("uatom"),
                ),
                (
                    lp.to_string(),
                    cw_asset::AssetInfoBase::native(get_pool_token(pool_id)),
                ),
            ],
            vec![],
        )
        .unwrap();
    deployment
        .ans_host
        .update_pools(
            vec![(
                PoolAddressBase::id(pool_id),
                PoolMetadata::concentrated_liquidity(
                    OSMOSIS,
                    vec!["osmo2".to_string(), "atom2".to_string()],
                ),
            )],
            vec![],
        )
        .unwrap();

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances, coins(swap_value, "uatom"));
    // swap 100_000 uatom to uosmo
    dex_adapter.swap(("atom2", swap_value), "osmo2", OSMOSIS.into(), &os)?;

    // Assert balances
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(proxy_addr.as_ref(), "uosmo")?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn provide() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, pool_id) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the proxy
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert!(balances.is_empty());
    chain.bank_send(proxy_addr.to_string(), coins(provide_value, "uatom"))?;
    chain.bank_send(proxy_addr.to_string(), coins(provide_value, "uosmo"))?;

    // provide to the pool
    dex_adapter.provide(
        ("atom", provide_value),
        ("osmo", provide_value),
        OSMOSIS.into(),
        &os,
    )?;

    // After providing, we need to get the liquidity token
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    println!("{:?}", balances);
    assert_eq!(
        balances,
        coins(10_000_000_000_000_000_000, get_pool_token(pool_id))
    );

    Ok(())
}

#[test]
fn withdraw() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, pool_id) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the proxy
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert!(balances.is_empty());
    chain.bank_send(proxy_addr.to_string(), coins(provide_value, "uatom"))?;
    chain.bank_send(proxy_addr.to_string(), coins(provide_value, "uosmo"))?;

    // provide to the pool
    dex_adapter.provide(
        ("atom", provide_value),
        ("osmo", provide_value),
        OSMOSIS.into(),
        &os,
    )?;

    // After providing, we need to get the liquidity token
    let balance = chain.query_balance(proxy_addr.as_ref(), &get_pool_token(pool_id))?;

    // withdraw from the pool
    dex_adapter.withdraw(
        "osmosis/atom,osmo",
        balance / Uint128::from(2u128),
        OSMOSIS.into(),
        &os,
    )?;

    // After withdrawing, we should get some tokens in return and have some lp token left
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    println!("{:?}", balances);
    assert_eq!(balances.len(), 3);

    Ok(())
}
