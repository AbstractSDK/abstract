#![cfg(feature = "osmosis-test")]

use std::{format, rc::Rc};

use abstract_adapter::std::{
    ans_host::ExecuteMsgFns,
    objects::{
        gov_type::GovernanceDetails, pool_id::PoolAddressBase, AnsAsset, AssetEntry, PoolMetadata,
    },
};
use abstract_dex_adapter::{
    contract::CONTRACT_VERSION,
    interface::DexAdapter,
    msg::{DexInstantiateMsg, SwapNode},
    DEX_ADAPTER_ID,
};
use abstract_dex_standard::ans_action::DexAnsAction;
use abstract_interface::{
    Abstract, AbstractInterfaceError, AccountI, AdapterDeployer, AnsHost, DeployStrategy,
};
use abstract_osmosis_adapter::OSMOSIS;
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_asset::AssetBase;
use cw_orch::prelude::*;
use cw_orch_osmosis_test_tube::{osmosis_test_tube::Account, OsmosisTestTube};

/// Provide liquidity using Abstract's OS (registered in daemon_state).
pub fn provide<Chain: CwEnv>(
    dex_adapter: &DexAdapter<Chain>,
    asset1: (&str, u128),
    asset2: (&str, u128),
    dex: String,
    os: &AccountI<Chain>,
    ans_host: &AnsHost<Chain>,
) -> Result<(), AbstractInterfaceError> {
    let asset_entry1 = AssetEntry::new(asset1.0);
    let asset_entry2 = AssetEntry::new(asset2.0);

    dex_adapter.ans_action(
        dex,
        DexAnsAction::ProvideLiquidity {
            assets: vec![
                AnsAsset::new(asset_entry1, asset1.1),
                AnsAsset::new(asset_entry2, asset2.1),
            ],
            max_spread: Some(Decimal::percent(30)),
        },
        os,
        ans_host,
    )?;
    Ok(())
}

/// Withdraw liquidity using Abstract's OS (registered in daemon_state).
pub fn withdraw<Chain: CwEnv>(
    dex_adapter: &DexAdapter<Chain>,
    lp_token: &str,
    amount: impl Into<Uint128>,
    dex: String,
    os: &AccountI<Chain>,
    ans_host: &AnsHost<Chain>,
) -> Result<(), AbstractInterfaceError> {
    let lp_token = AnsAsset::new(lp_token, amount.into());

    dex_adapter.ans_action(
        dex,
        DexAnsAction::WithdrawLiquidity { lp_token },
        os,
        ans_host,
    )?;
    Ok(())
}

fn get_pool_token(id: u64) -> String {
    format!("gamm/pool/{}", id)
}

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    OsmosisTestTube,
    DexAdapter<OsmosisTestTube>,
    AccountI<OsmosisTestTube>,
    Abstract<OsmosisTestTube>,
    u64,
)> {
    let atom = "uatom";
    let osmo = "uosmo";
    let juno = "ujunox";

    let chain = OsmosisTestTube::new(vec![
        coin(1_000_000_000_000, atom),
        coin(1_000_000_000_000, juno),
        coin(1_000_000_000_000, osmo),
    ]);

    let deployment = Abstract::deploy_on(chain.clone(), ())?;

    let _root_os = AccountI::create_default_account(
        &deployment,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string(),
        },
    )?;
    let dex_adapter = DexAdapter::new(DEX_ADAPTER_ID, chain.clone());

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

    let account = AccountI::create_default_account(
        &deployment,
        GovernanceDetails::Monarchy {
            monarch: chain.sender_addr().to_string(),
        },
    )?;

    // install DEX_ADAPTER_ID on OS
    account.install_adapter(&dex_adapter, &[])?;

    Ok((chain, dex_adapter, account, deployment, pool_id))
}

#[test]
fn swap() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, _pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(account_addr.to_string(), coins(swap_value, "uatom"))?;

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances, coins(swap_value, "uatom"));
    // swap 100_000 uatom to uosmo
    dex_adapter.ans_swap(
        ("atom", swap_value),
        "osmo",
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // Assert balances
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(&account_addr, "uosmo")?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn swap_concentrated_liquidity() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, deployment, _pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(account_addr.to_string(), coins(swap_value, "uatom"))?;

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
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances, coins(swap_value, "uatom"));
    // swap 100_000 uatom to uosmo
    dex_adapter.ans_swap(
        ("atom2", swap_value),
        "osmo2",
        OSMOSIS.into(),
        &os,
        &deployment.ans_host,
    )?;

    // Assert balances
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(&account_addr, "uosmo")?;
    assert!(balance > Uint128::zero());

    Ok(())
}

#[test]
fn provide_liquidity_two_sided() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the account
    let balances = chain.query_all_balances(&account_addr)?;
    assert!(balances.is_empty());
    chain.bank_send(account_addr.to_string(), coins(provide_value * 2, "uatom"))?;
    chain.bank_send(account_addr.to_string(), coins(provide_value * 2, "uosmo"))?;

    // provide to the pool
    provide(
        &dex_adapter,
        ("atom", provide_value),
        ("osmo", provide_value),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // provide to the pool reversed
    provide(
        &dex_adapter,
        // reversed denoms
        ("osmo", provide_value),
        ("atom", provide_value),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // After providing, we need to get the liquidity token
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(
        balances,
        coins(
            10_000_000_000_000_000_000 + 9_999_999_999_999_999_990,
            get_pool_token(pool_id)
        )
    );

    Ok(())
}

#[test]
fn provide_liquidity_one_sided() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the account
    let balances = chain.query_all_balances(&account_addr)?;
    assert!(balances.is_empty());
    chain.bank_send(account_addr.to_string(), coins(provide_value, "uatom"))?;
    chain.bank_send(account_addr.to_string(), coins(provide_value, "uosmo"))?;

    // provide to the pool
    provide(
        &dex_adapter,
        ("atom", provide_value),
        ("osmo", 0),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // provide to the pool reversed
    provide(
        &dex_adapter,
        // reversed denoms
        ("osmo", provide_value),
        ("atom", 0),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // After providing, we need to get the liquidity token
    let balances = chain.query_all_balances(&account_addr)?;
    let lp_balance = balances
        .iter()
        .find(|c| c.denom == get_pool_token(pool_id))
        .unwrap();
    assert!(lp_balance.amount.u128() > 9_000_000_000_000_000_000);

    Ok(())
}

#[test]
fn withdraw_liquidity() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let provide_value = 1_000_000_000u128;

    // Before providing, we need to have no assets in the account
    let balances = chain.query_all_balances(&account_addr)?;
    assert!(balances.is_empty());
    chain.bank_send(account_addr.to_string(), coins(provide_value, "uatom"))?;
    chain.bank_send(account_addr.to_string(), coins(provide_value, "uosmo"))?;

    // provide to the pool
    provide(
        &dex_adapter,
        ("atom", provide_value),
        ("osmo", provide_value),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // After providing, we need to get the liquidity token
    let balance = chain.query_balance(&account_addr, &get_pool_token(pool_id))?;

    // withdraw from the pool
    withdraw(
        &dex_adapter,
        "osmosis/atom,osmo",
        balance / Uint128::from(2u128),
        OSMOSIS.into(),
        &os,
        &abstr.ans_host,
    )?;

    // After withdrawing, we should get some tokens in return and have some lp token left
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances.len(), 3);

    Ok(())
}

#[test]
fn swap_route() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, pool_id_atom_osmo) = setup_mock()?;
    let juno = "ujunox";
    let osmo = "uosmo";

    let pool_id_osmo_juno =
        chain.create_pool(vec![coin(10_000_000_000, osmo), coin(10_000_000_000, juno)])?;

    abstr
        .ans_host
        .update_asset_addresses(
            vec![
                ("osmo".to_string(), cw_asset::AssetInfoBase::native(osmo)),
                ("juno".to_string(), cw_asset::AssetInfoBase::native(juno)),
                (
                    "osmosis/osmo,juno".to_string(),
                    cw_asset::AssetInfoBase::native(get_pool_token(pool_id_osmo_juno)),
                ),
            ],
            vec![],
        )
        .unwrap();

    abstr
        .ans_host
        .update_pools(
            vec![(
                PoolAddressBase::id(pool_id_osmo_juno),
                PoolMetadata::constant_product(
                    OSMOSIS,
                    vec!["osmo".to_string(), "juno".to_string()],
                ),
            )],
            vec![],
        )
        .unwrap();

    let account_addr = os.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(account_addr.to_string(), coins(swap_value, "uatom"))?;

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances, coins(swap_value, "uatom"));
    // swap 100_000 uatom to uosmo
    dex_adapter.raw_action(
        OSMOSIS.to_string(),
        abstract_dex_adapter::msg::DexAction::RouteSwap {
            route: vec![
                SwapNode {
                    pool_id: PoolAddressBase::Id(pool_id_atom_osmo),
                    ask_asset: cw_asset::AssetInfoBase::Native("uosmo".to_owned()),
                },
                SwapNode {
                    pool_id: PoolAddressBase::Id(pool_id_osmo_juno),
                    ask_asset: cw_asset::AssetInfoBase::Native(juno.to_owned()),
                },
            ],
            offer_asset: AssetBase::native("uatom", swap_value),
            max_spread: None,
            belief_price: None,
        },
        &os,
    )?;

    // Assert balances
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(&account_addr, juno)?;
    assert!(balance > Uint128::zero());

    Ok(())
}
