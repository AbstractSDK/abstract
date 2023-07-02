use cosmwasm_std::{Addr, Decimal, Uint128};
use cw20::{Cw20Coin, MinterResponse};
use cw_multi_test::Executor;

use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use wyndex::asset::{AssetInfo, AssetInfoExt, AssetInfoValidated};

use crate::{multitest::suite::SuiteBuilder, ContractError};

use super::suite::{contract_token, juno, SEVEN_DAYS};

const UNBONDER: &str = "unbonder";
const ADMIN: &str = "admin";

#[test]
fn execute_unbond_all_case() {
    let mut suite = SuiteBuilder::new().with_unbonder(UNBONDER).build();

    // Random account cannot execute unbond all.
    let err = suite.execute_unbond_all("fake_unbonder").unwrap_err();

    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap(),);

    assert!(!suite.query_unbond_all().unwrap());

    // Unbonder can execute unbond all.
    suite.execute_unbond_all(UNBONDER).unwrap();

    assert!(suite.query_unbond_all().unwrap());
}

#[test]
fn execute_stop_unbond_all_case() {
    let mut suite = SuiteBuilder::new()
        .with_unbonder(UNBONDER)
        .with_admin(ADMIN)
        .build();

    // Fails to stop if flag is false both for unbonder and admin.
    let mut err = suite.execute_stop_unbond_all(UNBONDER).unwrap_err();

    assert_eq!(ContractError::FlagAlreadySet {}, err.downcast().unwrap(),);

    err = suite.execute_stop_unbond_all(ADMIN).unwrap_err();

    assert_eq!(ContractError::FlagAlreadySet {}, err.downcast().unwrap(),);

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Fails with unauthorized
    err = suite.execute_stop_unbond_all("user").unwrap_err();

    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap(),);

    // Unbonder can stop unbond all
    suite.execute_stop_unbond_all(UNBONDER).unwrap();

    suite.execute_unbond_all(UNBONDER).unwrap();

    // Admin can stop unbond all
    suite.execute_stop_unbond_all(ADMIN).unwrap();
}

#[test]
fn delegate_and_unbond_with_unbond_all() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Delegate half of the tokens for 7 days (default with None).
    suite.delegate(user, 50_000u128, None).unwrap();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Unbond with unbond all flag to true.
    suite.unbond(user, 50_000u128, None).unwrap();

    // Staking contract has no token since sent back to user.
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0u128);

    // Total stake is zero.
    assert_eq!(suite.query_total_staked().unwrap(), 0u128);

    // No claims
    let claims = suite.query_claims(user).unwrap();
    assert_eq!(claims.len(), 0);

    assert_eq!(
        suite.query_balance_vesting_contract(user).unwrap(),
        100_000u128
    );
}

#[test]
fn single_delegate_unbond_and_claim_with_unbond_all() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Delegate half of the tokens for 7 days (default with None).
    suite.delegate(user, 50_000u128, None).unwrap();

    // Unbond.
    suite.unbond(user, 25_000u128, None).unwrap();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Staking contract has all tokens previously deposited
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 50_000u128);

    // Staking tokens are half of the delegated
    assert_eq!(suite.query_total_staked().unwrap(), 25_000u128);

    // Claim is there since made before unbond all.
    let claims = suite.query_claims(user).unwrap();
    assert_eq!(claims.len(), 1);

    // Free locked tokens.
    suite.update_time(SEVEN_DAYS * 2);
    suite.claim(user).unwrap();

    // User has not delegated tokens + delegated and then unbonded.
    assert_eq!(
        suite.query_balance_vesting_contract(user).unwrap(),
        75_000u128
    );
}

#[test]
fn multiple_delegate_unbond_and_claim_with_unbond_all() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![SEVEN_DAYS, SEVEN_DAYS * 3])
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Delegate half of the tokens for 7 days (default with None).
    suite.delegate(user, 50_000u128, SEVEN_DAYS).unwrap();

    // Delegate half of the tokens for 21 days.
    suite.delegate(user, 50_000u128, SEVEN_DAYS * 3).unwrap();

    // Unbond.
    suite.unbond(user, 25_000u128, None).unwrap();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Staking contract has all initial tokens.
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 100_000u128);

    // Tokens in stake are 100_000 minus unbonded.
    assert_eq!(suite.query_total_staked().unwrap(), 75_000u128);

    // Claim is there since made before unbond all.
    let claims = suite.query_claims(user).unwrap();
    assert_eq!(claims.len(), 1);

    suite.update_time(SEVEN_DAYS * 2);
    suite.claim(user).unwrap();

    // User claims only tokens unbonded before unbond all.
    assert_eq!(
        suite.query_balance_vesting_contract(user).unwrap(),
        25_000u128
    );

    // Unbond tokens delegated for 21 days.
    suite.unbond(user, 25_000u128, SEVEN_DAYS * 3).unwrap();

    // No claims
    let claims = suite.query_claims(user).unwrap();
    assert_eq!(claims.len(), 0);

    // User has previously claimed tokens + unbonded tokens from 21 days.
    assert_eq!(
        suite.query_balance_vesting_contract(user).unwrap(),
        50_000u128
    );

    // Staking contract has half available tokens.
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 50_000u128);
}

#[test]
fn delegate_with_unbond_all_flag() {
    let user = "user";
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Cannot delegate if unbond all.
    let err = suite.delegate(user, 50_000u128, None).unwrap_err();
    assert_eq!(
        ContractError::CannotDelegateIfUnbondAll {},
        err.downcast().unwrap()
    );
}

#[test]
fn delegate_as_with_unbond_all_flag() {
    let user = "factory";
    let user2 = "client";
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Cannot delegate through cw20 contract if unbond all.
    let err = suite
        .delegate_as(user, 50_000u128, None, Some(user2))
        .unwrap_err();

    assert_eq!(
        ContractError::CannotDelegateIfUnbondAll {},
        err.downcast().unwrap()
    );
}

#[test]
fn mass_delegation_with_unbond_all_flag() {
    let user = "factory";
    let user2 = "client";
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Cannot mass delegate if unbond all.
    let err = suite
        .mass_delegate(user, 50_000u128, None, &[(user2, 50_000u128)])
        .unwrap_err();

    assert_eq!(
        ContractError::CannotDelegateIfUnbondAll {},
        err.downcast().unwrap()
    );
}

#[test]
fn rebond_with_unbond_all_flag() {
    let user = "user";
    let new_unbonding_period = 1000u64;
    let old_unbonding_period = 5000u64;
    let mut suite = SuiteBuilder::new()
        .with_initial_balances(vec![(user, 100_000)])
        .with_unbonder(UNBONDER)
        .build();

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Rebond results in error.
    let err = suite
        .rebond(user, 2_000u128, old_unbonding_period, new_unbonding_period)
        .unwrap_err();

    assert_eq!(
        ContractError::CannotRebondIfUnbondAll {},
        err.downcast().unwrap()
    );
}

#[test]
fn multiple_distribution_flows() {
    let user = "user";
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![(user, 100_000)])
        .with_admin("admin")
        .with_unbonder(UNBONDER)
        .with_native_balances("juno", vec![(user, 1200)])
        .build();

    // Create CW20 token.
    let token_id = suite.app.store_code(contract_token());
    let wynd_token = suite
        .app
        .instantiate_contract(
            token_id,
            Addr::unchecked("admin"),
            &Cw20InstantiateMsg {
                name: "wynd-token".to_owned(),
                symbol: "WYND".to_owned(),
                decimals: 9,
                initial_balances: vec![Cw20Coin {
                    // member4 gets some to distribute
                    address: "user".to_owned(),
                    amount: Uint128::from(500u128),
                }],
                mint: Some(MinterResponse {
                    minter: "minter".to_owned(),
                    cap: None,
                }),
                marketing: None,
            },
            &[],
            "vesting",
            None,
        )
        .unwrap();

    // Distribution flow for native and CW20 tokens.
    suite
        .create_distribution_flow(
            "admin",
            user,
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite
        .create_distribution_flow(
            "admin",
            user,
            AssetInfo::Token(wynd_token.to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite.delegate(user, 1_000, unbonding_period).unwrap();

    // Fund both distribution flows with same amount.
    suite
        .execute_fund_distribution(user, None, juno(400))
        .unwrap();
    suite
        .execute_fund_distribution_with_cw20(
            user,
            AssetInfoValidated::Token(wynd_token).with_balance(400u128),
        )
        .unwrap();

    suite.update_time(100);

    // Set unbond all flag to true.
    suite.execute_unbond_all(UNBONDER).unwrap();

    // Cannot distribute funds when unbod all.
    let err = suite.distribute_funds(user, None, None).unwrap_err();

    assert_eq!(
        ContractError::CannotDistributeIfUnbondAll {
            what: "rewards".into()
        },
        err.downcast().unwrap()
    );
}
