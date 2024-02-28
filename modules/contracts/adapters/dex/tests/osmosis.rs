#![cfg(feature = "osmosis-test")]

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
use abstract_dex_standard::ans_action::DexAnsAction;
use abstract_dex_standard::msg::DexExecuteMsg;
use abstract_interface::{
    Abstract, AbstractAccount, AbstractInterfaceError, AccountFactory, AdapterDeployer,
    DeployStrategy,
};
use abstract_osmosis_adapter::OSMOSIS;
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_orch::{interface, osmosis_test_tube::OsmosisTestTube, prelude::*};
use osmosis_pool::helpers::osmosis_pool_token;
use osmosis_pool::OsmosisPools;
use std::format;

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
            request: DexExecuteMsg::AnsAction {
                dex,
                action: DexAnsAction::Swap {
                    offer_asset: AnsAsset::new(asset, offer_asset.1),
                    ask_asset,
                    max_spread: Some(Decimal::percent(30)),
                    belief_price: None,
                },
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
            request: DexExecuteMsg::AnsAction {
                dex,
                action: DexAnsAction::ProvideLiquidity {
                    assets: vec![
                        AnsAsset::new(asset_entry1, asset1.1),
                        AnsAsset::new(asset_entry2, asset2.1),
                    ],
                    max_spread: Some(Decimal::percent(30)),
                },
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
        let lp_token = AnsAsset::new(lp_token, amount.into());

        let swap_msg = abstract_dex_adapter::msg::ExecuteMsg::Module(adapter::AdapterRequestMsg {
            proxy_address: None,
            request: DexExecuteMsg::AnsAction {
                dex,
                action: DexAnsAction::WithdrawLiquidity { lp_token },
            },
        });
        account
            .manager
            .execute_on_module(DEX_ADAPTER_ID, swap_msg)?;
        Ok(())
    }
}

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    OsmosisTestTube,
    OsmosisDexAdapter<OsmosisTestTube>,
    AbstractAccount<OsmosisTestTube>,
    Abstract<OsmosisTestTube>,
    OsmosisPools,
)> {
    let chain = OsmosisTestTube::new(vec![coin(1_000_000_000_000, "uosmo")]);

    let deployment = Abstract::deploy_on(chain.clone(), chain.sender().to_string())?;
    let osmosis_pools = OsmosisPools::deploy_on(chain.clone(), Empty {})?;

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

    let account = create_default_account(&deployment.account_factory)?;

    // install DEX_ADAPTER_ID on OS
    account.install_adapter(&dex_adapter, None)?;

    Ok((
        chain.call_as(&osmosis_pools.owner),
        dex_adapter,
        account,
        deployment,
        osmosis_pools,
    ))
}

#[test]
fn swap() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, osmosis) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(
        proxy_addr.to_string(),
        coins(swap_value, osmosis.eur_token.clone()),
    )?;

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances, coins(swap_value, osmosis.eur_token.clone()));
    // swap 100_000 uatom to uosmo
    dex_adapter.swap(
        (&osmosis.eur_token, swap_value),
        &osmosis.usd_token,
        OSMOSIS.into(),
        &os,
    )?;

    // Assert balances
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(proxy_addr.as_ref(), &osmosis.usd_token)?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn swap_concentrated_liquidity() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, deployment, osmosis) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(
        proxy_addr.to_string(),
        coins(swap_value, osmosis.eur_token.clone()),
    )?;

    let lp = "osmosis/osmo2,atom2";
    let pool_id = chain.create_pool(vec![
        coin(1_000, &osmosis.eur_token),
        coin(1_000, &osmosis.usd_token),
    ])?;

    deployment
        .ans_host
        .update_asset_addresses(
            vec![
                (
                    "osmo2".to_string(),
                    cw_asset::AssetInfoBase::native(osmosis.usd_token.clone()),
                ),
                (
                    "atom2".to_string(),
                    cw_asset::AssetInfoBase::native(osmosis.eur_token.clone()),
                ),
                (
                    lp.to_string(),
                    cw_asset::AssetInfoBase::native(osmosis_pool_token(pool_id)),
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
    assert_eq!(balances, coins(swap_value, &osmosis.eur_token));
    // swap 100_000 uatom to uosmo
    dex_adapter.swap(("atom2", swap_value), "osmo2", OSMOSIS.into(), &os)?;

    // Assert balances
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(proxy_addr.as_ref(), &osmosis.usd_token)?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn provide() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, osmosis) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the proxy
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert!(balances.is_empty());
    chain.bank_send(
        proxy_addr.to_string(),
        coins(provide_value, osmosis.axl_usd_token.clone()),
    )?;
    chain.bank_send(
        proxy_addr.to_string(),
        coins(provide_value, osmosis.usd_token.clone()),
    )?;

    // provide to the pool
    dex_adapter.provide(
        (&osmosis.axl_usd_token, provide_value),
        (&osmosis.usd_token, provide_value),
        OSMOSIS.into(),
        &os,
    )?;

    // After providing, we need to get the liquidity token
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(
        balances,
        coins(
            100_000_000_000_000_000_000_000,
            osmosis_pool_token(osmosis.usd_axl_usd_pool)
        )
    );

    Ok(())
}

#[test]
fn withdraw() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, _abstr, osmosis) = setup_mock()?;

    let proxy_addr = os.proxy.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the proxy
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert!(balances.is_empty());
    chain.bank_send(
        proxy_addr.to_string(),
        coins(provide_value, osmosis.eur_token.clone()),
    )?;
    chain.bank_send(
        proxy_addr.to_string(),
        coins(provide_value, osmosis.usd_token.clone()),
    )?;

    // provide to the pool
    dex_adapter.provide(
        (&osmosis.eur_token, provide_value),
        (&osmosis.usd_token, provide_value),
        OSMOSIS.into(),
        &os,
    )?;

    // After providing, we need to get the liquidity token
    let balance = chain.query_balance(
        proxy_addr.as_ref(),
        &osmosis_pool_token(osmosis.eur_usd_pool),
    )?;

    // withdraw from the pool
    dex_adapter.withdraw(
        &format!("osmosis/{},{}", osmosis.eur_token, osmosis.usd_token),
        balance / Uint128::from(2u128),
        OSMOSIS.into(),
        &os,
    )?;

    // After withdrawing, we should get some tokens in return and have some lp token left
    let balances = chain.query_all_balances(proxy_addr.as_ref())?;
    assert_eq!(balances.len(), 3);

    Ok(())
}
