use wyndex::{asset::MINIMUM_LIQUIDITY_AMOUNT, stake::ConverterConfig};
use wyndex_stake::msg::MigrateMsg;

use super::suite::{juno, uusd, Pair, SuiteBuilder, DAY};

#[test]
fn migrate_to_existing_pool() {
    let user = "user";

    let ujuno_amount = 1_000_000u128;
    let lsd_amount = 1_000_000u128;
    let uusd_amount = 1_000_000u128;

    let unbonding_period = 14 * DAY;

    let mut suite = SuiteBuilder::new()
        .with_native_balances("ujuno", vec![(user, lsd_amount + ujuno_amount)])
        .with_native_balances("uusd", vec![(user, 2 * uusd_amount)])
        .build();

    // get some wyJUNO
    suite.bond_juno(user, lsd_amount).unwrap();
    let lsd_balance = suite.query_cw20_balance(user, &suite.lsd_token).unwrap();
    assert_eq!(lsd_balance, lsd_amount);

    // provide some base liquidity to both pools
    let native_lp = suite
        .provide_liquidity(user, juno(ujuno_amount), uusd(uusd_amount))
        .unwrap();
    let lsd_lp = suite
        .provide_liquidity(user, suite.lsd_asset(lsd_amount), uusd(uusd_amount))
        .unwrap();

    // stake native LP
    suite
        .stake_lp(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128()
    );

    // migrating from lsd LP to native LP should fail
    let err = suite
        .migrate_stake(Pair::Lsd, user, lsd_lp, unbonding_period)
        .unwrap_err();
    assert_eq!(
        wyndex_stake::ContractError::NoConverter {},
        err.downcast().unwrap()
    );

    // migrate it to lsd LP
    suite
        .migrate_stake(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();

    // check that the stake was migrated
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(stake.stake.u128(), 0);
    let stake = suite
        .query_stake(Pair::Lsd, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128(),
        "all of the stake that was previously in native LP should now be migrated to lsd LP"
    );
}

#[test]
fn migrate_converter_config() {
    let user = "user";

    let ujuno_amount = 1_000_000u128;
    let uusd_amount = 1_000_000u128;

    let unbonding_period = 14 * DAY;

    let mut suite = SuiteBuilder::new()
        .with_native_balances("ujuno", vec![(user, ujuno_amount)])
        .with_native_balances("uusd", vec![(user, uusd_amount)])
        .without_converter()
        .build();

    // provide some liquidity to the native pair
    let native_lp = suite
        .provide_liquidity(user, juno(ujuno_amount), uusd(uusd_amount))
        .unwrap();

    // stake native LP
    suite
        .stake_lp(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();

    // migrating the liquidity before the converter is set should fail
    let err = suite
        .migrate_stake(Pair::Native, user, native_lp, unbonding_period)
        .unwrap_err();
    assert_eq!(
        wyndex_stake::ContractError::NoConverter {},
        err.downcast().unwrap()
    );

    // migrate the staking contract to add the converter
    suite
        .migrate_staking_contract(
            Pair::Native,
            MigrateMsg {
                unbonder: None,
                converter: Some(ConverterConfig {
                    contract: suite.converter.to_string(),
                    pair_to: suite.lsd_pair.to_string(),
                }),
                unbond_all: false,
            },
        )
        .unwrap();

    // migrate liquidity to lsd pair
    suite
        .migrate_stake(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();

    // check that the stake was migrated
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(stake.stake.u128(), 0);
    let stake = suite
        .query_stake(Pair::Lsd, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        ujuno_amount - 2 * MINIMUM_LIQUIDITY_AMOUNT.u128(), // 2x because we lp'd twice on empty pools
        "all of the stake that was previously in native LP should now be migrated to lsd LP"
    );
}

#[test]
fn partial_migration() {
    let user = "user";

    let ujuno_amount = 1_000_000u128;
    let uusd_amount = 1_000_000u128;

    let unbonding_period = 14 * DAY;

    let mut suite = SuiteBuilder::new()
        .with_native_balances("ujuno", vec![(user, ujuno_amount)])
        .with_native_balances("uusd", vec![(user, 2 * uusd_amount)])
        .build();

    // provide some base liquidity to native pool
    let native_lp = suite
        .provide_liquidity(user, juno(ujuno_amount), uusd(uusd_amount))
        .unwrap();

    // stake native LP
    suite
        .stake_lp(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();

    // migrate half of native LP to lsd LP
    suite
        .migrate_stake(Pair::Native, user, native_lp / 2, unbonding_period)
        .unwrap();

    // check that only half of the stake was migrated
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        (ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128()) / 2,
        "half of the stake (minus minimum amount) should remain in native LP"
    );
    let stake = suite
        .query_stake(Pair::Lsd, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        (ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128()) / 2 - MINIMUM_LIQUIDITY_AMOUNT.u128(),
        "half of the stake should be migrated to lsd LP (minus minimum amount)"
    );
}

#[test]
fn empty_stake_fails() {
    let user = "user";

    let ujuno_amount = 1_000_000u128;
    let uusd_amount = 1_000_000u128;

    let unbonding_period = 14 * DAY;

    let mut suite = SuiteBuilder::new()
        .with_native_balances("ujuno", vec![(user, ujuno_amount)])
        .with_native_balances("uusd", vec![(user, uusd_amount)])
        .build();

    // provide some base liquidity to native pool
    let native_lp = suite
        .provide_liquidity(user, juno(ujuno_amount), uusd(uusd_amount))
        .unwrap();

    // stake native LP
    suite
        .stake_lp(Pair::Native, user, native_lp, unbonding_period)
        .unwrap();
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128()
    );

    // migrating zero amount should fail
    let err = suite
        .migrate_stake(Pair::Native, user, 0, unbonding_period)
        .unwrap_err();
    assert!(err.root_cause().to_string().contains("empty coins"));

    // migrating more stake than available should fail
    suite
        .migrate_stake(Pair::Native, user, ujuno_amount, unbonding_period)
        .unwrap_err();

    // check that the stake was not migrated
    let stake = suite
        .query_stake(Pair::Native, user, unbonding_period)
        .unwrap();
    assert_eq!(
        stake.stake.u128(),
        ujuno_amount - MINIMUM_LIQUIDITY_AMOUNT.u128()
    );
}
