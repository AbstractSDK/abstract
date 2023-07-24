mod common;

use abstract_cw_staking::contract::CONTRACT_VERSION;
use abstract_cw_staking::interface::CwStakingAdapter;
use abstract_cw_staking::msg::StakingQueryMsgFns;
use abstract_interface::Abstract;
use abstract_interface::AbstractAccount;
use abstract_interface::AdapterDeployer;
use cosmwasm_std::coins;

use abstract_core::objects::{AnsAsset, AssetEntry};
use cw_orch::contract::Deploy;

use abstract_staking_adapter_traits::msg::{
    Claim, RewardTokensResponse, StakingInfoResponse, UnbondingResponse,
};
use cosmwasm_std::{coin, Addr, Empty, Uint128};
use cw_asset::AssetInfoBase;
use cw_orch::prelude::*;
use speculoos::*;

const OSMOSIS: &str = "cosmos-testnet>osmosis";

const ASSET_1: &str = "eur";
const ASSET_2: &str = "usd";

pub const EUR_USD_LP: &str = "osmosis/eur,usd";

const DENOM: &str = "uosmo";

use abstract_cw_staking::CW_STAKING;
use common::create_default_account;

fn setup_osmosis() -> anyhow::Result<(
    OsmosisTestTube,
    u64,
    CwStakingAdapter<OsmosisTestTube>,
    AbstractAccount<OsmosisTestTube>,
)> {
    let tube = OsmosisTestTube::new(coins(104_860_310_000, DENOM));

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
    // wyndex
    //     .eur_usd_lp
    //     .call_as(&Addr::unchecked(WYNDEX_OWNER))
    //     .transfer(1000u128.into(), proxy_addr.to_string())?;

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
    let (tube, pool_id, staking, _) = setup_osmosis()?;

    // query staking info
    let staking_info = staking.info(OSMOSIS.into(), AssetEntry::new(EUR_USD_LP))?;
    println!("staking_info: {staking_info:?}");
    // assert_that!(staking_info).is_equal_to(StakingInfoResponse {
    //     staking_contract_address: wyndex.eur_usd_staking,
    //     staking_token: AssetInfoBase::Cw20(wyndex.eur_usd_lp.address()?),
    //     unbonding_periods: Some(vec![
    //         cw_utils::Duration::Time(1),
    //         cw_utils::Duration::Time(2),
    //     ]),
    //     max_claims: None,
    // });

    // query reward tokens
    let reward_tokens = staking.reward_tokens(OSMOSIS.into(), AssetEntry::new(EUR_USD_LP))?;
    // assert_that!(reward_tokens).is_equal_to(RewardTokensResponse {
    //     tokens: vec![AssetInfoBase::Native(WYND_TOKEN.to_owned())],
    // });

    Ok(())
}
