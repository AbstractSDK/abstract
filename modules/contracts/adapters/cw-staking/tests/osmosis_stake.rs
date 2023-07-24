mod common;

use abstract_core::adapter;
use abstract_core::ans_host::ExecuteMsgFns;
use abstract_core::ans_host::QueryMsgFns;
use abstract_core::objects::pool_id::PoolAddressBase;
use abstract_core::objects::PoolMetadata;
use abstract_cw_staking::contract::CONTRACT_VERSION;
use abstract_cw_staking::interface::CwStakingAdapter;
use abstract_cw_staking::msg::StakingQueryMsgFns;
use abstract_interface::Abstract;
use abstract_interface::AbstractAccount;
use abstract_interface::AdapterDeployer;
use abstract_staking_adapter_traits::msg::StakingAction;
use abstract_staking_adapter_traits::msg::StakingExecuteMsg;
use cosmwasm_std::coins;

use abstract_core::objects::{AnsAsset, AssetEntry};
use cw_orch::contract::Deploy;

use abstract_staking_adapter_traits::msg::{
    Claim, RewardTokensResponse, StakingInfoResponse, UnbondingResponse,
};
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cw_asset::AssetInfoBase;
use cw_orch::prelude::networks::parse_network;
use cw_orch::prelude::*;
use speculoos::*;

const OSMOSIS: &str = "osmosis";
const DENOM: &str = "uosmo";

const ASSET_1: &str = DENOM;
const ASSET_2: &str = "uatom";

pub const LP: &str = "osmosis/osmo,atom";

use abstract_cw_staking::CW_STAKING;
use common::create_default_account;

fn get_pool_token(id: u64) -> String {
    format!("gamm/pool/{}", id)
}

fn setup_osmosis() -> anyhow::Result<(
    OsmosisTestTube,
    u64,
    CwStakingAdapter<OsmosisTestTube>,
    AbstractAccount<OsmosisTestTube>,
)> {
    let tube = OsmosisTestTube::new(vec![
        coin(1_000_000_000_000, ASSET_1),
        coin(1_000_000_000_000, ASSET_2),
    ]);

    let sender = tube.sender();
    let deployment = Abstract::deploy_on(tube.clone(), sender.to_string())?;

    let _root_os = create_default_account(&deployment.account_factory)?;
    let staking = CwStakingAdapter::new(CW_STAKING, tube.clone());

    staking.deploy(CONTRACT_VERSION.parse()?, Empty {})?;

    let os = create_default_account(&deployment.account_factory)?;
    // let proxy_addr = os.proxy.address()?;
    let _manager_addr = os.manager.address()?;

    // transfer some LP tokens to the AbstractAccount, as if it provided liquidity
    let pool_id = tube.create_pool(vec![coin(1_000, ASSET_1), coin(1_000, ASSET_2)])?;

    deployment
        .ans_host
        .update_asset_addresses(
            vec![
                ("osmo".to_string(), cw_asset::AssetInfoBase::native(ASSET_1)),
                ("atom".to_string(), cw_asset::AssetInfoBase::native(ASSET_2)),
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
    os.manager.install_module(CW_STAKING, &Empty {}, None)?;
    // load exchange data into type
    staking.set_address(&Addr::unchecked(
        os.manager.module_info(CW_STAKING)?.unwrap().address,
    ));

    Ok((tube, pool_id, staking, os))
}

#[test]
fn staking_inited() -> anyhow::Result<()> {
    let (_, pool_id, staking, _) = setup_osmosis()?;

    // query staking info
    let staking_info = staking.info(OSMOSIS.into(), AssetEntry::new(LP))?;
    let staking_coin = AssetInfoBase::native(get_pool_token(pool_id));
    assert_that!(staking_info).is_equal_to(StakingInfoResponse {
        staking_contract_address: Addr::unchecked(""),
        staking_token: staking_coin.clone(),
        unbonding_periods: Some(vec![]),
        max_claims: None,
    });

    // query reward tokens
    let reward_tokens = staking.reward_tokens(OSMOSIS.into(), AssetEntry::new(LP))?;
    assert_that!(reward_tokens).is_equal_to(RewardTokensResponse { tokens: vec![] });

    Ok(())
}

#[test]
fn stake_lp() -> anyhow::Result<()> {
    let (tube, _, staking, os) = setup_osmosis()?;
    let proxy_addr = os.proxy.address()?;

    let dur = Some(cw_utils::Duration::Time(2));

    // stake 100 atom
    staking.stake(AnsAsset::new(LP, 100u128), OSMOSIS.into(), dur)?;

    // query stake
    let staked_balance = staking.staked(
        OSMOSIS.into(),
        proxy_addr.to_string(),
        AssetEntry::new(LP),
        dur,
    )?;
    assert_that!(staked_balance.amount.u128()).is_equal_to(100u128);

    Ok(())
}
