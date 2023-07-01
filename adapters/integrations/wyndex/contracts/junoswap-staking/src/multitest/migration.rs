use crate::{multitest::suite::PoolDenom, ContractError};

use super::suite::SuiteBuilder;

use wasmswap_cw20::Denom;

use cosmwasm_std::{assert_approx_eq, coin, Addr, Coin, Decimal, Uint128};

#[test]
fn testsuite_instantiation() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Native(ujuno.to_owned()),
            PoolDenom::Native(uluna.to_owned()),
        )
        .build();

    suite
        .provide_liquidity_to_junoswap_pool(
            user,
            1_000_000u128,
            1_000_000u128,
            None,
            None,
            vec![coin(1_000_000, "ujuno"), coin(1_000_000, "uluna")],
        )
        .unwrap();

    // stake some of these tokens
    let lp = suite.junoswap_lp(user, None).unwrap();
    assert_eq!(lp.u128(), 1_000_000u128);
    let to_stake = lp * Decimal::percent(30);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();
    let new_lp = suite.junoswap_lp(user, None).unwrap();
    assert_eq!(new_lp, lp - to_stake);
}

#[test]
fn partial_migration() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let users = (0..10).map(|i| format!("user{}", i)).collect::<Vec<_>>();

    let suite = SuiteBuilder::new().with_denoms(
        PoolDenom::Native(ujuno.to_owned()),
        PoolDenom::Native(uluna.to_owned()),
    );
    let mut suite = suite.build();

    for (i, user) in users.iter().enumerate() {
        // Provide (roughly) 1m of each asset
        suite
            .provide_liquidity_to_junoswap_pool(
                user,
                1_000_000u128,
                1_000_000 + i as u128,
                None,
                None,
                vec![coin(1_000_000, ujuno), coin(1_000_000 + i as u128, uluna)],
            )
            .unwrap();

        // stake some of these tokens - 80% of liquidity should be moved
        let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
        suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();
    }

    // migrate the tokens
    suite
        .migrate_tokens_with_self_upgrade(None, None, None)
        .unwrap();

    assert!(
        !suite.migration_finished().unwrap(),
        "stakers not migrated yet"
    );

    suite.migrate_stakers(5).unwrap();

    // only 5 stakers should have tokens now
    let lp_tokens: Vec<_> = users
        .iter()
        .map(|u| suite.wyndex_staked(u, suite.migration_unbonding_period()))
        .filter(|lp| *lp > 0)
        .collect();
    assert_eq!(lp_tokens.len(), 5);
    assert!(lp_tokens.iter().all(|lp| *lp == 799900));

    assert!(!suite.migration_finished().unwrap(), "still stakers left");

    suite.migrate_stakers(5).unwrap();

    // next 5 stakers should also have tokens now
    let lp_tokens: Vec<_> = users
        .iter()
        .map(|u| suite.wyndex_staked(u, suite.migration_unbonding_period()))
        .filter(|lp| *lp > 0)
        .collect();
    assert_eq!(lp_tokens.len(), 10);
    assert!(lp_tokens.iter().all(|lp| *lp == 799900));

    assert!(suite.migration_finished().unwrap(), "all stakers migrated");
}

#[test]
fn migration_sanity_check() {
    // This just does a basic migration of one staked user with no claims
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";
    let liquidity = vec![coin(1_000_000, ujuno), coin(1_000_000, uluna)];

    let mut suite = setup_basic_suite(ujuno, uluna, user, liquidity.clone());

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // make sure no lp before the deposit
    let wyndex_total = suite.total_wyndex_lp();
    assert_eq!(wyndex_total, 0u128);

    // cross our fingers this works ;)
    suite.migrate_to_wyndex(None, None, None).unwrap();

    // 80% of liquidity moved to wyndex pool
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

    // 20% of liquidity still in junoswap
    let junoswap = suite
        .app
        .wrap()
        .query_all_balances(&suite.junoswap_pool_contract)
        .unwrap();
    let expected = liquidity
        .iter()
        .map(|Coin { amount, denom }| coin(amount.u128() / 5, denom))
        .collect::<Vec<_>>();
    assert_eq!(junoswap, expected);

    // ensure all lp belong to the staking contract
    // except for the MINIMUM_LIQUIDITY_DEPOSIT - held by pool
    // https://github.com/cosmorama/wyndex-priv/blob/d39f7369d22d458a85c6828d151bc3844a1604bf/contracts/pair/src/contract.rs#L381-L395
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

#[test]
fn non_migrator_cant_migrate() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";
    let liquidity = vec![coin(1_000_000, ujuno), coin(1_000_000, uluna)];

    let mut suite = setup_basic_suite(ujuno, uluna, user, liquidity);

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // this won't work, not the migrator
    let err = suite
        .migrate_to_wyndex(Some(Addr::unchecked("notthemigrator")), None, None)
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap())
}
#[test]
fn migrator_cant_migrate_to_own_addr() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";
    let liquidity = vec![coin(1_000_000, ujuno), coin(1_000_000, uluna)];

    let mut suite = setup_basic_suite(ujuno, uluna, user, liquidity);

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // this won't work, we can only migrate to a deployed pool contract.
    let err = suite
        .migrate_to_wyndex(
            Some(Addr::unchecked("owner")),
            Some(suite.wyndex_pair_contract.clone()),
            Some(Addr::unchecked("owner")),
        )
        .unwrap_err();

    assert_eq!(
        ContractError::InvalidDestination("owner".to_string()),
        err.downcast().unwrap()
    );
}

fn setup_basic_suite(
    ujuno: &str,
    uluna: &str,
    user: &str,
    liquidity: Vec<Coin>,
) -> super::suite::Suite {
    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Native(ujuno.to_owned()),
            PoolDenom::Native(uluna.to_owned()),
        )
        .build();
    suite
        .provide_liquidity_to_junoswap_pool(
            user,
            1_000_000u128,
            1_000_000u128,
            None,
            None,
            liquidity.clone(),
        )
        .unwrap();
    // check balances of pool
    let junoswap = suite
        .app
        .wrap()
        .query_all_balances(&suite.junoswap_pool_contract)
        .unwrap();
    assert_eq!(junoswap, liquidity);
    suite
}

#[test]
fn migration_multiple_users() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let users = ["user", "user2", "user3"];
    // Setup Pools with initial liquidity
    let liquidity = vec![coin(5_000_000, ujuno), coin(5_000_003, uluna)];

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Native(ujuno.to_owned()),
            PoolDenom::Native(uluna.to_owned()),
        )
        .build();
    // First user to provide 1.5m of each asset
    suite
        .provide_liquidity_to_junoswap_pool(
            users[0],
            1_500_000u128,
            1_500_000u128,
            None,
            None,
            vec![coin(1_500_000, ujuno), coin(1_500_000, uluna)],
        )
        .unwrap();

    // Need more coins this time on asset 2, because of this: https://github.com/Wasmswap/wasmswap-contracts/blob/cbca1f2bd8088b2ac6784c47a4509227a227c755/src/contract.rs#L278
    suite
        .provide_liquidity_to_junoswap_pool(
            users[1],
            500_000u128,
            500_001u128,
            None,
            None,
            vec![coin(500_000, ujuno), coin(500_001, uluna)],
        )
        .unwrap();

    suite
        .provide_liquidity_to_junoswap_pool(
            users[2],
            3_000_000u128,
            3_000_002u128,
            None,
            None,
            vec![coin(3_000_000, ujuno), coin(3_000_002, uluna)],
        )
        .unwrap();

    // check balances of pool
    let junoswap = suite
        .app
        .wrap()
        .query_all_balances(&suite.junoswap_pool_contract)
        .unwrap();
    assert_eq!(junoswap, liquidity);

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(users[0], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(users[0], to_stake, None, None)
        .unwrap();
    let to_stake = suite.junoswap_lp(users[1], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(users[1], to_stake, None, None)
        .unwrap();
    let to_stake = suite.junoswap_lp(users[2], None).unwrap() * Decimal::percent(80);
    suite
        .stake_junoswap_lp(users[2], to_stake, None, None)
        .unwrap();

    // perform the migration of liquidity
    suite.migrate_to_wyndex(None, None, None).unwrap();

    // 80% of liquidity moved to wyndex pool
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

    // ensure all lp belong to the staking contract
    let wyndex_total = suite.total_wyndex_lp();
    let wyndex_staked = suite.total_wyndex_staked();
    assert_approx_eq!(wyndex_total, wyndex_staked, "0.01");
    // ensure all staked tokens belong to the migrated user
    let user_staked = suite.wyndex_staked(users[0], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 10 * 3);
    // user 2 staked 500k so they should have about 10% of the staked tokens
    let user_staked = suite.wyndex_staked(users[1], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 10,);
    // user 3 did 3m and should have 60%
    let user_staked = suite.wyndex_staked(users[2], suite.migration_unbonding_period());
    assert_eq!(user_staked, wyndex_staked / 10 * 6);
}

#[test]
fn migrate_with_mixed_pairs() {
    let ujuno = "ujuno";
    let uraw = "rawtoken";
    let users = ["user", "user2", "user3"];
    // Setup Pools with initial liquidity
    let liquidity = vec![coin(3_000_000, ujuno)];
    let liquidity_cw20 = 3_000_002u128;

    let mut suite = SuiteBuilder::new()
        .with_denoms(
            PoolDenom::Native(ujuno.to_owned()),
            PoolDenom::Cw20(uraw.to_owned()),
        )
        .build();

    let raw_denom = suite.pool_denom2.clone();
    let raw_address = match &raw_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };

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
    suite.migrate_to_wyndex(None, None, None).unwrap();

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
    let wyndex = cw20::Cw20Contract(raw_address.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = liquidity_cw20 * 4 / 5;
    assert_eq!(wyndex.u128(), expected);

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
}

#[test]
fn migrate_with_unbonding_claims() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";
    let liquidity = vec![coin(1_000_000, ujuno), coin(1_000_000, uluna)];

    let mut suite = setup_basic_suite(ujuno, uluna, user, liquidity.clone());

    // stake some of these tokens - 80% of liquidity should be moved
    let to_stake = suite.junoswap_lp(user, None).unwrap() * Decimal::percent(80);
    suite.stake_junoswap_lp(user, to_stake, None, None).unwrap();

    // Process an unstake request to generate a claim
    suite
        .unstake_junoswap_lp(user, 800_000u128.into(), None)
        .unwrap();
    assert_eq!(
        suite
            .query_stake_claims_for_pair(user.to_string())
            .claims
            .len(),
        1
    );

    // make sure no lp before the deposit
    let wyndex_total = suite.total_wyndex_lp();
    assert_eq!(wyndex_total, 0u128);

    // Migrate the liquidity to Wyndex, the liquidity includes 1 pair with 1 user who is currently unstaking
    suite.migrate_to_wyndex(None, None, None).unwrap();

    // 80% of liquidity moved to wyndex pool
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

    // 20% of liquidity still in junoswap
    let junoswap = suite
        .app
        .wrap()
        .query_all_balances(&suite.junoswap_pool_contract)
        .unwrap();
    let expected = liquidity
        .iter()
        .map(|Coin { amount, denom }| coin(amount.u128() / 5, denom))
        .collect::<Vec<_>>();
    assert_eq!(junoswap, expected);

    // ensure all lp belong to the staking contract
    // except for the MINIMUM_LIQUIDITY_DEPOSIT - held by pool
    // https://github.com/cosmorama/wyndex-priv/blob/d39f7369d22d458a85c6828d151bc3844a1604bf/contracts/pair/src/contract.rs#L381-L395
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

#[test]
fn migration_two_cw20() {
    let user = "user";
    let liquidity = 2_000_000u128;

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
    let cw20_address = match &cw20_denom {
        Denom::Cw20(address) => address,
        _ => panic!("expected cw20 denom"),
    };

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

    // cross our fingers this works ;)
    suite.migrate_to_wyndex(None, None, None).unwrap();

    // 80% of liquidity moved to wyndex pool
    let wyndex = cw20::Cw20Contract(raw_address.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = liquidity * 4 / 5;
    assert_eq!(wyndex.u128(), expected);
    let wyndex = cw20::Cw20Contract(cw20_address.clone())
        .balance(&suite.app.wrap(), suite.wyndex_pair_contract.clone())
        .unwrap();
    let expected = liquidity * 4 / 5;
    assert_eq!(wyndex.u128(), expected);

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
