//! Currently you can run only 1 test at a time: `cargo ct`
//! Otherwise you will have too many requests

use abstract_app::objects::{
    pool_id::PoolAddressBase, AssetEntry, LpToken, PoolMetadata, PoolType, UncheckedContractEntry,
};
use abstract_client::{AbstractClient, Environment};
use abstract_cw_staking::staking_tester::{MockStaking, StakingTester};
use abstract_interface::ExecuteMsgFns;
use abstract_modules_interchain_tests::common::load_abstr;
use anyhow::Ok;
use cosmwasm_std::{coin, coins, Decimal, Uint128};
use cw_asset::AssetInfoUnchecked;
use cw_orch::daemon::networks::HARPOON_4;
use cw_orch::prelude::*;
use cw_orch_clone_testing::CloneTesting;

// testnet addr of abstract
const SENDER: &str = "kujira14cl2dthqamgucg9sfvv4relp3aa83e40yjx3f5";

// Instantiation of Bow Market Maker results in
// `Unexpected exec msg Empty from Addr("kujira18ef6ylkdcwdgdzeyuwrtpa9vzc06cmd24crcxnfjt24dn8qn9sfqe0vg6k")`
// And Kujira have closed-source contracts, so we have no way of knowing what's happening inside. Because of that testing done against existing kujira contracts

const FIN_ADDR: &str = "kujira1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsqq4jjh";
const BOW_MM_ADDR: &str = "kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq";
const BOW_ADDR: &str = "kujira1e7hxytqdg6v05f8ev3wrfcm5ecu3qyhl7y4ga73z76yuufnlk2rqd4uwf4";

const ASSET_A: &str = "ukuji";
const ASSET_B: &str = "factory/kujira1ltvwg69sw3c5z99c6rr08hal7v0kdzfxz07yj5/demo";
const LP_DENOM: &str =
    "factory/kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq/ulp";

const REWARD_DENOM: &str = "rewardtest";
pub struct KujiraStake {
    chain: CloneTesting,
    lp_asset: (String, AssetInfoUnchecked),
}

impl KujiraStake {
    fn dex_name() -> String {
        "fin".to_string()
    }

    fn new(chain: CloneTesting) -> anyhow::Result<Self> {
        let abstr_deployment = AbstractClient::new(chain.clone())?;
        let ans_asset_a = "tao";
        let ans_asset_b = "tao";
        let asset_a = cw_asset::AssetInfoBase::native(ASSET_A);
        let asset_b = cw_asset::AssetInfoBase::native(ASSET_B);

        let pool = PoolAddressBase::SeparateAddresses {
            swap: FIN_ADDR.to_owned(),
            liquidity: BOW_MM_ADDR.to_owned(),
        };
        let pool_metadata = PoolMetadata {
            dex: Self::dex_name(),
            pool_type: PoolType::ConstantProduct,
            assets: vec![AssetEntry::new(ans_asset_a), AssetEntry::new(ans_asset_b)],
        };
        let lp_asset = AssetInfoUnchecked::native(LP_DENOM);

        abstr_deployment.name_service().update_contract_addresses(
            vec![(
                UncheckedContractEntry {
                    protocol: "bow".to_owned(),
                    contract: format!(
                        "staking/{dex}/{asset_a},{asset_b}",
                        dex = Self::dex_name(),
                        asset_a = &ans_asset_a,
                        asset_b = &ans_asset_b,
                    ),
                },
                BOW_ADDR.to_owned(),
            )],
            vec![],
        )?;
        // Add assets
        abstr_deployment.name_service().update_asset_addresses(
            vec![
                (ans_asset_a.to_string(), asset_a),
                (ans_asset_b.to_string(), asset_b),
            ],
            vec![],
        )?;
        // Add lp asset
        let lp_token = LpToken::new(Self::dex_name(), vec![ans_asset_a, ans_asset_b]);
        // Add dex
        abstr_deployment
            .name_service()
            .update_dexes(vec![Self::dex_name()], vec![])?;
        // Add pool
        abstr_deployment
            .name_service()
            .update_pools(vec![(pool, pool_metadata)], vec![])?;
        abstr_deployment
            .name_service()
            .update_asset_addresses(vec![(lp_token.to_string(), lp_asset.clone())], vec![])?;

        Ok(Self {
            chain,
            lp_asset: (lp_token.to_string(), lp_asset),
            // minter: Addr::unchecked(BOW_MM_ADDR),
        })
    }
}

impl MockStaking for KujiraStake {
    fn name(&self) -> String {
        "bow".to_owned()
    }

    fn stake_token(&self) -> (String, AssetInfoUnchecked) {
        self.lp_asset.clone()
    }

    fn mint_lp(&self, addr: &Addr, amount: u128) -> anyhow::Result<()> {
        self.chain
            .add_balance(addr, coins(amount, LP_DENOM))
            .map_err(Into::into)
    }

    fn generate_rewards(&self, addr: &Addr, amount: u128) -> anyhow::Result<()> {
        let block_info = self.chain.block_info()?;
        let config: kujira::bow::staking::ConfigResponse = self
            .chain
            .wasm_querier()
            .smart_query(BOW_ADDR, &kujira::bow::staking::QueryMsg::Config {})?;
        let pool: kujira::bow::staking::PoolResponse = self.chain.wasm_querier().smart_query(
            BOW_ADDR,
            &kujira::bow::staking::QueryMsg::Pool {
                denom: LP_DENOM.into(),
            },
        )?;
        let stake: kujira::bow::staking::StakeResponse = self.chain.wasm_querier().smart_query(
            BOW_ADDR,
            &kujira::bow::staking::QueryMsg::Stake {
                denom: LP_DENOM.into(),
                addr: addr.to_owned(),
            },
        )?;

        let ratio = Decimal::from_ratio(pool.total, stake.amount).ceil();
        let amount = (Uint128::new(amount) * ratio).u128();

        let send_amount = vec![config.incentive_fee, coin(amount, REWARD_DENOM)];
        self.chain
            .add_balance(&self.chain.sender, send_amount.clone())?;

        self.chain.execute(
            &kujira::bow::staking::ExecuteMsg::AddIncentive {
                denom: LP_DENOM.into(),
                schedule: kujira::Schedule {
                    start: block_info.time.plus_seconds(1),
                    end: block_info.time.plus_seconds(100),
                    amount: amount.into(),
                    release: kujira::Release::Fixed,
                },
            },
            &send_amount,
            &Addr::unchecked(BOW_ADDR),
        )?;
        self.chain.wait_seconds(400)?;
        Ok(())
    }

    fn reward_asset(&self) -> AssetInfoUnchecked {
        cw_asset::AssetInfoBase::Native(REWARD_DENOM.to_owned())
    }

    fn staking_target(&self) -> abstract_cw_staking::msg::StakingTarget {
        abstract_cw_staking::msg::StakingTarget::Contract(Addr::unchecked(BOW_ADDR))
    }
}

fn setup() -> anyhow::Result<StakingTester<CloneTesting, KujiraStake>> {
    let chain_info = HARPOON_4;
    let sender = Addr::unchecked(SENDER);
    // std::env::set_var("RUST_LOG", "debug");
    let abstr_deployment = load_abstr(chain_info, sender)?;
    let chain = abstr_deployment.environment();
    let kujira_stake = KujiraStake::new(chain)?;
    StakingTester::new(abstr_deployment, kujira_stake)
}

#[test]
fn test_stake() -> anyhow::Result<()> {
    let stake_tester = setup()?;
    stake_tester.test_stake()?;
    Ok(())
}

#[test]
fn test_unstake() -> anyhow::Result<()> {
    let stake_tester = setup()?;
    stake_tester.test_unstake()?;
    Ok(())
}

#[test]
fn test_claim() -> anyhow::Result<()> {
    let stake_tester = setup()?;
    stake_tester.test_claim()?;
    Ok(())
}

#[test]
fn test_queries() -> anyhow::Result<()> {
    let stake_tester = setup()?;
    stake_tester.test_staking_info()?;
    stake_tester.test_query_rewards()?;
    Ok(())
}
