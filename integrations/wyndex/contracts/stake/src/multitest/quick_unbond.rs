use cosmwasm_std::{assert_approx_eq, Decimal};
use wyndex::asset::{AssetInfo, AssetInfoExt, AssetInfoValidated};

use crate::{multitest::suite::SuiteBuilder, ContractError};

use super::suite::Suite;

const DAY: u64 = 24 * 60 * 60;
const UNBONDING_PERIODS: &[u64; 2] = &[DAY, 2 * DAY];
const ADMIN: &str = "owner";
const UNBONDER: &str = "unbonder";
const REWARDS_DISTRIBUTOR: &str = "rewardsdistributor";
const VOTER1: &str = "voter1";
const VOTER2: &str = "voter2";
const VOTER3: &str = "voter3";

fn cash() -> AssetInfoValidated {
    AssetInfoValidated::Native("cash".to_string())
}

fn initial_setup() -> Suite {
    let mut suite = SuiteBuilder::new()
        .with_admin(ADMIN)
        .with_unbonder(UNBONDER)
        .with_min_bond(0)
        .with_tokens_per_power(1)
        .with_unbonding_periods(UNBONDING_PERIODS.to_vec())
        .with_native_balances("cash", vec![(REWARDS_DISTRIBUTOR, 100_000)])
        .with_initial_balances(vec![(VOTER1, 500), (VOTER2, 600), (VOTER3, 450)])
        .build();

    suite
        .create_distribution_flow(
            ADMIN,
            REWARDS_DISTRIBUTOR,
            cash().into(),
            vec![
                (UNBONDING_PERIODS[0], Decimal::percent(100)),
                (UNBONDING_PERIODS[1], Decimal::percent(200)),
            ],
        )
        .unwrap();

    suite.delegate(VOTER1, 500, UNBONDING_PERIODS[0]).unwrap();
    suite.delegate(VOTER2, 600, UNBONDING_PERIODS[1]).unwrap();
    suite.delegate(VOTER3, 450, UNBONDING_PERIODS[0]).unwrap();

    suite
        .rebond(VOTER2, 300, UNBONDING_PERIODS[1], UNBONDING_PERIODS[0])
        .unwrap();
    suite.unbond(VOTER2, 100, UNBONDING_PERIODS[1]).unwrap();
    suite.unbond(VOTER3, 450, UNBONDING_PERIODS[0]).unwrap();

    // at this point, we have:
    assert_eq!(
        suite.query_rewards_power(VOTER1).unwrap()[0].1,
        500,
        "500 in period 1 => power 500"
    );
    assert_eq!(
        suite.query_rewards_power(VOTER2).unwrap()[0].1,
        700,
        "300 in period 1, 200 in period 2 => power 300 + 400 = 700"
    );
    assert!(
        suite.query_rewards_power(VOTER3).unwrap().is_empty(),
        "no stake in any period"
    );
    assert_eq!(
        suite.query_total_rewards_power().unwrap()[0].1,
        1200,
        "500 + 700 = 1200"
    );

    suite
        .distribute_funds(
            REWARDS_DISTRIBUTOR,
            REWARDS_DISTRIBUTOR,
            Some(cash().with_balance(1200u128)),
        )
        .unwrap();

    // validate rewards:
    assert_eq!(
        suite.withdrawable_rewards(VOTER1).unwrap()[0].amount.u128(),
        500,
        "500 / 1200 * 1200 = 500"
    );
    assert_eq!(
        suite.withdrawable_rewards(VOTER2).unwrap()[0].amount.u128(),
        700,
        "700 / 1200 * 1200 = 700"
    );
    assert_eq!(
        suite.withdrawable_rewards(VOTER3).unwrap()[0].amount.u128(),
        0,
        "0 / 1200 * 1200 = 0"
    );

    suite
}

fn run_checks(suite: Suite) {
    // at this point, we have:
    assert_eq!(
        suite.query_rewards_power(VOTER1).unwrap()[0].1,
        500,
        "500 in period 1 => power 500"
    );
    assert!(
        suite.query_rewards_power(VOTER2).unwrap().is_empty(),
        "no stake in any period"
    );
    assert!(
        suite.query_rewards_power(VOTER3).unwrap().is_empty(),
        "no stake in any period"
    );
    assert_eq!(suite.query_total_rewards_power().unwrap()[0].1, 500);

    // check unstaked LP balance
    assert_eq!(
        suite
            .query_cw20_balance(VOTER1, suite.token_contract())
            .unwrap(),
        0
    );
    assert_eq!(
        suite
            .query_cw20_balance(VOTER2, suite.token_contract())
            .unwrap(),
        600
    );
    assert_eq!(
        suite
            .query_cw20_balance(VOTER3, suite.token_contract())
            .unwrap(),
        450
    );

    // check withdrawable rewards
    assert_approx_eq!(
        suite.withdrawable_rewards(VOTER1).unwrap()[0].amount.u128(),
        1300,
        "0.001",
        "500 + 800 = 1300",
    );
    assert_eq!(
        suite.withdrawable_rewards(VOTER2).unwrap()[0].amount.u128(),
        700,
        "same as before"
    );
    assert_eq!(
        suite.withdrawable_rewards(VOTER3).unwrap()[0].amount.u128(),
        0,
        "same as before"
    );

    assert_eq!(
        suite.query_staked(VOTER1, UNBONDING_PERIODS[0]).unwrap(),
        500
    );
    assert_eq!(suite.query_staked(VOTER1, UNBONDING_PERIODS[1]).unwrap(), 0);
    assert_eq!(suite.query_staked(VOTER2, UNBONDING_PERIODS[0]).unwrap(), 0);
    assert_eq!(suite.query_staked(VOTER2, UNBONDING_PERIODS[1]).unwrap(), 0);
    assert_eq!(suite.query_staked(VOTER3, UNBONDING_PERIODS[0]).unwrap(), 0);
    assert_eq!(suite.query_staked(VOTER3, UNBONDING_PERIODS[1]).unwrap(), 0);
    assert_eq!(suite.query_total_staked().unwrap(), 500);

    let bonding_infos = suite.query_staked_periods().unwrap();
    assert_eq!(bonding_infos[0].total_staked.u128(), 500);
    assert_eq!(bonding_infos[1].total_staked.u128(), 0);

    // also make sure nobody has claims left
    assert_eq!(suite.query_claims(VOTER1).unwrap().len(), 0);
    assert_eq!(suite.query_claims(VOTER2).unwrap().len(), 0);
    assert_eq!(suite.query_claims(VOTER3).unwrap().len(), 0);
}

#[test]
fn control_case() {
    let mut suite = initial_setup();

    suite.unbond(VOTER2, 200, UNBONDING_PERIODS[1]).unwrap();

    suite.update_time(DAY);

    suite.unbond(VOTER2, 300, UNBONDING_PERIODS[0]).unwrap();

    suite.update_time(DAY);

    suite.claim(VOTER2).unwrap();
    suite.claim(VOTER3).unwrap();

    suite
        .distribute_funds(
            REWARDS_DISTRIBUTOR,
            REWARDS_DISTRIBUTOR,
            Some(cash().with_balance(800u128)),
        )
        .unwrap();

    run_checks(suite);
}

#[test]
fn quick_unbond_case() {
    // same as control case, but quick unbond with no waiting
    let mut suite = initial_setup();

    suite.quick_unbond(UNBONDER, &[VOTER2, VOTER3]).unwrap();

    suite
        .distribute_funds(
            REWARDS_DISTRIBUTOR,
            REWARDS_DISTRIBUTOR,
            Some(cash().with_balance(800u128)),
        )
        .unwrap();

    run_checks(suite);
}

#[test]
fn unbonder_permission_check() {
    let mut suite = initial_setup();

    assert_eq!(
        ContractError::Unauthorized {},
        suite
            .quick_unbond(VOTER2, &[VOTER2])
            .unwrap_err()
            .downcast()
            .unwrap(),
        "only unbonder should be able to quick unbond"
    );

    // now without unbonder
    let mut suite = SuiteBuilder::new().with_admin(ADMIN).build();

    assert_eq!(
        ContractError::Unauthorized {},
        suite
            .quick_unbond(ADMIN, &[VOTER1])
            .unwrap_err()
            .downcast()
            .unwrap(),
        "no one should be able to quick unbond"
    );
}

#[test]
fn non_staker_works() {
    let mut suite = initial_setup();

    suite.quick_unbond(UNBONDER, &[VOTER2, "ignoreme"]).unwrap();

    assert_eq!(
        suite
            .query_cw20_balance(VOTER2, suite.token_contract())
            .unwrap(),
        600
    );
    assert_eq!(
        suite.withdrawable_rewards(VOTER2).unwrap()[0].amount.u128(),
        700,
        "same as before"
    );
}

#[test]
fn multiple_distributions() {
    let mut suite = SuiteBuilder::new()
        .with_admin(ADMIN)
        .with_unbonder(UNBONDER)
        .with_min_bond(100) // also make power calculation a bit more interesting
        .with_tokens_per_power(10)
        .with_unbonding_periods(UNBONDING_PERIODS.to_vec())
        .with_native_balances("cash", vec![(REWARDS_DISTRIBUTOR, 100_000)])
        .with_native_balances("juno", vec![(REWARDS_DISTRIBUTOR, 100_000)])
        .with_initial_balances(vec![(VOTER1, 10), (VOTER2, 100), (VOTER3, 200)])
        .build();

    suite
        .create_distribution_flow(
            ADMIN,
            REWARDS_DISTRIBUTOR,
            cash().into(),
            vec![
                (UNBONDING_PERIODS[0], Decimal::percent(100)),
                (UNBONDING_PERIODS[1], Decimal::percent(200)),
            ],
        )
        .unwrap();

    suite
        .create_distribution_flow(
            ADMIN,
            REWARDS_DISTRIBUTOR,
            AssetInfo::Native("juno".to_string()),
            vec![
                (UNBONDING_PERIODS[0], Decimal::percent(100)),
                (UNBONDING_PERIODS[1], Decimal::percent(200)),
            ],
        )
        .unwrap();

    suite.delegate(VOTER1, 10, UNBONDING_PERIODS[1]).unwrap();
    suite.delegate(VOTER2, 100, UNBONDING_PERIODS[1]).unwrap();
    suite.delegate(VOTER3, 200, UNBONDING_PERIODS[1]).unwrap();

    suite
        .rebond(VOTER2, 100, UNBONDING_PERIODS[1], UNBONDING_PERIODS[0])
        .unwrap();
    suite
        .rebond(VOTER3, 200, UNBONDING_PERIODS[1], UNBONDING_PERIODS[0])
        .unwrap();

    // at this point, we have:
    assert!(
        suite.query_rewards_power(VOTER1).unwrap().is_empty(),
        "10 in period 2 < MIN_BOND"
    );
    assert_eq!(
        suite.query_rewards_power(VOTER2).unwrap()[0].1,
        10,
        "100 in period 1 => power 100 / 10 = 10"
    );
    assert_eq!(
        suite.query_rewards_power(VOTER3).unwrap()[0].1,
        20,
        "200 in period 1 => power 200 / 10 = 20"
    );
    // => total power is 30

    // distribute 3000 cash and 1500 juno
    suite
        .distribute_funds(
            REWARDS_DISTRIBUTOR,
            REWARDS_DISTRIBUTOR,
            Some(cash().with_balance(3000u128)),
        )
        .unwrap();
    suite
        .distribute_funds(
            REWARDS_DISTRIBUTOR,
            REWARDS_DISTRIBUTOR,
            Some(AssetInfoValidated::Native("juno".to_string()).with_balance(1500u128)),
        )
        .unwrap();

    fn assert_rewards(suite: &mut Suite) {
        // summing balance and withdrawable rewards, because some have withdrawn
        let voter1_cash = suite.query_balance(VOTER1, "cash").unwrap();
        let voter2_cash = suite.query_balance(VOTER2, "cash").unwrap();
        let voter3_cash = suite.query_balance(VOTER3, "cash").unwrap();
        let voter1_juno = suite.query_balance(VOTER1, "juno").unwrap();
        let voter2_juno = suite.query_balance(VOTER2, "juno").unwrap();
        let voter3_juno = suite.query_balance(VOTER3, "juno").unwrap();

        let voter1_rewards = suite.withdrawable_rewards(VOTER1).unwrap();
        let voter2_rewards = suite.withdrawable_rewards(VOTER2).unwrap();
        let voter3_rewards = suite.withdrawable_rewards(VOTER3).unwrap();

        // assert cash rewards
        assert_eq!(
            voter1_rewards[0].amount.u128() + voter1_cash,
            0,
            "no power => no rewards"
        );
        assert_eq!(
            voter2_rewards[0].amount.u128() + voter2_cash,
            1000,
            "10 / 30 * 3000 = 1000"
        );
        assert_eq!(
            voter3_rewards[0].amount.u128() + voter3_cash,
            2000,
            "20 / 30 * 3000 = 2000"
        );
        // assert juno rewards
        assert_eq!(
            voter1_rewards[1].amount.u128() + voter1_juno,
            0,
            "no power => no rewards"
        );
        assert_eq!(
            voter2_rewards[1].amount.u128() + voter2_juno,
            500,
            "10 / 30 * 1500 = 500"
        );
        assert_eq!(
            voter3_rewards[1].amount.u128() + voter3_juno,
            1000,
            "20 / 30 * 1500 = 1000"
        );
    }

    assert_rewards(&mut suite);

    // withdraw some rewards before unbonding
    suite.withdraw_funds(VOTER2, None, None).unwrap();

    // create claim
    suite.unbond(VOTER1, 10, UNBONDING_PERIODS[1]).unwrap();

    // now we unbond all of them
    suite
        .quick_unbond(UNBONDER, &[VOTER1, VOTER2, VOTER3])
        .unwrap();

    // rewards should stay the same
    assert_rewards(&mut suite);

    // assert token balances
    assert_eq!(
        suite
            .query_cw20_balance(VOTER1, suite.token_contract())
            .unwrap(),
        10
    );
    assert_eq!(
        suite
            .query_cw20_balance(VOTER2, suite.token_contract())
            .unwrap(),
        100
    );
    assert_eq!(
        suite
            .query_cw20_balance(VOTER3, suite.token_contract())
            .unwrap(),
        200
    );

    // no claims created and none left
    assert!(suite.query_claims(VOTER1).unwrap().is_empty());
    assert!(suite.query_claims(VOTER2).unwrap().is_empty());
    assert!(suite.query_claims(VOTER3).unwrap().is_empty());
}
