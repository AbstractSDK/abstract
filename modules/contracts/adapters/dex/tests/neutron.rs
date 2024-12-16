#![cfg(feature = "neutron-test")]

use abstract_adapter::std::{
    ans_host::ExecuteMsgFns,
    objects::{
        gov_type::GovernanceDetails, pool_id::PoolAddressBase, AnsAsset, AssetEntry, PoolMetadata,
    },
};
use abstract_dex_adapter::{
    contract::CONTRACT_VERSION, interface::DexAdapter, msg::DexInstantiateMsg, DEX_ADAPTER_ID,
};
use abstract_dex_standard::ans_action::DexAnsAction;
use abstract_interface::{
    Abstract, AbstractInterfaceError, AccountI, AdapterDeployer, AnsHost, DeployStrategy,
};
use abstract_neutron_dex_adapter::NEUTRON;
use anyhow::Result as AnyResult;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_orch::prelude::*;
use cw_orch_neutron_test_tube::{
    neutron_test_tube::{
        neutron_std::types::neutron::dex::{DepositOptions, MsgDeposit},
        Dex, Module,
    },
    NeutronTestTube,
};

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

pub const NTRN: &str = "untrn";
pub const ATOM: &str = "uatom";
pub const STARS: &str = "ustars";

#[allow(clippy::type_complexity)]
fn setup_mock() -> anyhow::Result<(
    NeutronTestTube,
    DexAdapter<NeutronTestTube>,
    AccountI<NeutronTestTube>,
    Abstract<NeutronTestTube>,
    u64,
)> {
    let chain = NeutronTestTube::new(vec![
        coin(1_000_000_000_000, NTRN),
        coin(1_000_000_000_000, ATOM),
        coin(1_000_000_000_000, STARS),
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

    // Deposit some inital liquidity
    let pool_create_msg = MsgDeposit {
        token_a: NTRN.to_string(),
        token_b: ATOM.to_string(),
        amounts_a: vec!["100000000000".to_string()],
        amounts_b: vec!["100000000000".to_string()],
        creator: chain.sender_addr().to_string(),
        receiver: chain.sender_addr().to_string(),
        fees: vec![0],
        options: vec![DepositOptions {
            disable_autoswap: false,
            fail_tx_on_bel: false,
        }],
        tick_indexes_a_to_b: vec![0],
    };

    let app = chain.app.borrow_mut();
    let dex = Dex::new(&*app);
    dex.deposit(pool_create_msg, chain.sender())?;
    drop(app);

    // We need to register some pairs and assets on the ans host contract
    // Register NTRN and ATOM assets
    deployment
        .ans_host
        .update_asset_addresses(
            vec![
                ("ntrn".to_string(), cw_asset::AssetInfoBase::native(NTRN)),
                ("atom".to_string(), cw_asset::AssetInfoBase::native(ATOM)),
            ],
            vec![],
        )
        .unwrap();
    deployment
        .ans_host
        .update_dexes(vec![NEUTRON.into()], vec![])
        .unwrap();

    deployment
        .ans_host
        .update_pools(
            vec![(
                PoolAddressBase::id(0u64),
                PoolMetadata::constant_product(
                    NEUTRON,
                    vec!["ntrn".to_string(), "atom".to_string()],
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

    Ok((chain.clone(), dex_adapter, account, deployment, 0))
}

#[test]
fn swap() -> AnyResult<()> {
    // We need to deploy a Testube pool
    let (chain, dex_adapter, os, abstr, _pool_id) = setup_mock()?;

    let account_addr = os.address()?;

    let swap_value = 1_000_000_000u128;

    chain.bank_send(&account_addr, coins(swap_value, NTRN))?;

    // Before swap, we need to have 0 uosmo and swap_value uatom
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances, coins(swap_value, NTRN));
    // swap 100_000 ntrn to atom
    let asset = AssetEntry::new("ntrn");
    let ask_asset = AssetEntry::new("atom");

    let action = DexAnsAction::Swap {
        offer_asset: AnsAsset::new(asset, swap_value),
        ask_asset,
        max_spread: Some(Decimal::percent(30)),
        belief_price: Some(Decimal::percent(1)),
    };
    dex_adapter.ans_action(NEUTRON.into(), action, &os, &abstr.ans_host)?;

    // Assert balances
    let balances = chain.query_all_balances(&account_addr)?;
    assert_eq!(balances.len(), 1);
    let balance = chain.query_balance(&account_addr, ATOM)?;
    assert!(balance > Uint128::zero());

    Ok(())
}
