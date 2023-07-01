use super::suite::SuiteBuilder;

use cosmwasm_std::testing::MockApi;
use cosmwasm_std::{assert_approx_eq, coin, Decimal, Fraction, Uint128};
use wyndex::pair::{add_referral, take_referral};
use wyndex::querier::query_factory_config;

use crate::error::ContractError;
use crate::msg::{SwapOperation, MAX_SWAP_OPERATIONS};
use wyndex::asset::{AssetInfo, AssetInfoExt, AssetInfoValidated};
use wyndex::factory::PairType;

#[test]
fn must_provide_operations() {
    let ujuno = "ujuno";
    let user = "user";

    let mut suite = SuiteBuilder::new()
        .with_funds(user, &[coin(100_000, ujuno)])
        .build();

    let err = suite
        .swap_operations(user, coin(100_000u128, ujuno), vec![])
        .unwrap_err();
    assert_eq!(
        ContractError::MustProvideOperations {},
        err.downcast().unwrap()
    );
}

#[test]
fn single_swap() {
    let ujuno = "ujuno";
    let user = "user";

    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "wynd");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token.to_string()), 100_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
            vec![coin(100_000_000, ujuno)],
        )
        .unwrap();

    // Mint some cw20 for user to exchange
    suite
        .mint_cw20(&owner, &token, 100_000_000u128, user)
        .unwrap();

    suite
        .swap_operations_cw20(
            user,
            &token,
            100_000u128,
            vec![SwapOperation::WyndexSwap {
                offer_asset_info: AssetInfo::Token(token.to_string()),
                ask_asset_info: AssetInfo::Native(ujuno.to_string()),
            }],
        )
        .unwrap();

    assert_eq!(suite.query_balance(user, ujuno).unwrap(), 99_900u128);
}

#[test]
fn multiple_swaps() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";

    let mut suite = SuiteBuilder::new()
        .with_funds(user, &[coin(100_000, ujuno)])
        .build();

    let owner = suite.owner.clone();

    let token_a = suite.instantiate_token(&owner, "wynd");
    let token_b = suite.instantiate_token(&owner, "ueco");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_a.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, ujuno)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_a.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_b.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();

    suite
        .swap_operations(
            user,
            coin(100_000u128, "ujuno"),
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native(ujuno.to_string()),
                    ask_asset_info: AssetInfo::Token(token_a.to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token(token_a.to_string()),
                    ask_asset_info: AssetInfo::Native(uluna.to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native(uluna.to_string()),
                    ask_asset_info: AssetInfo::Token(token_b.to_string()),
                },
            ],
        )
        .unwrap();

    assert_eq!(
        suite.query_cw20_balance(user, &token_b).unwrap(),
        99_970u128
    );
}

#[test]
fn multi_hop_does_not_enforce_spread_assetion() {
    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner.clone();

    let token_a = suite.instantiate_token(&owner, "TOKA");
    let token_b = suite.instantiate_token(&owner, "TOKB");
    let token_c = suite.instantiate_token(&owner, "TOKC");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_a.to_string()), 100_000_000_000u128),
            (AssetInfo::Token(token_b.to_string()), 100_000_000_000u128),
            vec![],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Stable {},
            (AssetInfo::Token(token_b.to_string()), 1_000_000_000_000u128),
            (AssetInfo::Token(token_c.to_string()), 1_000_000_000_000u128),
            vec![],
        )
        .unwrap();

    let user = "user";
    suite
        .mint_cw20(&owner, &token_a, 100_000_000_000u128, user)
        .unwrap();

    // Triggering swap with a huge spread fees
    suite
        .swap_operations_cw20(
            user,
            &token_a,
            50_000_000_000u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token(token_a.to_string()),
                    ask_asset_info: AssetInfo::Token(token_b.to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token(token_b.to_string()),
                    ask_asset_info: AssetInfo::Token(token_c.to_string()),
                },
            ],
        )
        .unwrap();

    // However, single hop will still enforce spread assertion
    let err = suite
        .swap_operations_cw20(
            user,
            &token_a,
            50_000_000_000u128,
            vec![SwapOperation::WyndexSwap {
                offer_asset_info: AssetInfo::Token(token_a.to_string()),
                ask_asset_info: AssetInfo::Token(token_b.to_string()),
            }],
        )
        .unwrap_err();
    assert_eq!(
        wyndex::pair::ContractError::MaxSpreadAssertion {},
        err.downcast().unwrap()
    )
}

#[test]
fn query_buy_with_routes() {
    let ujuno = "ujuno";
    let uluna = "uluna";

    let mut suite = SuiteBuilder::new().build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "TOKA");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Native(ujuno.to_owned()), 1_000_000_000u128),
            (AssetInfo::Token(token.to_string()), 1_000_000_000u128),
            vec![coin(1_000_000_000, ujuno)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            (AssetInfo::Token(token.to_string()), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();

    let response = suite
        .query_simulate_swap_operations(
            1_000_000u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("ujuno".to_owned()),
                    ask_asset_info: AssetInfo::Token(token.to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token(token.to_string()),
                    ask_asset_info: AssetInfo::Native("uluna".to_owned()),
                },
            ],
        )
        .unwrap();
    // ideal amount for first swap is `1_000_000`, but because of spread it's `999_000`
    // starting with that, the ideal amount for the second swap is `999_000`, but because of spread it's `998_002`
    assert_eq!(response.amount.u128(), 998_002u128);
    assert_approx_eq!(
        response.spread.numerator(),
        (Decimal::one() - Decimal::from_ratio(998_002u128, 1_000_000u128)).numerator(),
        "0.000000000000001"
    );

    // reverse swap
    let response = suite
        .query_simulate_reverse_swap_operations(
            998_002u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native("ujuno".to_owned()),
                    ask_asset_info: AssetInfo::Token(token.to_string()),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Token(token.to_string()),
                    ask_asset_info: AssetInfo::Native("uluna".to_owned()),
                },
            ],
        )
        .unwrap();
    // ideal amount for second swap is 999_000,
    // but we only get 998_002 because of spread
    // ideal amount for first swap is 1_000_000, but we only get 999_000 because of spread
    // input amount should be (approximately) 1_000_000
    assert_approx_eq!(response.amount.u128(), 1_000_000u128, "0.00001");
    assert_approx_eq!(
        response.spread.numerator(),
        (Decimal::one() - Decimal::from_ratio(998_002u128, 1_000_000u128)).numerator(),
        "0.01"
    );
}

#[test]
fn simulation_with_fee() {
    let ujuno = "ujuno";
    let uluna = "uluna";

    // fee is 1% for both tokens
    let mut suite = SuiteBuilder::new().with_fees(100, 50).build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "TOKA");

    let token_info = AssetInfo::Token(token.to_string());
    let ujuno_info = AssetInfo::Native(ujuno.to_owned());
    let uluna_info = AssetInfo::Native(uluna.to_owned());

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (ujuno_info.clone(), 1_000_000_000u128),
            (token_info.clone(), 1_000_000_000u128),
            vec![coin(1_000_000_000, ujuno)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (uluna_info.clone(), 1_000_000_000u128),
            (token_info.clone(), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();

    let response = suite
        .query_simulate_swap_operations(
            1_000_000u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: ujuno_info.clone(),
                    ask_asset_info: token_info.clone(),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: token_info.clone(),
                    ask_asset_info: uluna_info.clone(),
                },
            ],
        )
        .unwrap();

    // ideal amount for first swap is `1_000_000`, but because of spread (1_000) it's `999_000` and
    // the fee is `999_000 * 1% = 9_990`, so it returns `989_010`
    // the ideal amount for the second swap is `989_010`, but because of spread (978) it's `988_032` and
    // the fee is `988_032 * 1% = 9_880`, so it returns `978_152`
    assert_eq!(response.amount.u128(), 978_152u128);
    assert_eq!(
        response.spread,
        (Decimal::one()
            - (Decimal::from_ratio(999_000u128, 1_000_000u128)
                * Decimal::from_ratio(988_032u128, 989_010u128)))
    );
    // validate absolute amounts
    let api = MockApi::default();
    let ujuno_val = ujuno_info.validate(&api).unwrap();
    let token_val = token_info.validate(&api).unwrap();
    let uluna_val = uluna_info.validate(&api).unwrap();
    assert_eq!(
        response.spread_amounts,
        vec![
            token_val.with_balance(1000u128),
            uluna_val.with_balance(978u128)
        ],
    );
    assert_eq!(
        response.commission_amounts,
        vec![
            token_val.with_balance(9_990u128),
            uluna_val.with_balance(9_880u128)
        ],
        "999_000 * 1% = 9_990, 988_032 * 1% = 9_880"
    );
    assert_eq!(
        response.referral_amount,
        ujuno_val.with_balance(0u128),
        "no referral"
    );

    // now another swap with referral commission
    let response = suite
        .query_simulate_swap_operations_ref(
            1_000_000u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: ujuno_info.clone(),
                    ask_asset_info: token_info.clone(),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: token_info.clone(),
                    ask_asset_info: uluna_info.clone(),
                },
            ],
            Decimal::percent(1),
        )
        .unwrap();

    // ideal amount for first swap is `1_000_000 - 10_000 = 990_000`, but because of spread (980) it's `989_020` and
    // the fee is `989_020 * 1% = 9_890`, so it returns `979_130`
    // the ideal amount for the second swap is `979_130`, but because of spread (958) it's `978_172` and
    // the fee is `978_172 * 1% = 9_781`, so it returns `968_391`
    assert_eq!(response.amount.u128(), 968_391u128);
    assert_eq!(
        response.spread,
        (Decimal::one()
            - (Decimal::from_ratio(989_020u128, 990_000u128)
                * Decimal::from_ratio(978_172u128, 979_130u128)))
    );
    // validate absolute amounts
    assert_eq!(
        response.spread_amounts,
        vec![
            token_val.with_balance(980u128),
            uluna_val.with_balance(958u128)
        ],
    );
    assert_eq!(
        response.commission_amounts,
        vec![
            token_val.with_balance(9_890u128),
            uluna_val.with_balance(9_781u128)
        ],
        "989_020 * 1% = 9_890, 978_172 * 1% = 9_781"
    );
    assert_eq!(response.referral_amount, ujuno_val.with_balance(10_000u128));

    // now same swap, but in reverse
    let response = suite
        .query_simulate_reverse_swap_operations_ref(
            968_391u128,
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: ujuno_info,
                    ask_asset_info: token_info.clone(),
                },
                SwapOperation::WyndexSwap {
                    offer_asset_info: token_info,
                    ask_asset_info: uluna_info,
                },
            ],
            Decimal::percent(1),
        )
        .unwrap();
    // should result in approximately 1_000_000 with the same spread
    assert_approx_eq!(response.amount.u128(), 1_000_000u128, "0.00001");
    assert_approx_eq!(
        response.spread.numerator(),
        (Decimal::one()
            - (Decimal::from_ratio(989_020u128, 990_000u128)
                * Decimal::from_ratio(978_172u128, 979_130u128)))
        .numerator(),
        "0.01"
    );
}

#[test]
fn assert_minimum_receive_native_tokens() {
    let ujuno = "ujuno";

    let mut suite = SuiteBuilder::new()
        .with_funds("user", &[coin(1_000_000, ujuno)])
        .build();

    // that works
    suite
        .assert_minimum_receive("user", AssetInfo::Native(ujuno.to_owned()), 1_000_000u128)
        .unwrap();

    let err = suite
        .assert_minimum_receive("user", AssetInfo::Native(ujuno.to_owned()), 1_000_001u128)
        .unwrap_err();
    assert_eq!(
        ContractError::AssertionMinimumReceive {
            receive: Uint128::new(1_000_001),
            amount: Uint128::new(1_000_000)
        },
        err.downcast().unwrap()
    );
}

#[test]
fn assert_minimum_receive_cw20_tokens() {
    let mut suite = SuiteBuilder::new().build();

    let token = suite.instantiate_token("owner", "TOKA");
    suite
        .mint_cw20("owner", &token, 1_000_000u128, "user")
        .unwrap();

    // that works
    suite
        .assert_minimum_receive("user", AssetInfo::Token(token.to_string()), 1_000_000u128)
        .unwrap();

    let err = suite
        .assert_minimum_receive("user", AssetInfo::Token(token.to_string()), 1_000_001u128)
        .unwrap_err();
    assert_eq!(
        ContractError::AssertionMinimumReceive {
            receive: Uint128::new(1_000_001),
            amount: Uint128::new(1_000_000)
        },
        err.downcast().unwrap()
    );
}

#[test]
fn maximum_receive_swap_operations() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";

    let mut suite = SuiteBuilder::new()
        .with_funds(user, &[coin(100_000, ujuno)])
        .build();

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Native(ujuno.to_owned()), 1_000_000_000u128),
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, ujuno), coin(1_000_000_000, uluna)],
        )
        .unwrap();

    let err = suite
        .swap_operations(
            user,
            coin(100_000u128, "ujuno"),
            vec![
                SwapOperation::WyndexSwap {
                    offer_asset_info: AssetInfo::Native(ujuno.to_string()),
                    ask_asset_info: AssetInfo::Native(uluna.to_owned()),
                };
                MAX_SWAP_OPERATIONS + 1
            ],
        )
        .unwrap_err();
    assert_eq!(ContractError::SwapLimitExceeded {}, err.downcast().unwrap());
}

/// Tests the helper functions for calculating referral commission.
/// Specifically, it tests the property that [`take_referral`] reverses the effect of [`add_referral`].
#[test]
fn take_add_referral() {
    // setup contracts (only factory is relevant for us)
    let suite = SuiteBuilder::new().build();
    let querier = suite.app.wrap();

    // just some random amounts
    let test_amounts = vec![
        0, 1, 15, 100, 1000, 10000, 100000, 234, 43806, 20420, 2, 345, 63, 354,
    ];

    for offer_amount in test_amounts {
        let offer_asset = AssetInfoValidated::Native("test".to_string()).with_balance(offer_amount);

        // add referral on top
        let (mut with_referral, _) = add_referral(
            &querier,
            &suite.factory,
            true,
            Some(Decimal::percent(1)),
            offer_asset,
        )
        .unwrap();

        // take it away again
        let factory_config = query_factory_config(&querier, &suite.factory).unwrap();
        take_referral(
            &factory_config,
            Some(Decimal::percent(1)),
            &mut with_referral,
        )
        .unwrap();

        // should be the same as before
        assert_eq!(with_referral.amount.u128(), offer_amount);
    }
}

#[test]
fn referral_single() {
    let ujuno = "ujuno";
    let user = "user";
    let referral = "referral";

    let mut suite = SuiteBuilder::new()
        .with_max_referral_commission(Decimal::percent(1))
        .build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "wynd");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token.to_string()), 100_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
            vec![coin(100_000_000, ujuno)],
        )
        .unwrap();

    // Mint some cw20 tokens
    suite.mint_cw20(&owner, &token, 101_010u128, user).unwrap();

    // single router swap with referral
    // amount is chosen such that it will be 100_000 after referral commission is deducted
    suite
        .swap_operations_cw20_ref(
            user,
            &token,
            101_010u128,
            vec![SwapOperation::WyndexSwap {
                offer_asset_info: AssetInfo::Token(token.to_string()),
                ask_asset_info: AssetInfo::Native(ujuno.to_string()),
            }],
            referral.to_string(),
            Decimal::percent(1),
        )
        .unwrap();
    assert_eq!(suite.query_balance(user, ujuno).unwrap(), 99_900u128);

    // make sure referral got the commission
    assert_eq!(
        suite.query_cw20_balance(referral, &token).unwrap(),
        1010u128
    );
}

#[test]
fn referral_multiple() {
    let ujuno = "ujuno";
    let uluna = "uluna";
    let user = "user";
    let referral = "referral";

    let mut suite = SuiteBuilder::new()
        .with_funds(user, &[coin(101_010, ujuno)])
        .build();

    let owner = suite.owner.clone();

    let token_a = suite.instantiate_token(&owner, "wynd");
    let token_b = suite.instantiate_token(&owner, "ueco");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_a.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, ujuno)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_a.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token_b.to_string()), 1_000_000_000u128),
            (AssetInfo::Native(uluna.to_owned()), 1_000_000_000u128),
            vec![coin(1_000_000_000, uluna)],
        )
        .unwrap();

    let operations = vec![
        SwapOperation::WyndexSwap {
            offer_asset_info: AssetInfo::Native(ujuno.to_string()),
            ask_asset_info: AssetInfo::Token(token_a.to_string()),
        },
        SwapOperation::WyndexSwap {
            offer_asset_info: AssetInfo::Token(token_a.to_string()),
            ask_asset_info: AssetInfo::Native(uluna.to_string()),
        },
        SwapOperation::WyndexSwap {
            offer_asset_info: AssetInfo::Native(uluna.to_string()),
            ask_asset_info: AssetInfo::Token(token_b.to_string()),
        },
    ];

    // query the result first, so we can compare it with the result after referral
    let query_result = suite
        .query_simulate_swap_operations_ref(101_010u128, operations.clone(), Decimal::percent(1))
        .unwrap()
        .amount
        .u128();

    suite
        .swap_operations_ref(
            user,
            coin(101_010u128, "ujuno"),
            operations,
            referral.to_string(),
            Decimal::percent(1),
        )
        .unwrap();

    assert_eq!(
        suite.query_cw20_balance(user, &token_b).unwrap(),
        query_result
    );

    assert_eq!(
        suite.query_cw20_balance(user, &token_b).unwrap(),
        99_970u128
    );

    // make sure referral got the commission
    assert_eq!(suite.query_balance(referral, ujuno).unwrap(), 1010u128);
}

#[test]
fn invalid_referral_commission() {
    let ujuno = "ujuno";
    let user = "user";
    let referral = "referral";

    let mut suite = SuiteBuilder::new()
        .with_max_referral_commission(Decimal::percent(1))
        .build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "wynd");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token.to_string()), 100_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
            vec![coin(100_000_000, ujuno)],
        )
        .unwrap();

    // Mint some cw20 tokens to swap
    suite
        .mint_cw20(&owner, &token, 100_000_000u128, user)
        .unwrap();

    // single router swap with referral, but commission too high
    let err = suite
        .swap_operations_cw20_ref(
            user,
            &token,
            100_000,
            vec![SwapOperation::WyndexSwap {
                offer_asset_info: AssetInfo::Token(token.to_string()),
                ask_asset_info: AssetInfo::Native(ujuno.to_string()),
            }],
            referral.to_string(),
            Decimal::percent(2),
        )
        .unwrap_err();

    assert_eq!(
        "Referral commission is higher than the allowed maximum",
        err.root_cause().to_string()
    );
}

#[test]
fn referral_commission_zero() {
    let ujuno = "ujuno";
    let user = "user";
    let referral = "referral";

    let mut suite = SuiteBuilder::new()
        .with_max_referral_commission(Decimal::percent(1))
        .build();

    let owner = suite.owner.clone();

    let token = suite.instantiate_token(&owner, "wynd");

    // create LP for just instantiated tokens
    suite
        .create_pair_and_provide_liquidity(
            PairType::Xyk {},
            (AssetInfo::Token(token.to_string()), 100_000_000u128),
            (AssetInfo::Native(ujuno.to_owned()), 100_000_000u128),
            vec![coin(100_000_000, ujuno)],
        )
        .unwrap();

    // Mint some cw20 tokens to swap
    suite.mint_cw20(&owner, &token, 1000u128, user).unwrap();

    // single router swap with referral, but zero commission, should not fail
    suite
        .swap_operations_cw20_ref(
            user,
            &token,
            1000,
            vec![SwapOperation::WyndexSwap {
                offer_asset_info: AssetInfo::Token(token.to_string()),
                ask_asset_info: AssetInfo::Native(ujuno.to_string()),
            }],
            referral.to_string(),
            Decimal::from_ratio(1u128, 10_000u128),
        )
        .unwrap();

    // make sure referral commission is zero
    assert_eq!(suite.query_balance(referral, ujuno).unwrap(), 0u128);
}
