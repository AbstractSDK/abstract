use super::suite::SuiteBuilder;
use crate::{multitest::suite::PoolDenom, ContractError};

use wyndex::asset::{AssetInfoValidated, AssetValidated};

use wasmswap_cw20::Denom;

use cosmwasm_std::{assert_approx_eq, coin, Addr, Coin, Decimal, Uint128};

#[test]
fn migrate_raw_with_native() {
    let ujuno = "ujuno";
    let raw = "RAW";
    let users = ["user", "user2", "user3"];
    // Setup Pools with initial liquidity
    let liquidity = vec![coin(3_000_000, ujuno)];
    let liquidity_cw20 = 3_000_002u128;

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Native(ujuno.to_owned()),
            PoolDenom::Cw20(raw.to_owned()),
        )
        .build();

    let raw_denom = suite.pool_denom2.clone();
    let raw_address = match &raw_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };

    let wynd_token = suite.wynd_cw20_token.clone();

    // mint WYND tokens for migration contract
    let junoswap_staking_contract = suite.junoswap_staking_contract.to_string();
    suite
        .mint_cw20(
            "owner",
            &wynd_token,
            5_000_000u128,
            &junoswap_staking_contract,
        )
        .unwrap();

    // Provide Liquidity to the CW20 Native pair
    // Note: Second deposits need 1 extra cw20 similar to the above rounding errors
    suite
        .provide_liquidity_to_junoswap_pool(
            users[0],
            1_000_000u128,
            1_000_000u128,
            Some(Denom::Native(ujuno.to_owned())),
            Some(raw_denom.clone()),
            vec![coin(1_000_000, ujuno)],
        )
        .unwrap();

    suite
        .provide_liquidity_to_junoswap_pool(
            users[1],
            1_000_000u128,
            1_000_001u128,
            Some(Denom::Native(ujuno.to_owned())),
            Some(raw_denom.clone()),
            vec![coin(1_000_000, ujuno)],
        )
        .unwrap();
    suite
        .provide_liquidity_to_junoswap_pool(
            users[2],
            1_000_000u128,
            1_000_001u128,
            Some(Denom::Native(ujuno.to_owned())),
            Some(raw_denom.clone()),
            vec![coin(1_000_000, ujuno)],
        )
        .unwrap();

    // check balances of pool
    let junoswap = suite
        .app
        .wrap()
        .query_all_balances(&suite.junoswap_pool_contract)
        .unwrap();
    assert_eq!(junoswap, liquidity);
    let junoswap = cw20::Cw20Contract(raw_address.clone())
        .balance(&suite.app.wrap(), suite.junoswap_pool_contract.clone())
        .unwrap();
    assert_eq!(junoswap.u128(), liquidity_cw20);

    suite.wyndex_lp_holders();

    // stake some of these tokens - 80% of liquidity should be moved
    // users[0] will keep everything staked
    let to_stake = suite.junoswap_lp(users[0], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(
            users[0],
            to_stake,
            None,
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();
    // users[1] will unstake half
    let to_stake = suite.junoswap_lp(users[1], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(
            users[1],
            to_stake,
            None,
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();
    suite
        .unstake_junoswap_lp(
            users[1],
            to_stake / Uint128::new(2),
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();
    // users[2] will unstake all
    let to_stake = suite.junoswap_lp(users[2], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(
            users[2],
            to_stake,
            None,
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();
    suite
        .unstake_junoswap_lp(
            users[2],
            to_stake,
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();

    // perform the migration of liquidity
    let raw_to_wynd_exchange_rate = Decimal::percent(200);
    suite
        .migrate_to_wyndex(
            None,
            None,
            None,
            raw_to_wynd_exchange_rate,
            raw_address.clone(),
        )
        .unwrap();

    // 80% of native liquidity moved to wyndex pool
    let wyndex = suite
        .app
        .wrap()
        .query_all_balances(&suite.wyndex_pair_contract)
        .unwrap();
    let expected = liquidity
        .iter()
        .map(|Coin { amount, denom }| coin(amount.u128() * 4 / 5, denom))
        .collect::<Vec<_>>();
    assert_eq!(wyndex, expected);

    // 80% of cw20 liquidity moved to wyndex pool
    // RAW token is now GONE, it was swapped in prepare_denom_deposits at a rate of 200% (set above)
    // Check the balance of WYND token instead, it should have been swapped at a 2-1 rate (200%)
    let wyndex = cw20::Cw20Contract(wynd_token.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = liquidity_cw20 * 4 / 5;
    assert_eq!(wyndex.u128(), expected * 2);

    // ensure all lp belong to the staking contract
    let wyndex_total = suite.total_wyndex_lp();
    let wyndex_staked = suite.total_wyndex_staked();
    assert_approx_eq!(wyndex_total, wyndex_staked, "0.001");
    // ensure all staked tokens belong to the migrated user
    let user_staked = suite.wyndex_staked(users[0], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 3);
    // user 2 staked 500k so they should have about 10% of the staked tokens
    let user_staked = suite.wyndex_staked(users[1], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 3);
    // user 3 did 3m and should have 60%
    let user_staked = suite.wyndex_staked(users[2], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 3);

    // now to the meat - check assets of the pool and balance
    // verify amounts here, for cw20 its 3mil * 2 (exchange rate) * 4/5 (80% of liquidity was moved) = 4.8M
    // for native, just the 3 mil * 4/5 (80% of liquidity was moved) = 2.4M
    let pool_info = suite.query_wyndex_pool();
    assert_eq!(
        pool_info.assets,
        vec![
            AssetValidated {
                info: AssetInfoValidated::Native(ujuno.to_string()),
                amount: Uint128::new(2_400_000)
            },
            AssetValidated {
                info: AssetInfoValidated::Token(wynd_token.clone()),
                amount: Uint128::new(4_800_002)
            }
        ]
    );
}

#[test]
fn non_migrator_cant_migrate() {
    let user = "user";

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Cw20("raw".to_owned()),
            PoolDenom::Cw20("cwt".to_owned()),
        )
        .build();

    let raw_denom = suite.pool_denom1.clone();
    let raw_address = match &raw_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };
    let cw20_denom = suite.pool_denom2.clone();
    let wynd_token = suite.wynd_cw20_token.clone();

    // mint WYND tokens for migration contract
    let junoswap_staking_contract = suite.junoswap_staking_contract.to_string();
    suite
        .mint_cw20(
            "owner",
            &wynd_token,
            5_000_000u128,
            &junoswap_staking_contract,
        )
        .unwrap();

    // Provide Liquidity to the CW20 pair
    suite
        .provide_liquidity_to_junoswap_pool(
            user,
            2_000_000u128,
            2_000_000u128,
            Some(cw20_denom),
            Some(raw_denom.clone()),
            vec![],
        )
        .unwrap();

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // this won't work, not the migrator
    let err = suite
        .migrate_to_wyndex(
            Some(Addr::unchecked("notthemigrator")),
            None,
            None,
            Decimal::percent(100),
            raw_address.clone(),
        )
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap())
}
#[test]
fn migrator_cant_migrate_to_own_addr() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Cw20("cwt".to_owned()),
            PoolDenom::Cw20("raw".to_owned()),
        )
        .build();

    let raw_denom = suite.pool_denom1.clone();
    let cw20_denom = suite.pool_denom2.clone();
    let raw_address = match &cw20_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };
    let wynd_token = suite.wynd_cw20_token.clone();

    // mint WYND tokens for migration contract
    let junoswap_staking_contract = suite.junoswap_staking_contract.to_string();
    suite
        .mint_cw20(
            "owner",
            &wynd_token,
            5_000_000u128,
            &junoswap_staking_contract,
        )
        .unwrap();

    // Provide Liquidity to the CW20 pair
    suite
        .provide_liquidity_to_junoswap_pool(
            user,
            2_000_000u128,
            2_000_000u128,
            Some(cw20_denom.clone()),
            Some(raw_denom),
            vec![],
        )
        .unwrap();

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // this won't work, we can only migrate to a deployed pool contract.
    let err = suite
        .migrate_to_wyndex(
            Some(Addr::unchecked("owner")),
            Some(suite.wyndex_pair_contract.clone()),
            Some(Addr::unchecked("owner")),
            Decimal::percent(100),
            raw_address.clone(),
        )
        .unwrap_err();

    assert_eq!(
        ContractError::InvalidDestination("owner".to_string()),
        err.downcast().unwrap()
    );
}

#[test]
fn migration_two_cw20() {
    let user = "user";
    let liquidity = 2_000_000u128;

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Cw20("cwt".to_owned()),
            PoolDenom::Cw20("raw".to_owned()),
        )
        .build();

    let cw20_denom = suite.pool_denom1.clone();
    let cw20_address = match &cw20_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };
    let raw_denom = suite.pool_denom2.clone();
    let raw_address = match &raw_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };
    let wynd_token = suite.wynd_cw20_token.clone();

    // mint WYND tokens for migration contract
    let junoswap_staking_contract = suite.junoswap_staking_contract.to_string();
    suite
        .mint_cw20(
            "owner",
            &wynd_token,
            5_000_000u128,
            &junoswap_staking_contract,
        )
        .unwrap();

    // Provide Liquidity to the CW20 pair
    suite
        .provide_liquidity_to_junoswap_pool(
            user,
            2_000_000u128,
            2_000_000u128,
            Some(cw20_denom.clone()),
            Some(raw_denom.clone()),
            vec![],
        )
        .unwrap();

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(
            user,
            to_stake,
            None,
            Some(&suite.junoswap_staking_contract.clone()),
        )
        .unwrap();

    // make sure no lp before the deposit
    let wyndex_total = suite.total_wyndex_lp();
    assert_eq!(wyndex_total, 0u128);

    // perform the migration of liquidity
    let raw_to_wynd_exchange_rate = Decimal::percent(200);
    suite
        .migrate_to_wyndex(
            None,
            None,
            None,
            raw_to_wynd_exchange_rate,
            raw_address.clone(),
        )
        .unwrap();
    // 80% of liquidity moved to wyndex pool
    let cw20_token_liquidity = cw20::Cw20Contract(cw20_address.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = liquidity * 4 / 5;
    assert_eq!(cw20_token_liquidity.u128(), expected);
    // Wynd given for RAW tokens
    let wyndex = cw20::Cw20Contract(wynd_token.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = Uint128::from(liquidity * 4 / 5) * raw_to_wynd_exchange_rate;
    assert_eq!(wyndex, expected);

    let raw = cw20::Cw20Contract(raw_address.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    assert_eq!(raw.u128(), 0u128);

    // ensure all lp belong to the staking contract
    let wyndex_total = suite.total_wyndex_lp();
    let wyndex_staked = suite.total_wyndex_staked();
    let pool = suite.wyndex_pair_contract.to_string();
    let pool_own_lp = suite.wyndex_lp(&pool);
    assert_eq!(wyndex_total, wyndex_staked + pool_own_lp);
    assert_eq!(pool_own_lp, wyndex::asset::MINIMUM_LIQUIDITY_AMOUNT.u128());

    // ensure all staked tokens belong to the migrated user
    let user_staked = suite.wyndex_staked(user, suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked);
}
