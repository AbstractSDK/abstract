use crate::multitest::suite::Suite;
use cosmwasm_std::{assert_approx_eq, coin, Addr, Decimal};
use wyndex::{asset::AssetInfo, factory::DefaultStakeConfig};

use super::suite::SuiteBuilder;

const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

#[test]
fn native_rewards_work() {
    let mut suite = SuiteBuilder::new()
        .with_funds("owner", &[coin(100_000, "juno")])
        .with_stake_config(DefaultStakeConfig {
            staking_code_id: 0,
            tokens_per_power: 1000u128.into(),
            min_bond: 1000u128.into(),
            unbonding_periods: vec![SECONDS_PER_DAY * 7],
            max_distributions: 5,
            converter: None,
        })
        .with_native_reward(100_000, "juno")
        .build();

    // create cw20 token
    let wynd = suite.instantiate_token("owner", "WYND");
    let wynd_info = AssetInfo::Token(wynd.to_string());

    let juno = AssetInfo::Native("juno".to_string());
    let asdf = AssetInfo::Native("asdf".to_string());
    // create pairs to reward
    let (pair1_staking, pair1_lpt) = suite
        .create_pair_staking(juno.clone(), wynd_info.clone())
        .unwrap();
    let (pair2_staking, pair2_lpt) = suite
        .create_pair_staking(juno.clone(), asdf.clone())
        .unwrap();

    // stake all lp tokens
    pair1_staking
        .stake(
            &mut suite.app,
            "whale",
            999_000,
            SECONDS_PER_DAY * 7,
            pair1_lpt,
        )
        .unwrap();
    pair2_staking
        .stake(
            &mut suite.app,
            "whale",
            999_000,
            SECONDS_PER_DAY * 7,
            pair2_lpt,
        )
        .unwrap();

    // create distribution flows, so we can distribute juno
    suite
        .create_distribution_flow(
            "owner",
            vec![juno.clone(), wynd_info],
            juno.clone(),
            vec![(SECONDS_PER_DAY * 7, Decimal::one())],
        )
        .unwrap();
    suite
        .create_distribution_flow(
            "owner",
            vec![juno.clone(), asdf],
            juno,
            vec![(SECONDS_PER_DAY * 7, Decimal::one())],
        )
        .unwrap();

    // sample messages
    let messages = suite.sample_gauge_msgs(vec![
        (pair1_staking.0.to_string(), Decimal::percent(50)),
        (pair2_staking.0.to_string(), Decimal::percent(20)),
    ]);

    // execute messages as owner
    suite
        .app
        .execute_multi(Addr::unchecked("owner"), messages)
        .unwrap();

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();
    pair2_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    // no rewards yet
    assert_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount
            .u128(),
        0u128,
    );

    // let's move forward 20% of the epoch time (100_000 from the reward size - to change)
    suite.next_block(20_000);

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    // 20% of 50_000 should be withdrawable
    assert_approx_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount,
        11_500u128.into(),
        "0.01"
    );

    // let's move forward remaining 80% of the epoch time (100_000 from the reward size - to change)
    suite.next_block(80_000);

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();
    pair2_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    // check final rewards
    assert_approx_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount,
        50_000u128.into(),
        "0.0001"
    );
    assert_approx_eq!(
        pair2_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount,
        20_000u128.into(),
        "0.0001"
    );
}

#[test]
fn cw20_rewards_work_direct() {
    let suite = SuiteBuilder::new()
        .with_funds("owner", &[])
        .with_stake_config(DefaultStakeConfig {
            staking_code_id: 0,
            tokens_per_power: 1000u128.into(),
            min_bond: 1000u128.into(),
            unbonding_periods: vec![SECONDS_PER_DAY * 7],
            max_distributions: 5,
            converter: None,
        })
        .with_cw20_reward(100)
        .build();

    cw20_rewards_work(suite);
}

#[test]
// Like the above test, but here we create the adapter via migration
fn cw20_rewards_work_via_migration() {
    let suite = SuiteBuilder::new()
        .with_funds("owner", &[])
        .with_stake_config(DefaultStakeConfig {
            staking_code_id: 0,
            tokens_per_power: 1000u128.into(),
            min_bond: 1000u128.into(),
            unbonding_periods: vec![SECONDS_PER_DAY * 7],
            max_distributions: 5,
            converter: None,
        })
        .with_cw20_reward(100)
        .via_placeholder()
        .build();

    cw20_rewards_work(suite);
}

fn cw20_rewards_work(mut suite: Suite) {
    // FIXME: how does this work? AssetInfo::to_string() ?? not a cleaner way to unwrap the enum?
    let reward_contract = Addr::unchecked(suite.reward.info.to_string());

    // mint reward token to distribute later for owner
    suite
        .mint_cw20("owner", &reward_contract, 200, "owner")
        .unwrap();

    // create cw20 token
    let wynd = suite.instantiate_token("owner", "WYND");
    let wynd_info = AssetInfo::Token(wynd.to_string());

    let juno = AssetInfo::Native("juno".to_string());
    let asdf = AssetInfo::Native("asdf".to_string());
    // create pairs to reward
    let (pair1_staking, pair1_lpt) = suite
        .create_pair_staking(juno.clone(), wynd_info.clone())
        .unwrap();
    let (pair2_staking, pair2_lpt) = suite
        .create_pair_staking(juno.clone(), asdf.clone())
        .unwrap();

    // stake all lp tokens
    pair1_staking
        .stake(
            &mut suite.app,
            "whale",
            999_000,
            SECONDS_PER_DAY * 7,
            pair1_lpt,
        )
        .unwrap();
    pair2_staking
        .stake(
            &mut suite.app,
            "whale",
            999_000,
            SECONDS_PER_DAY * 7,
            pair2_lpt,
        )
        .unwrap();

    // create distribution flows, so we can distribute juno
    suite
        .create_distribution_flow(
            "owner",
            vec![juno.clone(), wynd_info],
            AssetInfo::Token(reward_contract.to_string()),
            vec![(SECONDS_PER_DAY * 7, Decimal::one())],
        )
        .unwrap();
    suite
        .create_distribution_flow(
            "owner",
            vec![juno, asdf],
            AssetInfo::Token(reward_contract.to_string()),
            vec![(SECONDS_PER_DAY * 7, Decimal::one())],
        )
        .unwrap();

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    assert_eq!(
        suite
            .query_cw20_balance(pair1_staking.0.as_str(), &reward_contract)
            .unwrap(),
        0
    );
    assert_eq!(
        suite
            .query_cw20_balance(pair2_staking.0.as_str(), &reward_contract)
            .unwrap(),
        0
    );

    // sample messages
    let messages = suite.sample_gauge_msgs(vec![
        (pair1_staking.0.to_string(), Decimal::percent(90)),
        (pair2_staking.0.to_string(), Decimal::percent(10)),
    ]);

    // execute messages as owner
    suite
        .app
        .execute_multi(Addr::unchecked("owner"), messages)
        .unwrap();

    // tokens transfered but not distributed to users
    assert_eq!(
        suite
            .query_cw20_balance(pair1_staking.0.as_str(), &reward_contract)
            .unwrap(),
        90
    );
    assert_eq!(
        suite
            .query_cw20_balance(pair2_staking.0.as_str(), &reward_contract)
            .unwrap(),
        10
    );

    // no rewards yet
    assert_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount
            .u128(),
        0u128,
    );

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    // no rewards yet
    assert_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount
            .u128(),
        0u128,
    );

    // let's move forward 20% of the epoch time (86_400 hardcoded)
    suite.next_block(17_280);

    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();
    pair2_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();

    // 20% of 90 should be withdrawable (ERROR: get 109 not 18)
    assert_approx_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount,
        17u128.into(), // 18-1 rounding error - why???
        "0.01"
    );

    // let's move forward remaining epoch time
    suite.next_block(80_000);
    pair1_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();
    // 100% of 90 should be withdrawable (ERROR: get 109 not 90)
    assert_approx_eq!(
        pair1_staking
            .query_withdrawable_rewards(&suite.app, "whale")
            .unwrap()[0]
            .amount,
        89u128.into(), // 90-1 rounding error - why???
        "0.01"
    );

    // withdraw rewards
    pair1_staking
        .withdraw_rewards(&mut suite.app, "whale")
        .unwrap();
    assert_eq!(
        suite.query_cw20_balance("whale", &reward_contract).unwrap(),
        89 // rounding error from 0.9*100
    );

    // withdraw other rewards
    pair2_staking
        .distribute_rewards(&mut suite.app, "owner")
        .unwrap();
    pair2_staking
        .withdraw_rewards(&mut suite.app, "whale")
        .unwrap();
    assert_eq!(
        suite.query_cw20_balance("whale", &reward_contract).unwrap(),
        98 // 2 rounding error from 0.9*100 + 0.1*100
    );
}
