use std::str::FromStr;

use cosmwasm_std::{assert_approx_eq, coin, Addr, Decimal, Fraction, Uint128};
use cw_multi_test::{BankSudo, SudoMsg};
use wyndex::pair::LsdInfo;
use wyndex::{
    asset::{AssetInfo, AssetInfoExt},
    factory::PairType,
    pair::StablePoolParams,
};

use super::suite::{Suite, SuiteBuilder};

const DAY: u64 = 24 * 60 * 60;

#[test]
fn basic_provide_and_swap() {
    let target_rate = Decimal::from_str("1.5").unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(150_0000000000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 150_000_000_000_000_000),
            (wy_juno_info.clone(), 100_000_000_000_000_000),
            vec![coin(150_000_000_000_000_000, "juno")],
        )
        .unwrap();

    // check spot price is 1 wyJUNO -> 1.5 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::percent(150));

    // check spot price is 1 JUNO -> 0.666666 wyJUNO
    let spot = suite
        .query_spot_price(&pair, &juno_info, &wy_juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_ratio(666_667u128, 1_000_000u128));

    let sim = suite
        .query_simulation(&pair, wy_juno_info.with_balance(10u128), None)
        .unwrap();
    assert_eq!(sim.return_amount.u128(), 15);
    assert_eq!(sim.spread_amount.u128(), 0);

    let sim = suite
        .query_simulation(&pair, juno_info.with_balance(150u128), None)
        .unwrap();
    assert_eq!(sim.return_amount.u128(), 100);
    assert_eq!(sim.spread_amount.u128(), 0);

    let sim = suite
        .query_simulation(&pair, juno_info.with_balance(150_000u128), None)
        .unwrap();
    assert_eq!(sim.spread_amount.u128(), 0);

    // do an actual swap
    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(150_000u128),
            wy_juno_info,
            None,
            None,
            None,
        )
        .unwrap();

    assert_eq!(
        suite.query_cw20_balance("sender", &wy_juno).unwrap(),
        100_000u128
    );
}

#[test]
fn simple_provide_liquidity() {
    let target_rate = Decimal::from_str("1.5").unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(150_0000000000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info, 150_000_000_000_000_000),
            (wy_juno_info, 100_000_000_000_000_000),
            vec![coin(150_000_000_000_000_000, "juno")],
        )
        .unwrap();

    let pair_info = suite.query_pair(&pair).unwrap();
    let lp_amount = suite
        .query_cw20_balance("whale", &pair_info.liquidity_token)
        .unwrap();

    // withdraw all liquidity
    suite
        .withdraw_liquidity(
            "whale",
            &pair,
            &pair_info.liquidity_token,
            lp_amount,
            vec![],
        )
        .unwrap();

    assert_approx_eq!(
        suite.query_balance("whale", "juno").unwrap(),
        150_000_000_000_000_000,
        "0.00000000000001"
    );
    assert_approx_eq!(
        suite.query_cw20_balance("whale", &wy_juno).unwrap(),
        100_000_000_000_000_000,
        "0.00000000000001"
    );
}

#[test]
fn provide_liquidity_multiple() {
    let providers = [
        ("provider1", Decimal::percent(50)),
        ("provider2", Decimal::percent(25)),
        ("provider3", Decimal::percent(15)),
        ("provider4", Decimal::percent(10)),
    ];
    let total_lsd = Uint128::from(1_000_000_000_000_000u128);

    let target_rate = Decimal::from_str("1.5").unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(150_0000000000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair(
            "owner",
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            &[juno_info.clone(), wy_juno_info.clone()],
        )
        .unwrap();
    let pair_info = suite.query_pair(&pair).unwrap();

    // each provides liquidity according to their share
    for (provider, share) in &providers {
        let juno_amt = total_lsd * target_rate * *share;
        let wy_juno_amt = total_lsd * *share;

        // mint wyJUNO tokens and increase allowance for LP contract
        suite
            .mint_cw20("owner", &wy_juno, wy_juno_amt.u128(), provider)
            .unwrap();
        suite
            .increase_allowance(provider, &wy_juno, pair.as_str(), wy_juno_amt.u128())
            .unwrap();
        suite
            .app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: provider.to_string(),
                amount: vec![coin(juno_amt.u128(), "juno")],
            }))
            .unwrap();

        suite
            .provide_liquidity(
                provider,
                &pair,
                &[
                    wy_juno_info.with_balance(wy_juno_amt),
                    juno_info.with_balance(juno_amt),
                ],
                &[coin(juno_amt.u128(), "juno")],
            )
            .unwrap();
    }

    // check LP token balances
    let lp_balances: Vec<_> = providers
        .iter()
        .map(|(provider, _)| {
            suite
                .query_cw20_balance(provider, &pair_info.liquidity_token)
                .unwrap()
        })
        .collect();
    let total_lp: u128 = lp_balances.iter().sum();
    for (i, balance) in lp_balances.into_iter().enumerate() {
        // check that each LP token balance is proportional to their share
        assert_approx_eq!(
            Decimal::from_ratio(balance, total_lp).numerator(),
            providers[i].1.numerator(),
            "0.000000000001"
        );
    }

    // withdraw liquidity one by one
    for (provider, share) in providers {
        let lp_amount = suite
            .query_cw20_balance(provider, &pair_info.liquidity_token)
            .unwrap();
        suite
            .withdraw_liquidity(
                provider,
                &pair,
                &pair_info.liquidity_token,
                lp_amount,
                vec![],
            )
            .unwrap();

        // should have received back their share of the pool
        assert_approx_eq!(
            suite.query_balance(provider, "juno").unwrap().into(),
            total_lsd * target_rate * share,
            "0.000000000001"
        );
        assert_approx_eq!(
            suite.query_cw20_balance(provider, &wy_juno).unwrap().into(),
            total_lsd * share,
            "0.000000000001"
        );
    }
}

#[test]
fn provide_liquidity_changing_rate() {
    let target_rate = Decimal::from_str("1.5").unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(150_0000000000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 150_000_000_000_000_000),
            (wy_juno_info, 100_000_000_000_000_000),
            vec![coin(150_000_000_000_000_000, "juno")],
        )
        .unwrap();
    let pair_info = suite.query_pair(&pair).unwrap();

    // change rate
    let target_rate = Decimal::from_str("1.8").unwrap();
    suite.change_target_value(target_rate).unwrap();
    suite.wait(DAY);
    arbitrage_to(&mut suite, &pair, &juno_info, target_rate);

    // withdraw all liquidity
    let lp_amount = suite
        .query_cw20_balance("whale", &pair_info.liquidity_token)
        .unwrap();
    suite
        .withdraw_liquidity(
            "whale",
            &pair,
            &pair_info.liquidity_token,
            lp_amount,
            vec![],
        )
        .unwrap();

    let juno_balance = suite.query_balance("whale", "juno").unwrap();
    let wy_juno_balance = suite.query_cw20_balance("whale", &wy_juno).unwrap();
    assert_approx_eq!(
        Decimal::from_ratio(juno_balance, wy_juno_balance).atomics(),
        target_rate.atomics(),
        "0.0002"
    );
}

#[test]
fn changing_target_rate() {
    let target_rate = Decimal::from_str("1.5").unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(1_000_000_000_000_000_000_000, "juno")])
        .with_funds(
            "arbitrageur",
            &[coin(1_000_000_000_000_000_000_000, "juno")],
        )
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 150_000_000_000_000_000),
            (wy_juno_info.clone(), 100_000_000_000_000_000),
            vec![coin(150_000_000_000_000_000, "juno")],
        )
        .unwrap();

    let sim = suite
        .query_simulation(&pair, wy_juno_info.with_balance(10u128), None)
        .unwrap();
    assert_eq!(sim.return_amount.u128(), 15);
    assert_eq!(sim.spread_amount.u128(), 0);

    let max_target_rate = Decimal::from_str("1.6").unwrap();
    let target_rate_step = Decimal::from_str("0.01").unwrap();
    let mut target_rate = target_rate;
    while target_rate < max_target_rate {
        // change target rate and wait for cache to expire
        target_rate += target_rate_step;
        suite.change_target_value(target_rate).unwrap();
        suite.wait(DAY);

        // now the pool is out of balance, so we arbitrage it to target rate
        arbitrage_to(&mut suite, &pair, &juno_info, target_rate);

        let sim = suite
            .query_simulation(&pair, wy_juno_info.with_balance(100_000u128), None)
            .unwrap();
        assert_approx_eq!(
            sim.return_amount,
            Uint128::from(100_000u128) * target_rate,
            "0.0001"
        );
    }

    // do an actual swap
    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(160_000u128),
            wy_juno_info.clone(),
            None,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        suite.query_cw20_balance("sender", &wy_juno).unwrap(),
        100_000u128
    );

    let min_target_rate = Decimal::from_str("1.4").unwrap();
    let target_rate_step = Decimal::from_str("0.1").unwrap();
    let mut target_rate = target_rate;
    while target_rate > min_target_rate {
        // change target rate and wait for cache to expire
        target_rate -= target_rate_step;
        suite.change_target_value(target_rate).unwrap();
        suite.wait(DAY);

        // now the pool is out of balance, so we arbitrage it to target rate
        arbitrage_to(&mut suite, &pair, &wy_juno_info, target_rate);

        let sim = suite
            .query_simulation(&pair, juno_info.with_balance(100_000u128), None)
            .unwrap();
        assert_approx_eq!(
            sim.return_amount,
            Uint128::from(100_000u128)
                .multiply_ratio(target_rate.denominator(), target_rate.numerator()),
            "0.0001"
        );
    }
}

#[test]
fn drastic_rate_change() {
    let target_rate = Decimal::from_atomics(2u128, 0).unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(1_000_000_000_000_000_000_000, "juno")])
        .with_funds(
            "arbitrageur",
            &[coin(1_000_000_000_000_000_000_000, "juno")],
        )
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 200_000_000_000_000_000),
            (wy_juno_info.clone(), 100_000_000_000_000_000),
            vec![coin(200_000_000_000_000_000, "juno")],
        )
        .unwrap();

    let sim = suite
        .query_simulation(&pair, wy_juno_info.with_balance(100_000u128), None)
        .unwrap();
    assert_eq!(sim.return_amount.u128(), 200_000);
    assert_eq!(sim.spread_amount.u128(), 0);

    // check spot price is 1 wyJUNO -> 2 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::percent(200));

    // change target rate to 1.2 and wait for cache to expire
    let target_rate = Decimal::from_atomics(12u128, 1).unwrap();
    suite.change_target_value(target_rate).unwrap();
    suite.wait(DAY);

    // check spot price is still 1 wyJUNO -> 2 JUNO (shows it is not always target rate)
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::percent(200));

    // we have too much JUNO in the pool, so we arbitrage it away
    arbitrage_to(&mut suite, &pair, &wy_juno_info, target_rate);

    // check spot price is now 1 wyJUNO -> 1.2 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::percent(120));

    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(1_200_000u128),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    assert_approx_eq!(
        suite.query_cw20_balance("sender", &wy_juno).unwrap(),
        1_000_000u128,
        "0.000002"
    );

    // check spot price is slightly less 1 wyJUNO -> 1.2 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(1_199_999u128, 6).unwrap());

    // change target rate to 2.5 and wait for cache to expire
    let target_rate = Decimal::from_atomics(25u128, 1).unwrap();
    suite.change_target_value(target_rate).unwrap();
    suite.wait(DAY);

    // check spot price is unchanged
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(1_199_999u128, 6).unwrap());

    // we have too much wyJUNO in the pool, so we arbitrage it away
    // the next swap will fail with spread assertion if we don't
    arbitrage_to(&mut suite, &pair, &juno_info, target_rate);

    // check spot price is now 1 wyJUNO -> 2.5 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(2_500_001u128, 6).unwrap());

    let prev_balance = suite.query_balance("sender", "juno").unwrap();
    suite
        .swap(
            &pair,
            "sender",
            wy_juno_info.with_balance(1_000_000u128),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    assert_approx_eq!(
        suite.query_balance("sender", "juno").unwrap() - prev_balance,
        2_500_000u128,
        "0.0000004"
    );
}

#[test]
fn changing_spot_price() {
    let target_rate = Decimal::from_atomics(15u128, 1).unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(1_000_000_000, "juno")])
        .with_funds("arbitrageur", &[coin(1_000_000_000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 45,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 150_000_000),
            (wy_juno_info.clone(), 100_000_000),
            vec![coin(150_000_000, "juno")],
        )
        .unwrap();

    // check spot price is about 1 wyJUNO -> 1.5 JUNO
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(1_499_674u128, 6).unwrap());

    // small swap (3% of juno) stays close to target
    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(4_500_000u128),
            None,
            None,
            None,
            None,
        )
        .unwrap();
    assert_approx_eq!(
        suite.query_cw20_balance("sender", &wy_juno).unwrap(),
        3_000_000u128,
        "0.001"
    );

    // check spot price is slightly higher than before
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(1_501_633u128, 6).unwrap());

    // big swap (double juno) changes price a lot target
    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(150_000_000u128),
            None,
            None,
            // allow a huge slippage...
            Decimal::percent(50),
            None,
        )
        .unwrap();

    // check spot price is much larger (TODO: verify exact value with math equations)
    let spot = suite
        .query_spot_price(&pair, &wy_juno_info, &juno_info)
        .unwrap();
    assert_eq!(spot, Decimal::from_atomics(3_483_313u128, 6).unwrap());
}

#[test]
fn predict_swap_spot_price() {
    let iterations = 10u8;
    let target_rate = Decimal::from_atomics(15u128, 1).unwrap();
    let mut suite = SuiteBuilder::new()
        .with_funds("sender", &[coin(1_000_000_000, "juno")])
        .with_funds("arbitrageur", &[coin(1_000_000_000, "juno")])
        .with_initial_target_rate(target_rate)
        .build();

    let juno_info = AssetInfo::Native("juno".to_string());
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

    let pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp: 6,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), 1_500_000_000),
            (wy_juno_info.clone(), 1_000_000_000),
            vec![coin(1_500_000_000, "juno")],
        )
        .unwrap();

    // check spot price is about 1 JUNO -> 0.666 wyJUNO
    let spot = suite
        .query_spot_price(&pair, &juno_info, &wy_juno_info)
        .unwrap();
    // this is within 0.01%
    assert_approx_eq!(
        spot * Uint128::new(1_000_000),
        Uint128::new(666666),
        "0.0001"
    );

    // aiming for a price above current will return None
    let amount = Uint128::new(100_000_000);
    let to_swap = suite
        .query_predict_spot_price(
            &pair,
            &juno_info,
            &wy_juno_info,
            amount,
            Decimal::percent(70),
            iterations,
        )
        .unwrap();
    assert_eq!(to_swap, None);

    // aiming for a price far below current will return full amount
    let swap_all = suite
        .query_predict_spot_price(
            &pair,
            &juno_info,
            &wy_juno_info,
            amount,
            Decimal::percent(60),
            iterations,
        )
        .unwrap();
    let swap_all = swap_all.unwrap();
    assert_eq!(swap_all, amount);

    // aiming for a price slightly below current will return some partial value
    let target = Decimal::permille(656);
    let to_swap = suite
        .query_predict_spot_price(&pair, &juno_info, &wy_juno_info, amount, target, iterations)
        .unwrap();
    // must be Some
    let to_swap = to_swap.unwrap();
    // must be less than amount
    assert!(to_swap < swap_all);

    // verify this value does lead to proper spot price after swap
    suite
        .swap(
            &pair,
            "sender",
            juno_info.with_balance(to_swap),
            None,
            None,
            Decimal::percent(5),
            None,
        )
        .unwrap();

    // check spot price is very close to desired (seems to be about 0.3% above... fee?)
    let spot = suite
        .query_spot_price(&pair, &juno_info, &wy_juno_info)
        .unwrap();
    // this is within 0.01%
    assert_approx_eq!(
        spot * Uint128::new(1_000_000),
        target * Uint128::new(1_000_000),
        "0.0001"
    );
}

/// Helper function that swaps until the target rate is reached.
pub fn arbitrage_to(
    suite: &mut Suite,
    pair: &Addr,
    offer_asset: &AssetInfo,
    mut target_rate: Decimal,
) {
    if !offer_asset.is_native_token() {
        // we have too much of the lsd token, so we swap it for the native one
        // but we need to invert the rate (since it is given as native tokens / lsd tokens)
        target_rate = Decimal::one() / target_rate;
    }

    let mut amount = Uint128::from(100_000_000_000_000u128);
    const TEN: Uint128 = Uint128::new(10u128);
    const MAX_AMT: Uint128 = Uint128::new(100u128);
    loop {
        let sim = suite
            .query_simulation(pair, offer_asset.with_balance(amount), None)
            .unwrap();

        if amount < MAX_AMT || (sim.return_amount + sim.commission_amount) * target_rate == amount {
            break;
        }
        if (sim.return_amount + sim.commission_amount) * target_rate <= amount {
            amount /= TEN;
            continue;
        }

        suite
            .mint("owner", offer_asset.with_balance(amount), "arbitrageur")
            .unwrap();

        suite
            .swap(
                pair,
                "arbitrageur",
                offer_asset.with_balance(amount),
                None,
                None,
                None,
                None,
            )
            .unwrap();
    }
}
