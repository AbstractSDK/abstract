use std::str::FromStr;

use cosmwasm_std::{assert_approx_eq, coin, Addr, Decimal, Fraction, Uint128};
use cw_multi_test::AppResponse;
use wyndex::pair::LsdInfo;
use wyndex::{
    asset::{Asset, AssetInfo, AssetInfoExt},
    factory::PairType,
    pair::StablePoolParams,
};

use crate::multitest::target_rate::arbitrage_to;

use super::suite::{Suite, SuiteBuilder};

const DAY: u64 = 24 * 60 * 60;

const TRADER: &str = "trader";

/// Simulates a year of trading where the exchange rate increases every day for different amp values.
/// This uses a constant trading volume per day.
#[test]
#[ignore = "only for finding good amp parameter"]
fn simulate_changing_rate() {
    let liquidity_discount = Decimal::percent(4);
    let tvl = 100_000_000_000_000_000u128; // total value locked in the pool
    let trade_volume = 1_000_000_000_000_000u128; // how much juno is traded

    let juno_info = AssetInfo::Native("juno".to_string());

    const AMPS: [u64; 13] = [
        100, 1000, 5000, 10_000, 15_000, 20_000, 50_000, 100_000, 200_000, 300_000, 400_000,
        500_000, 1_000_000,
    ];
    for amp in AMPS {
        // these will be measured
        let mut max_price_change = Decimal::zero();
        let mut max_slippage = Uint128::zero();

        let mut target_rate = Decimal::one() - liquidity_discount;
        let mut suite = SuiteBuilder::new()
            .with_funds(TRADER, &[coin(tvl / 2, "juno")])
            .with_initial_target_rate(target_rate)
            .build();
        let start_time = suite.app.block_info().time.seconds();

        let wy_juno = suite.instantiate_token("owner", "wyJUNO");
        let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

        // create the pair
        let pair = suite
            .create_pair_and_provide_liquidity(
                PairType::Lsd {},
                Some(StablePoolParams {
                    amp,
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

        suite.wait(DAY);

        // simulate whole year of trading
        for _ in 0..365 {
            let price = query_price(&mut suite, &pair, &wy_juno_info);

            // trade some amounts around
            let swap_response = suite
                .swap(
                    &pair,
                    TRADER,
                    juno_info.with_balance(trade_volume),
                    wy_juno_info.clone(),
                    None,
                    None,
                    None,
                )
                .unwrap();
            let spread = get_spread(swap_response);
            max_slippage = std::cmp::max(max_slippage, spread);

            let price_after_swap = query_price(&mut suite, &pair, &wy_juno_info);
            max_price_change = std::cmp::max(
                max_price_change,
                (std::cmp::max(price_after_swap, price) / std::cmp::min(price, price_after_swap))
                    - Decimal::one(),
            );

            let lsd_balance = suite.query_cw20_balance(TRADER, &wy_juno).unwrap();
            suite
                .swap(
                    &pair,
                    TRADER,
                    wy_juno_info.with_balance(lsd_balance),
                    juno_info.clone(),
                    None,
                    None,
                    None,
                )
                .unwrap();

            // update target rate
            target_rate = update_target_rate(&mut suite, start_time, liquidity_discount);
            suite.wait(DAY);

            // target rate was increased, so we arbitrage it by putting in more juno
            arbitrage_to(&mut suite, &pair, &juno_info, target_rate);
        }
        println!(
            "amp {}, max_slippage {}%, max_price_change {}%",
            amp,
            Decimal::from_ratio(max_slippage, trade_volume)
                * Decimal::from_atomics(100u128, 0).unwrap(),
            max_price_change * Decimal::from_atomics(100u128, 0).unwrap(),
        );
    }
}

/// This simulates the slippage of a trade at different prices relative to the target rate.
#[test]
#[ignore = "only for finding good amp parameter"]
fn simulate_slippage_vs_uniswap() {
    // input parameters
    let juno_amount = 50_000_000_000_000_000u128; // how much juno is in the pool
    let trade_volume = 1_000_000_000_000_000u128; // how much juno is traded

    let amps = [
        8u64, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
        30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    ];

    let price_diffs = [
        Decimal::percent(1),
        Decimal::percent(3),
        Decimal::percent(5),
    ];

    for price_diff in price_diffs {
        println!("amp,price_diff,direction,stableswap slippage,uniswap slippage");
        for amp in amps {
            let sub_results = compare_to_uniswap(amp, juno_amount, trade_volume, price_diff, false);
            println!(
                "{},-{},Juno => wyJuno,{}%,{}%",
                amp,
                price_diff,
                Decimal::from_ratio(sub_results.stable_juno_to_lsd_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
                Decimal::from_ratio(sub_results.uniswap_juno_to_lsd_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
            );
            println!(
                "{},-{},wyJuno => Juno,{}%,{}%",
                amp,
                price_diff,
                Decimal::from_ratio(sub_results.stable_lsd_to_juno_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
                Decimal::from_ratio(sub_results.uniswap_lsd_to_juno_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
            );

            let add_results = compare_to_uniswap(amp, juno_amount, trade_volume, price_diff, true);
            println!(
                "{},{},Juno => wyJuno,{}%,{}%",
                amp,
                price_diff,
                Decimal::from_ratio(add_results.stable_juno_to_lsd_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
                Decimal::from_ratio(add_results.uniswap_juno_to_lsd_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
            );
            println!(
                "{},{},wyJuno => Juno,{}%,{}%",
                amp,
                price_diff,
                Decimal::from_ratio(add_results.stable_lsd_to_juno_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
                Decimal::from_ratio(add_results.uniswap_lsd_to_juno_slippage, trade_volume)
                    * Decimal::from_atomics(100u128, 0).unwrap(),
            );
        }
    }
}

struct SwapSlippage {
    stable_juno_to_lsd_slippage: Uint128,
    uniswap_juno_to_lsd_slippage: Uint128,
    stable_lsd_to_juno_slippage: Uint128,
    uniswap_lsd_to_juno_slippage: Uint128,
}

/// This simulates the slippage of a trade at the given difference to the target rate for the given amp
fn compare_to_uniswap(
    amp: u64,
    juno_amount: u128,
    trade_volume: u128,
    price_diff: Decimal,
    add: bool,
) -> SwapSlippage {
    // the exchange rate of the lsd token (we keep this constant for this simulation)
    let expected_target_rate = Decimal::one();
    let actual_target_rate = if add {
        Decimal::one() + price_diff
    } else {
        Decimal::one() - price_diff
    };

    const TRADER: &str = "trader";
    let juno_info = AssetInfo::Native("juno".to_string());

    // find lsd amount for stable swap
    let lsd_amount = binary_search_lsd_provision(
        amp,
        expected_target_rate,
        actual_target_rate,
        &juno_info.with_balance(juno_amount),
    )
    .u128();

    let mut suite = SuiteBuilder::new()
        .with_funds(TRADER, &[coin(juno_amount, "juno")])
        .with_initial_target_rate(expected_target_rate)
        .build();

    // create lsd token for the stable pair
    let wy_juno = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno_info = AssetInfo::Token(wy_juno.to_string());
    // and one for the uniswap pair
    let wy_juno2 = suite.instantiate_token("owner", "wyJUNO");
    let wy_juno2_info = AssetInfo::Token(wy_juno2.to_string());

    // create the stable pair
    let stable_pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Lsd {},
            Some(StablePoolParams {
                amp,
                owner: Some("owner".to_string()),
                lsd: Some(LsdInfo {
                    asset: wy_juno_info.clone(),
                    hub: suite.mock_hub.to_string(),
                    target_rate_epoch: DAY,
                }),
            }),
            (juno_info.clone(), juno_amount),
            (wy_juno_info.clone(), lsd_amount),
            vec![coin(juno_amount, "juno")],
        )
        .unwrap();

    // create the uniswap pair for comparison
    let uniswap_pair = suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            None,
            (juno_info.clone(), juno_amount),
            (
                wy_juno2_info.clone(),
                Uint128::new(juno_amount)
                    .multiply_ratio(
                        actual_target_rate.denominator(),
                        actual_target_rate.numerator(),
                    )
                    .u128(),
            ), // juno_amount / lsd_amount = actual_target_rate <=> lsd_amount = juno_amount / actual_target_rate
            vec![coin(juno_amount, "juno")],
        )
        .unwrap();

    // check that the prices are correct
    assert_approx_eq!(
        query_price(&mut suite, &stable_pair, &wy_juno_info).numerator(),
        actual_target_rate.numerator(),
        "0.000002"
    );
    assert_approx_eq!(
        query_price(&mut suite, &uniswap_pair, &wy_juno2_info).numerator(),
        actual_target_rate.numerator(),
        "0.000002"
    );

    // using simulation here to avoid `MaxSpreadAssertion` error
    let sim = suite
        .query_simulation(
            &stable_pair,
            juno_info.with_balance(trade_volume),
            wy_juno_info.clone(),
        )
        .unwrap();
    // calculate slippage as: expected amount minus actual amount
    let swap_output = sim.return_amount + sim.commission_amount;
    let juno_wy_juno_slippage =
        Uint128::new(trade_volume).saturating_sub(swap_output * actual_target_rate);

    let sim = suite
        .query_simulation(
            &uniswap_pair,
            juno_info.with_balance(trade_volume),
            wy_juno2_info.clone(),
        )
        .unwrap();
    // calculate slippage as: expected amount minus actual amount
    let xyk_swap_output = sim.return_amount + sim.commission_amount;
    let xyk_juno_wy_juno_slippage =
        Uint128::new(trade_volume).saturating_sub(xyk_swap_output * actual_target_rate);

    let sim = suite
        .query_simulation(
            &stable_pair,
            wy_juno_info.with_balance(trade_volume),
            juno_info.clone(),
        )
        .unwrap();
    // calculate slippage as: expected amount minus actual amount
    // we expect to receive the fair market price, which is `actual_target_rate`
    let optimal_output = Uint128::new(trade_volume) * actual_target_rate;
    let stable_swap_output = sim.return_amount + sim.commission_amount;
    let stable_slippage = optimal_output.saturating_sub(stable_swap_output);

    let sim = suite
        .query_simulation(
            &uniswap_pair,
            wy_juno2_info.with_balance(trade_volume),
            juno_info,
        )
        .unwrap();
    // calculate slippage as: expected amount minus actual amount
    // we expect to receive the fair market price, which is `actual_target_rate`
    let xyk_swap_output = sim.return_amount + sim.commission_amount;
    let xyk_slippage = optimal_output.saturating_sub(xyk_swap_output);
    assert_eq!(sim.spread_amount, xyk_slippage);

    SwapSlippage {
        stable_juno_to_lsd_slippage: juno_wy_juno_slippage,
        uniswap_juno_to_lsd_slippage: xyk_juno_wy_juno_slippage,
        stable_lsd_to_juno_slippage: stable_slippage,
        uniswap_lsd_to_juno_slippage: xyk_slippage,
    }
}

fn update_target_rate(suite: &mut Suite, start_time: u64, liquidity_discount: Decimal) -> Decimal {
    // juno APR currently is around 35%, so we expect 35% / 365
    let daily_apr = Decimal::percent(35) * Decimal::from_ratio(1u128, 365u128);
    let elapsed_time = suite.app.block_info().time.seconds() - start_time;
    // compound interest formula gives us the expected exchange rate after elapsed_time
    let expected_value = (Decimal::one() + daily_apr).pow((elapsed_time / DAY) as u32);
    // to get the target rate, we apply the liquidity discount
    let target_rate = expected_value * (Decimal::one() - liquidity_discount);

    suite.change_target_value(target_rate).unwrap();
    target_rate
}

/// Query price (including commission) for one LSD token
fn query_price(suite: &mut Suite, pair: &Addr, input_asset: &AssetInfo) -> Decimal {
    // query price for one LSD token
    let simulation = suite
        .query_simulation(pair, input_asset.with_balance(1_000_000u128), None)
        .unwrap();

    Decimal::from_ratio(
        simulation.return_amount + simulation.commission_amount,
        1_000_000u128,
    )
}

fn get_spread(swap_response: AppResponse) -> Uint128 {
    Uint128::from_str(get_attribute(&swap_response, "spread_amount").unwrap()).unwrap()
}

fn get_attribute<'a>(swap_response: &'a AppResponse, key: &str) -> Option<&'a str> {
    swap_response
        .events
        .iter()
        .find_map(|e| e.attributes.iter().find(|a| a.key == key))
        .map(|a| a.value.as_str())
}

/// Uses binary search to find the amount of LSD tokens to provide together with the given amount ot juno
/// that will result in the given price.
fn binary_search_lsd_provision(
    amp: u64,
    expected_target_rate: Decimal,
    price: Decimal,
    juno: &Asset,
) -> Uint128 {
    binary_search(
        Uint128::one(),
        juno.amount * Uint128::new(1000),
        price,
        |lsd_amount| {
            let mut suite = SuiteBuilder::new()
                .with_funds(TRADER, &[coin(juno.amount.u128(), "juno")])
                .with_initial_target_rate(expected_target_rate)
                .build();

            let wy_juno = suite.instantiate_token("owner", "wyJUNO");
            let wy_juno_info = AssetInfo::Token(wy_juno.to_string());

            // create the pair
            let pair = suite
                .create_pair_and_provide_liquidity(
                    PairType::Lsd {},
                    Some(StablePoolParams {
                        amp,
                        owner: Some("owner".to_string()),
                        lsd: Some(LsdInfo {
                            asset: wy_juno_info.clone(),
                            hub: suite.mock_hub.to_string(),
                            target_rate_epoch: DAY,
                        }),
                    }),
                    (juno.info.clone(), juno.amount.u128()),
                    (wy_juno_info.clone(), lsd_amount.u128()),
                    vec![coin(juno.amount.u128(), "juno")],
                )
                .unwrap();

            query_price(&mut suite, &pair, &wy_juno_info)
        },
    )
}

/// A function that does binary search with a minimum and maximum Uint128 value
fn binary_search(
    mut min: Uint128,
    mut max: Uint128,
    find: Decimal,
    f: impl Fn(Uint128) -> Decimal,
) -> Uint128 {
    const TWO: Uint128 = Uint128::new(2);
    let mut half = max / TWO + min / TWO;
    let mut current = f(half);

    while min <= max {
        match current.cmp(&find) {
            std::cmp::Ordering::Equal => return half,
            std::cmp::Ordering::Greater => min = half + Uint128::one(),
            std::cmp::Ordering::Less => max = half - Uint128::one(),
        }
        half = max / TWO + min / TWO;
        current = f(half);
    }
    half
}
