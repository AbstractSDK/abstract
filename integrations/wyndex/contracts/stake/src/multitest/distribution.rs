use cosmwasm_std::{assert_approx_eq, Addr, Decimal, Uint128};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw_multi_test::Executor;
use wyndex::asset::{AssetInfo, AssetInfoExt, AssetInfoValidated};
use wyndex::stake::FundingInfo;

use super::suite::{contract_token, SuiteBuilder};
use crate::{
    multitest::suite::{juno, juno_power, native_token, JUNO_DENOM},
    ContractError,
};

#[test]
fn multiple_distribution_flows() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1200)])
        .with_native_balances("luna", vec![(&members[3], 1200)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();
    // Setup a second distribution flow
    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("luna".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    // create wynd token
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
                    address: "member4".to_owned(),
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

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // Fund both distribution flows
    suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();
    suite
        .execute_fund_distribution(&members[3], None, native_token("luna".to_string(), 400))
        .unwrap();

    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "luna")
            .unwrap(),
        400,
    );
    // Reward epoch is 100, so advance 50% of that
    suite.update_time(50);

    // Distribute the funds
    suite.distribute_funds(&members[3], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);

    // Assert that we have 2 rewards tokens and their amounts
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(25), native_token("luna".to_string(), 25)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(50), native_token("luna".to_string(), 50)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(125), native_token("luna".to_string(), 125)]
    );

    // add wynd distribution
    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Token(wynd_token.to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    // Finally, setup the Wynd distribution before advancing time again to collect rewards
    suite
        .execute_fund_distribution_with_cw20(
            &members[3],
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(400u128),
        )
        .unwrap();

    // Advance the final 50% for the first two native tokens and 50% for the Wynd token
    suite.update_time(50);

    // Distribute the funds
    suite.distribute_funds(&members[3], None, None).unwrap();

    // Assert we have gathered all the rewards from the two native tokens and 50% of the rewards from the Wynd token
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![
            juno(50),
            native_token("luna".to_string(), 50),
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(25u128)
        ]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![
            juno(100),
            native_token("luna".to_string(), 100),
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(50u128)
        ]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![
            juno(250),
            native_token("luna".to_string(), 250),
            AssetInfoValidated::Token(wynd_token).with_balance(125u128)
        ]
    );
}

// copy of multiple_distribution_flows but using the mass_bond approach to ensure
// it is consistent with the users staking individually
#[test]
fn mass_bond_with_multiple_distribution_flows() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    // this guy hodls the funds to mass bond to others
    let richie = "richie rich";
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            // all future bonds held by richie rich
            (richie, delegated),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1200)])
        .with_native_balances("luna", vec![(&members[3], 1200)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();
    // Setup a second distribution flow
    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("luna".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    // create wynd token
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
                    address: "member4".to_owned(),
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

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    // this is the only part we change from the above.. using mass_bond not delegate
    let delegations: &[(&str, u128)] = &[
        (&members[0], bonds[0]),
        (&members[1], bonds[1]),
        (&members[2], bonds[2]),
    ];
    suite
        .mass_delegate(richie, delegated, unbonding_period, delegations)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // Fund both distribution flows
    suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();
    suite
        .execute_fund_distribution(&members[3], None, native_token("luna".to_string(), 400))
        .unwrap();

    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "luna")
            .unwrap(),
        400,
    );
    // Reward epoch is 100, so advance 50% of that
    suite.update_time(50);

    // Distribute the funds
    suite.distribute_funds(&members[3], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);

    // Assert that we have 2 rewards tokens and their amounts
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(25), native_token("luna".to_string(), 25)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(50), native_token("luna".to_string(), 50)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(125), native_token("luna".to_string(), 125)]
    );

    // add wynd distribution
    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Token(wynd_token.to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    // Finally, setup the Wynd distribution before advancing time again to collect rewards
    suite
        .execute_fund_distribution_with_cw20(
            &members[3],
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(400u128),
        )
        .unwrap();

    // Advance the final 50% for the first two native tokens and 50% for the Wynd token
    suite.update_time(50);

    // Distribute the funds
    suite.distribute_funds(&members[3], None, None).unwrap();

    // Assert we have gathered all the rewards from the two native tokens and 50% of the rewards from the Wynd token
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![
            juno(50),
            native_token("luna".to_string(), 50),
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(25u128)
        ]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![
            juno(100),
            native_token("luna".to_string(), 100),
            AssetInfoValidated::Token(wynd_token.clone()).with_balance(50u128)
        ]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![
            juno(250),
            native_token("luna".to_string(), 250),
            AssetInfoValidated::Token(wynd_token).with_balance(125u128)
        ]
    );
}

#[test]
fn can_fund_an_inprogress_reward_period_with_more_funds_and_a_curve() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1200)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);

    let _resp = suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();

    // assert that staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );
    // Reward epoch is 100, so advance 50% of that
    suite.update_time(50);

    // Distribute the funds
    let _resp = suite.distribute_funds(&members[3], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);

    // We have 50% of the initial funds distributed, so we should have 50% of the rewards there
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(25)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(50)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(125)]
    );

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(200)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(200)]);

    // Do some withdrawals
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // Verify the amounts
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 25);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 125);

    // By now we have done 1 funding and 1 payout, perform another funding and progress
    suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();

    // Advanced time 50, this will unlock the final 50% of the first funding and 50% of the second
    suite.update_time(50);

    suite.distribute_funds(&members[3], None, None).unwrap();

    // 400 distributed from first funding (100%), 200 from the second as we are 50% of the way on that
    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(600)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(200)]);

    // Do some withdrawals
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // Verify the amounts
    // We should have the full amount of the first funding and half of the amount of the second by now
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 75);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 150);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 375);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 400);

    // Fund one more time with the same curves
    let _resp = suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();

    // assert that staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        600,
    );
}

#[test]
fn partial_payouts_by_rate() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 400)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);

    let _resp = suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();

    // assert that staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );
    // Reward epoch is 100, so advance 20% of that
    suite.update_time(20);

    // TODO: Would be better if we didn't need to pass in 1 token here, involves removing an error check in that function
    let _resp = suite.distribute_funds(&members[3], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);

    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(10)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(20)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(50)]
    );

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(80)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(320)]);

    // Do some withdrawals
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // Verify the amounts
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 10);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 20);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);

    // Reward epoch is 100, we already did 20, do another 20 so advance 40% of total
    suite.update_time(20);

    let _resp = suite.distribute_funds(&members[3], None, None).unwrap();
    // verify withdrawable rewards is
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(10)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(20)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(50)]
    );

    // Instead of withdrawing, lets advance and distribute one more time then withdraw
    // Reward epoch is 100, we have done 40 by now + 20 = 60% of total
    suite.update_time(20);

    let _resp = suite.distribute_funds(&members[3], None, None).unwrap();

    // verify withdrawable rewards is
    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(20)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(40)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(100)]
    );

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(240)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(160)]);

    // Do some withdrawals
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // Verify the amounts
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 30);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 60);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 150);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);

    // And now the final piece
    suite.update_time(40);

    let _resp = suite.distribute_funds(&members[3], None, None).unwrap();

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(400)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    // Do some withdrawals
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // Verify the amounts
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn divisible_amount_distributed_with_rate() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 401)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);

    let _resp = suite
        .execute_fund_distribution(&members[3], None, juno(400))
        .unwrap();

    // resp.assert_event(&distribution_event(&members[3], &denom, 400));

    // assert that staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );
    suite.update_time(100);

    let _resp = suite
        .distribute_funds(&members[3], None, Some(juno(1)))
        .unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);

    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(50)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(100)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(250)]
    );

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(401)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // assert_eq!(
    //     suite
    //         .query_balance_vesting_contract(suite.stake_contract().as_str())
    //         .unwrap(),
    //     0
    // );
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn calculate_apr() {
    let distributor = "distributor";
    let member1 = "member1";
    let member2 = "member2";
    let unbonding_periods = vec![100u64, 1000u64, 10_000u64];

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(unbonding_periods.clone())
        .with_admin("admin")
        .with_initial_balances(vec![(member1, 500_000_000), (member2, 500_000_000)])
        .with_native_balances("juno", vec![(distributor, 1_000_000_000)])
        .build();

    // create distribution flow
    suite
        .create_distribution_flow(
            "admin",
            distributor,
            AssetInfo::Native("juno".to_string()),
            vec![
                (unbonding_periods[0], Decimal::percent(50)),
                (unbonding_periods[1], Decimal::one()),
                (unbonding_periods[2], Decimal::percent(300)),
            ],
        )
        .unwrap();

    // Noting is staked, so we can't provide APR
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(annual_rewards[0].1[0].amount, None);

    // delegate to different unbonding periods (100 JUNO each, 2x per member)
    suite
        .delegate(member1, 100_000_000, unbonding_periods[0])
        .unwrap();
    suite
        .delegate(member1, 100_000_000, unbonding_periods[1])
        .unwrap();
    suite
        .delegate(member2, 100_000_000, unbonding_periods[1])
        .unwrap();
    suite
        .delegate(member2, 100_000_000, unbonding_periods[2])
        .unwrap();
    // rewards power breakdown:
    // 100_000_000 * 0.5 / 1000 = 50_000
    // 100_000_000 * 1 / 1000 = 100_000
    // 100_000_000 * 3 / 1000 = 300_000
    assert_eq!(
        suite.query_rewards_power(member1).unwrap()[0].1,
        150_000,
        "50_000 + 100_000 = 150_000"
    );
    assert_eq!(
        suite.query_rewards_power(member2).unwrap()[0].1,
        400_000,
        "100_000 + 300_000 = 400_000"
    );
    // apr should be 0 at the moment, because the distribution is not funded yet
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(annual_rewards[0].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[1].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[2].1[0].amount, Some(Decimal::zero()));

    // Fund distribution flow - 55 JUNO for 1 week (6 decimals)
    suite
        .execute_fund_distribution_curve(distributor, JUNO_DENOM, 55_000_000, 86400 * 7)
        .unwrap();

    // There are 55 JUNO over 1 week. We have 400 JUNO locked.
    // So something like 600% APR for middle tier would be a good reality check

    // There are a total of 550_000 reward points. Meaning each reward point receives 100 ujuno per week
    // or 5_214 ujuno per year.
    // 1 JUNO at lowest category gives 500 reward points, so 500 * 5_214 / 1_000_000 = 2.607 = 260.7%
    // 1 JUNO at lowest category gives 1000 reward points, so 1000 * 5_214 / 1_000_000 = 52.14 = 521.4%
    // 1 JUNO at lowest category gives 3000 reward points, so 3000 * 5_214 / 1_000_000 = 15.642 = 1564,2%

    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(
        // multiply by 1000 to get an int of promille. eg 123.4 % = 1.234 * 1000 = 1234
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(2607),
    );
    assert_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(5214),
    );
    assert_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(15642),
    );

    // 4 days later, the rewards are still active, APRs should remain the same
    suite.update_time(4 * 86_400);
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(
        // multiply by 1000 to get an int of promille. eg 123.4 % = 1.234 * 1000 = 1234
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(2607),
    );
    assert_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(5214),
    );
    assert_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(15642),
    );

    // Another 4 days later, 8 days have passed, rewards were for 7 days.
    // APRs should be at 0 again
    suite.update_time(4 * 86_400);
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(annual_rewards[0].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[1].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[2].1[0].amount, Some(Decimal::zero()));
}

#[test]
fn apr_cw20() {
    let distributor = "distributor";
    let member1 = "member1";
    let member2 = "member2";
    let unbonding_periods = vec![100u64, 1000u64, 10_000u64];

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(unbonding_periods.clone())
        .with_admin("admin")
        .with_initial_balances(vec![(member1, 500_000_000), (member2, 500_000_000)])
        .build();

    // create a cw20 token
    let cw20_contract = suite.instantiate_token(
        &Addr::unchecked("owner"),
        "TEST",
        Some(12),
        &[(distributor, 1_000_000_000_000_000_000)],
    );
    let cw20_info = AssetInfoValidated::Token(cw20_contract);

    // create distribution flow
    suite
        .create_distribution_flow(
            "admin",
            distributor,
            cw20_info.clone().into(),
            vec![
                (unbonding_periods[0], Decimal::percent(70)),
                (unbonding_periods[1], Decimal::one()),
                (unbonding_periods[2], Decimal::percent(200)),
            ],
        )
        .unwrap();

    // delegate to different unbonding periods (100 JUNO each, 2x per member)
    suite
        .delegate(member1, 100_000_000, unbonding_periods[0])
        .unwrap();
    suite
        .delegate(member1, 100_000_000, unbonding_periods[1])
        .unwrap();
    suite
        .delegate(member2, 100_000_000, unbonding_periods[1])
        .unwrap();
    suite
        .delegate(member2, 100_000_000, unbonding_periods[2])
        .unwrap();
    // rewards power breakdown:
    // 100_000_000 * 0.7 / 1000 = 70_000
    // 100_000_000 * 1 / 1000 = 100_000
    // 100_000_000 * 2 / 1000 = 200_000
    assert_eq!(
        suite.query_rewards_power(member1).unwrap()[0].1,
        170_000,
        "70_000 + 100_000 = 170_000"
    );
    assert_eq!(
        suite.query_rewards_power(member2).unwrap()[0].1,
        300_000,
        "100_000 + 200_000 = 300_000"
    );
    // apr should be 0 at the moment, because the distribution is not funded yet
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(annual_rewards[0].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[1].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[2].1[0].amount, Some(Decimal::zero()));

    // fund the distribution flow - 1_000_000 JUNO for a year
    const YEAR: u64 = 365 * 24 * 60 * 60;

    let curr_block = suite.app.block_info().time;
    suite
        .execute_fund_distribution_with_cw20_curve(
            distributor,
            cw20_info.with_balance(1_000_000_000_000_000u128),
            FundingInfo {
                start_time: curr_block.seconds(),
                distribution_duration: YEAR,
                amount: Uint128::from(1_000_000_000_000_000u128),
            },
        )
        .unwrap();

    // distributing 1_000_000_000_000_000 uJUNO,
    // total rewards power is 470_000
    // rewards power per period:
    // 70_000, 200_000, 200_000
    // so total rewards by period (period power / total power * total rewards):
    // 70_000 / 470_000 * 1_000_000_000_000_000 = 148936170212765
    // 200_000 / 470_000 * 1_000_000_000_000_000 = 425531914893617
    // 200_000 / 470_000 * 1_000_000_000_000_000 = 425531914893617
    // now divide by staked amount to get annualized rewards per token:
    // 148936170212765 / 100_000_000 = 1489361.70212765
    // 425531914893617 / 200_000_000 = 2127659.574468085
    // 425531914893617 / 100_000_000 = 4255319.14893617
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(
        // multiply by 1000 to get an int of promille. eg 123.4 % = 1.234 * 1000 = 1234
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(1489361702),
    );
    assert_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(2127659574),
    );
    assert_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(4255319148),
    );

    // forward almost a year
    suite.update_time(YEAR - 1);

    // APR should be the same as before (modulo some rounding difference), calculated by extrapolating the curve
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_approx_eq!(
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(1489361702),
        "0.000000001"
    );
    assert_approx_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(2127659574),
        "0.000000001"
    );
    assert_approx_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1000),
        Uint128::new(4255319148),
        "0.000000001"
    );

    // forward the last second
    suite.update_time(1);

    // APR should be 0 again, because the distribution is depleted
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(annual_rewards[0].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[1].1[0].amount, Some(Decimal::zero()));
    assert_eq!(annual_rewards[2].1[0].amount, Some(Decimal::zero()));

    // Following code should be removed if the code is not generalized for piecewise linear
    /*
    // fund with a complex piecewise linear curve
    suite
        .execute_fund_distribution_with_cw20_curve(
            distributor,
            cw20_info.with_balance(1_000_000_000u128),
            Curve::PiecewiseLinear(PiecewiseLinear {
                steps: vec![
                    (0, 1_000_000_000u128.into()),
                    (YEAR * 2, 500_000_000u128.into()),
                    (YEAR * 3, 100_000_000u128.into()),
                    (YEAR * 4, 0u128.into()),
                ],
            }),
        )
        .unwrap();

    // change stakes
    suite
        .unbond(member1, 99_999_000, unbonding_periods[0])
        .unwrap();
    suite
        .unbond(member2, 100_000_000, unbonding_periods[1])
        .unwrap();
    // stakes:
    // member1: 1_000 < min_bond in period 0, 100_000_000 in period 1
    // member2: 100_000_000 in period 2
    // rewards power:
    assert_eq!(
        suite.query_rewards_power(member1).unwrap()[0].1,
        100_000,
        "100_000_000 * 1 / 1000"
    );
    assert_eq!(
        suite.query_rewards_power(member2).unwrap()[0].1,
        200_000,
        "100_000_000 * 2 / 1000"
    );
    // total power: 300_000

    // distributing 250_000_000 uJUNO in the first year,
    // so total rewards by period (period power / total power * total rewards):
    // period 0 has no power, but we can calculate differently without the period's power:
    // rewards multiplier / total power * total rewards / tokens per power
    // = 0.7 / 300_000 * 250_000_000 / 1000 = 0.583333333333
    // period 1: 100_000 / 300_000 * 250_000_000 / 100_000_000 = 0.83333333
    // period 2: 200_000 / 300_000 * 250_000_000 = 1.66666666
    let annual_rewards = suite.query_annualized_rewards().unwrap();

    assert_eq!(
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(583333),
    );
    assert_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(833333),
    );
    assert_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(1666666),
    );

    // forward 2.5 years
    suite.update_time(YEAR * 2 + YEAR / 2);

    // distributing 200_000_000 uJUNO in the first half and 50_000_000 uJUNO in the second half,
    // so 250_000_000 uJUNO in the full year
    // annual rewards should be the same as before
    let annual_rewards = suite.query_annualized_rewards().unwrap();
    assert_eq!(
        annual_rewards[0].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(583333),
    );
    assert_eq!(
        annual_rewards[1].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(833333),
    );
    assert_eq!(
        annual_rewards[2].1[0].amount.unwrap() * Uint128::new(1_000_000),
        Uint128::new(1666666),
    );
    */
}

#[test]
fn simple_apr_simulation() {
    let distributor = "distributor";
    let members = vec!["member1", "member2"];
    let unbonding_periods = vec![1, 2, 3];
    let stakes = [100_000_000u128, 200_000_000u128];
    let rewards = 250_000_000u128;

    let mut suite = SuiteBuilder::new()
        .with_admin("admin")
        .with_unbonding_periods(unbonding_periods.clone())
        .with_initial_balances(vec![(members[0], stakes[0]), (members[1], stakes[1])])
        .with_native_balances("juno", vec![(distributor, rewards)])
        .build();

    // create distribution flow
    suite
        .create_distribution_flow(
            "admin",
            distributor,
            AssetInfo::Native("juno".to_string()),
            vec![
                (unbonding_periods[0], Decimal::percent(70)),
                (unbonding_periods[1], Decimal::one()),
                (unbonding_periods[2], Decimal::percent(200)),
            ],
        )
        .unwrap();

    const YEAR: u64 = 365 * 24 * 60 * 60;

    suite
        .execute_fund_distribution_curve(distributor, "juno", rewards, 2 * YEAR)
        .unwrap();

    suite
        .delegate(members[0], stakes[0], unbonding_periods[0])
        .unwrap();
    suite
        .delegate(members[1], stakes[1], unbonding_periods[1])
        .unwrap();

    // get promised rewards per token
    let expected_reward_per_token = suite
        .query_annualized_rewards()
        .unwrap()
        .into_iter()
        .map(|(_, rewards)| rewards[0].amount.unwrap())
        .collect::<Vec<_>>();

    // forward 1 year
    suite.update_time(YEAR);

    // distribute to update withdrawable rewards
    suite.distribute_funds(members[0], None, None).unwrap();

    // check actual rewards
    let actual_reward = suite
        .withdrawable_rewards(members[0])
        .unwrap()
        .swap_remove(0);
    assert_eq!(
        actual_reward.amount,
        expected_reward_per_token[0] * Uint128::new(stakes[0]),
    );
    let actual_reward = suite
        .withdrawable_rewards(members[1])
        .unwrap()
        .swap_remove(0);
    assert_eq!(
        actual_reward.amount,
        expected_reward_per_token[1] * Uint128::new(stakes[1]),
    );
}

#[test]
fn divisible_amount_distributed() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];
    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 400u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 400)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), 0);

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);

    let _resp = suite
        .distribute_funds(&members[3], None, Some(juno(400)))
        .unwrap();

    // resp.assert_event(&distribution_event(&members[3], &denom, 400));

    // assert that staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);
    // assert that rewards are there
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        400,
    );

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);

    assert_eq!(
        suite.withdrawable_rewards(&members[0]).unwrap(),
        vec![juno(50)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[1]).unwrap(),
        vec![juno(100)]
    );
    assert_eq!(
        suite.withdrawable_rewards(&members[2]).unwrap(),
        vec![juno(250)]
    );

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(400)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    // assert_eq!(
    //     suite
    //         .query_balance_vesting_contract(suite.stake_contract().as_str())
    //         .unwrap(),
    //     0
    // );
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn divisible_amount_distributed_twice() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let delegated: u128 = bonds.iter().sum();
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 1000u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1000)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::percent(200))],
        )
        .unwrap();

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    assert_eq!(suite.query_balance_staking_contract().unwrap(), delegated);

    suite
        .distribute_funds(&members[3], None, Some(juno(400)))
        .unwrap();

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(400)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 600);

    suite
        .distribute_funds(&members[3], None, Some(juno(600)))
        .unwrap();

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(1000)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 125);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 625);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn divisible_amount_distributed_twice_accumulated() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let bonds = vec![5_000u128, 10_000u128, 25_000u128];
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1000u128)])
        .with_initial_balances(vec![
            (&members[0], bonds[0]),
            (&members[1], bonds[1]),
            (&members[2], bonds[2]),
            (&members[3], 1000u128),
        ])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite
        .delegate(&members[0], bonds[0], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], bonds[1], unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], bonds[2], unbonding_period)
        .unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(400)))
        .unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(600)))
        .unwrap();

    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(1000)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(
        suite
            .query_balance_vesting_contract(suite.token_contract().as_str())
            .unwrap(),
        0
    );
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 125);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 250);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 625);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn points_changed_after_distribution() {
    let members = vec![
        "member0".to_owned(),
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_min_bond(1000)
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1500)])
        .with_initial_balances(vec![
            (&members[0], 6_000u128),
            (&members[1], 2_000u128),
            (&members[2], 5_000u128),
            (&members[3], 1500u128),
        ])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::percent(200))],
        )
        .unwrap();

    suite.delegate(&members[0], 1000, unbonding_period).unwrap();
    suite.delegate(&members[1], 2000, unbonding_period).unwrap();
    suite.delegate(&members[2], 5000, unbonding_period).unwrap();

    assert_eq!(
        suite.query_rewards_power(&members[0]).unwrap(),
        juno_power(2u128)
    );
    assert_eq!(
        suite.query_rewards_power(&members[1]).unwrap(),
        juno_power(4u128)
    );
    assert_eq!(
        suite.query_rewards_power(&members[2]).unwrap(),
        juno_power(10u128)
    );
    assert_eq!(suite.query_total_rewards_power().unwrap(), juno_power(16));

    suite
        .distribute_funds(&members[3], None, Some(juno(400)))
        .unwrap();
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0u128)]);
    assert_eq!(suite.withdrawable_funds().unwrap(), vec![juno(400)]);
    // TODO: add distributed / withdrawable tests

    // Modifying power to:
    // member[0] => 6
    // member[1] => 0 (removed)
    // member[2] => 5
    suite.delegate(&members[0], 5000, unbonding_period).unwrap();
    suite.unbond(&members[1], 2000, unbonding_period).unwrap();
    // BUG: unbonding tokens are considered rewards to be paid out
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0u128)]);
    assert_eq!(suite.withdrawable_funds().unwrap(), vec![juno(400)]);

    assert_eq!(
        suite.query_rewards_power(&members[0]).unwrap(),
        juno_power(12u128)
    );
    assert_eq!(suite.query_rewards_power(&members[1]).unwrap(), vec![]);
    assert_eq!(
        suite.query_rewards_power(&members[2]).unwrap(),
        juno_power(10u128)
    );
    assert_eq!(suite.query_total_rewards_power().unwrap(), juno_power(22));

    // Ensure funds are withdrawn properly, considering old points
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();
    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(400u128)]);
    assert_eq!(suite.undistributed_funds().unwrap(), vec![juno(0)]);
    assert_eq!(suite.withdrawable_funds().unwrap(), vec![juno(0)]);

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 50);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 250);

    // Distribute tokens again to ensure distribution considers new points
    // 600 -> member0 and 500 -> member2
    suite
        .distribute_funds(&members[3], None, Some(juno(1100)))
        .unwrap();
    assert_eq!(suite.distributed_funds().unwrap(), vec![juno(1500u128)]);
    assert_eq!(suite.withdrawable_funds().unwrap(), vec![juno(1100)]);

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();
    assert_eq!(suite.withdrawable_funds().unwrap(), vec![juno(0)]);

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 650);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 750);
}

#[test]
fn points_changed_after_distribution_accumulated() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_min_bond(1000)
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 1500)])
        .with_initial_balances(vec![
            (&members[0], 6_000u128),
            (&members[1], 2_000u128),
            (&members[2], 5_000u128),
            (&members[3], 1500u128),
        ])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::percent(200))],
        )
        .unwrap();

    suite.delegate(&members[0], 1000, unbonding_period).unwrap();
    suite.delegate(&members[1], 2000, unbonding_period).unwrap();
    suite.delegate(&members[2], 5000, unbonding_period).unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(400)))
        .unwrap();
    // Modifying wights to:
    // member[0] => 6
    // member[1] => 0 (removed)
    // member[2] => 5
    // total_points => 11
    suite.delegate(&members[0], 5000, unbonding_period).unwrap();
    suite.unbond(&members[1], 2000, unbonding_period).unwrap();

    // Distribute tokens again to ensure distribution considers new points
    suite
        .distribute_funds(&members[3], None, Some(juno(1100)))
        .unwrap();

    // Withdraws sums of both distributions, so it works when they were using different points
    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 650);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 750);
    assert_eq!(suite.query_balance(&members[3], "juno").unwrap(), 0);
}

#[test]
fn distribution_with_leftover() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        // points are set to be prime numbers, difficult to distribute over. All are mutually prime
        // with distributed amount
        .with_initial_balances(vec![
            (&members[0], 7_000u128),
            (&members[1], 11_000u128),
            (&members[2], 13_000u128),
            (&members[3], 3100u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 3100)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::percent(200))],
        )
        .unwrap();

    suite
        .delegate(&members[0], 7_000, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 11_000, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], 13_000, unbonding_period)
        .unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(100)))
        .unwrap();

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 22);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 35);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 41);

    // Second distribution adding to the first one would actually make it properly divisible,
    // all shares should be properly split
    suite
        .distribute_funds(&members[3], None, Some(juno(3000)))
        .unwrap();

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 700);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 1100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 1300);
}

#[test]
fn distribution_with_leftover_accumulated() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        // points are set to be prime numbers, difficult to distribute over. All are mutually prime
        // with distributed amount
        .with_initial_balances(vec![
            (&members[0], 7_000u128),
            (&members[1], 11_000u128),
            (&members[2], 13_000u128),
            (&members[3], 3100u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 3100)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::percent(200))],
        )
        .unwrap();

    suite
        .delegate(&members[0], 7_000, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 11_000, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[2], 13_000, unbonding_period)
        .unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(100)))
        .unwrap();

    // Second distribution adding to the first one would actually make it properly divisible,
    // all shares should be properly split
    suite
        .distribute_funds(&members[3], None, Some(juno(3000)))
        .unwrap();

    suite.withdraw_funds(&members[0], None, None).unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();
    suite.withdraw_funds(&members[2], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 700);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 1100);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 1300);
}

#[test]
fn redirecting_withdrawn_funds() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
        "member4".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_min_bond(1000)
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[3], 100)])
        // points are set to be prime numbers, difficult to distribute over. All are mutually prime
        // with distributed amount
        .with_initial_balances(vec![
            (&members[0], 4_000u128),
            (&members[1], 6_000u128),
            (&members[3], 100u128),
        ])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite
        .delegate(&members[0], 4_000, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 6_000, unbonding_period)
        .unwrap();

    suite
        .distribute_funds(&members[3], None, Some(juno(100)))
        .unwrap();

    suite
        .withdraw_funds(&members[0], None, members[2].as_str())
        .unwrap();
    suite.withdraw_funds(&members[1], None, None).unwrap();

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 60);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 40);
}

#[test]
fn cannot_withdraw_others_funds() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
    ];
    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_min_bond(1000)
        .with_initial_balances(vec![
            (&members[0], 4_000u128),
            (&members[1], 6_000u128),
            (&members[2], 100u128),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[2], 100)])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite
        .delegate(&members[0], 4_000u128, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 6_000u128, unbonding_period)
        .unwrap();

    suite
        .distribute_funds(&members[2], None, Some(juno(100)))
        .unwrap();
    // assert staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 10000);
    // assert rewards arrived
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        100
    );

    let err = suite
        .withdraw_funds(&members[0], members[1].as_str(), None)
        .unwrap_err();

    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());

    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    // assert staking token balance is still the same
    assert_eq!(suite.query_balance_staking_contract().unwrap(), 10000);
    // assert rewards arrived
    assert_eq!(
        suite
            .query_balance(suite.stake_contract().as_str(), "juno")
            .unwrap(),
        40
    );
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 60);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);
}

#[test]
fn funds_withdrawal_delegation() {
    let members = vec![
        "member1".to_owned(),
        "member2".to_owned(),
        "member3".to_owned(),
    ];

    let unbonding_period = 1000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period])
        .with_min_bond(1000)
        .with_admin("admin")
        .with_native_balances("juno", vec![(&members[2], 100)])
        .with_initial_balances(vec![
            (&members[0], 4_000u128),
            (&members[1], 6_000u128),
            (&members[2], 100u128),
        ])
        .build();

    suite
        .create_distribution_flow(
            "admin",
            &members[0],
            AssetInfo::Native("juno".to_string()),
            vec![(unbonding_period, Decimal::one())],
        )
        .unwrap();

    suite
        .delegate(&members[0], 4_000u128, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 6_000u128, unbonding_period)
        .unwrap();

    assert_eq!(
        suite.delegated(&members[0]).unwrap().as_str(),
        members[0].as_str()
    );
    assert_eq!(
        suite.delegated(&members[1]).unwrap().as_str(),
        members[1].as_str()
    );

    suite
        .distribute_funds(&members[2], None, Some(juno(100)))
        .unwrap();

    suite.delegate_withdrawal(&members[1], &members[0]).unwrap();

    suite
        .withdraw_funds(&members[0], members[1].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.delegated(&members[0]).unwrap().as_str(),
        members[0].as_str()
    );
    assert_eq!(
        suite.delegated(&members[1]).unwrap().as_str(),
        members[0].as_str()
    );

    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 100);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 0);
    assert_eq!(suite.query_balance(&members[2], "juno").unwrap(), 0);
}

#[test]
fn querying_unknown_address() {
    let suite = SuiteBuilder::new().build();

    let resp = suite.withdrawable_rewards("unknown").unwrap();
    assert_eq!(resp, vec![]);
}

#[test]
fn rebond_works() {
    let members = vec!["member0".to_owned(), "member1".to_owned()];
    let executor = "executor";

    let unbonding_period = 1000u64;
    let unbonding_period2 = 2000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period, unbonding_period2])
        .with_min_bond(1000)
        .with_initial_balances(vec![
            (&members[0], 1_000u128),
            (&members[1], 2_000u128),
            (executor, 450 + 300),
        ])
        .with_admin("admin")
        .with_native_balances("juno", vec![(executor, 1_000u128)]) // give some juno to reward people with
        .build();

    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![
                (unbonding_period, Decimal::one()),
                (unbonding_period2, Decimal::percent(200)),
            ],
        )
        .unwrap();

    // delegate
    suite
        .delegate(&members[0], 1_000u128, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 2_000u128, unbonding_period)
        .unwrap();

    // rebond member1 up to unbonding_period2
    suite
        .rebond(&members[1], 2_000u128, unbonding_period, unbonding_period2)
        .unwrap();
    // rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 2 / 1000 = 4
    // total: 5

    // distribute
    suite
        .distribute_funds(executor, None, Some(juno(450)))
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "juno").unwrap(),
        90,
        "member0 should have received 450 * 1 / 5 = 90"
    );
    assert_eq!(
        suite.query_balance(&members[1], "juno").unwrap(),
        360,
        "member1 should have received 450 * 4 / 5 = 360"
    );

    // rebond member1 down again to unbonding_period
    suite
        .rebond(&members[1], 2_000u128, unbonding_period2, unbonding_period)
        .unwrap();
    // rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 1 / 1000 = 2
    // total: 3

    // distribute
    suite
        .distribute_funds(executor, None, Some(juno(300)))
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "juno").unwrap(),
        90 + 100,
        "member0 should have received 300 * 1 / 3 = 100"
    );
    assert_eq!(
        suite.query_balance(&members[1], "juno").unwrap(),
        360 + 200,
        "member1 should have received 300 * 2 / 3 = 200"
    );
}

#[test]
fn rebond_multiple_works() {
    // This is just a version of `rebond_works` with multiple distributions in order to have
    // a more complex case with withdrawals and rebonding in-between distributing funds.
    let members = vec!["member0".to_owned(), "member1".to_owned()];
    let executor = "executor";

    let unbonding_period = 1000u64;
    let unbonding_period2 = 2000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period, unbonding_period2])
        .with_min_bond(1000)
        .with_initial_balances(vec![(&members[0], 1_000u128), (&members[1], 2_000u128)])
        .with_admin("admin")
        .with_native_balances("juno", vec![(executor, 1_000u128)]) // give some juno to reward people with
        .with_native_balances("osmo", vec![(executor, 600u128)]) // give some osmo to reward people with
        .build();

    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![
                (unbonding_period, Decimal::one()),
                (unbonding_period2, Decimal::percent(200)),
            ],
        )
        .unwrap();

    // create second distribution flow
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("osmo".to_string()),
            vec![
                (unbonding_period, Decimal::one()),
                (unbonding_period2, Decimal::one()),
            ],
        )
        .unwrap();

    // delegate
    suite
        .delegate(&members[0], 1_000u128, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 2_000u128, unbonding_period)
        .unwrap();

    // rebond member1 up to unbonding_period2
    suite
        .rebond(&members[1], 2_000u128, unbonding_period, unbonding_period2)
        .unwrap();
    // juno rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 2 / 1000 = 4
    // total: 5

    // distribute
    suite
        .distribute_funds(executor, None, Some(juno(450)))
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "juno").unwrap(),
        90,
        "member0 should have received 450 * 1 / 5 = 90"
    );
    assert_eq!(
        suite.query_balance(&members[1], "juno").unwrap(),
        360,
        "member1 should have received 450 * 4 / 5 = 360"
    );

    // osmo rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 1 / 1000 = 2
    // total: 3

    // distribute
    suite
        .distribute_funds(
            executor,
            None,
            Some(AssetInfoValidated::Native("osmo".to_string()).with_balance(300u128)),
        )
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "osmo").unwrap(),
        100,
        "member0 should have received 300 * 1 / 3 = 100"
    );
    assert_eq!(
        suite.query_balance(&members[1], "osmo").unwrap(),
        200,
        "member1 should have received 300 * 2 / 3 = 200"
    );

    // rebond member1 down again to unbonding_period
    suite
        .rebond(&members[1], 2_000u128, unbonding_period2, unbonding_period)
        .unwrap();

    // juno rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 1 / 1000 = 2
    // total: 3

    // distribute
    suite
        .distribute_funds(executor, None, Some(juno(300)))
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "juno").unwrap(),
        90 + 100,
        "member0 should have received 300 * 1 / 3 = 100"
    );
    assert_eq!(
        suite.query_balance(&members[1], "juno").unwrap(),
        360 + 200,
        "member1 should have received 300 * 2 / 3 = 200"
    );

    // osmo rewards power breakdown:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 1 / 1000 = 2
    // total: 3

    // distribute
    suite
        .distribute_funds(
            executor,
            None,
            Some(AssetInfoValidated::Native("osmo".to_string()).with_balance(300u128)),
        )
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    assert_eq!(
        suite.query_balance(&members[0], "osmo").unwrap(),
        100 + 100,
        "member0 should have received 300 * 1 / 3 = 100"
    );
    assert_eq!(
        suite.query_balance(&members[1], "osmo").unwrap(),
        200 + 200,
        "member1 should have received 300 * 2 / 3 = 200"
    );
}

#[test]
fn multiple_rewards() {
    // This test checks that handling of multiple different distributions works correctly.
    // One of them is a native token, one a cw20 token.
    // We add distributions for both, then delegate and distribute, then check that it was done correctly.

    let members = vec!["member0".to_owned(), "member1".to_owned()];
    let executor = "executor";

    let unbonding_period = 1000u64;
    let unbonding_period2 = 2000u64;

    let mut suite = SuiteBuilder::new()
        .with_unbonding_periods(vec![unbonding_period, unbonding_period2])
        .with_min_bond(1000)
        .with_initial_balances(vec![(&members[0], 1_000), (&members[1], 2_000)])
        .with_admin("admin")
        .with_native_balances("juno", vec![(executor, 1_000)])
        .build();

    // add juno distribution
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![
                (unbonding_period, Decimal::one()),
                (unbonding_period2, Decimal::percent(200)),
            ],
        )
        .unwrap();

    // create wynd token
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
                    // executor gets some to distribute
                    address: executor.to_string(),
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

    // add wynd distribution
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Token(wynd_token.to_string()),
            vec![
                (unbonding_period, Decimal::one()),
                (unbonding_period2, Decimal::one()),
            ],
        )
        .unwrap();

    // delegate
    suite
        .delegate(&members[0], 1_000u128, unbonding_period)
        .unwrap();
    suite
        .delegate(&members[1], 2_000u128, unbonding_period2)
        .unwrap();

    // distribute juno
    suite
        .distribute_funds(executor, executor, Some(juno(1_000)))
        .unwrap();
    // distribute wynd
    suite
        .distribute_funds(
            executor,
            executor,
            Some(AssetInfoValidated::Token(wynd_token.clone()).with_balance(500u128)),
        )
        .unwrap();

    // withdraw
    suite
        .withdraw_funds(&members[0], members[0].as_str(), None)
        .unwrap();
    suite
        .withdraw_funds(&members[1], members[1].as_str(), None)
        .unwrap();

    // rewards power for juno:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 2 / 1000 = 4
    // => 1000 * 1 / 5 = 200, 1000 * 4 / 5 = 800
    assert_eq!(suite.query_balance(&members[0], "juno").unwrap(), 200);
    assert_eq!(suite.query_balance(&members[1], "juno").unwrap(), 800);

    // rewards power for wynd:
    // member0: 1000 * 1 / 1000 = 1
    // member1: 2000 * 1 / 1000 = 2
    // => 500 * 1 / 3 = 166, 500 * 2 / 3 = 333
    assert_eq!(
        suite
            .query_cw20_balance(&members[0], wynd_token.clone())
            .unwrap(),
        166
    );
    assert_eq!(
        suite.query_cw20_balance(&members[1], wynd_token).unwrap(),
        333
    );
}

#[test]
fn distribute_staking_token_should_fail() {
    let executor = "executor";
    let mut suite = SuiteBuilder::new().with_admin("admin").build();

    // try to add staking token distribution
    let err = suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Token(suite.token_contract()),
            vec![],
        )
        .unwrap_err();

    assert_eq!(ContractError::InvalidAsset {}, err.downcast().unwrap());
}

#[test]
fn unbond_after_new_distribution() {
    let executor = "executor";
    let member = "member";
    let mut suite = SuiteBuilder::new()
        .with_admin("admin")
        .with_unbonding_periods(vec![100])
        .with_initial_balances(vec![(member, 1_000)])
        .with_native_balances("juno", vec![(member, 1_000)])
        .build();

    // delegate before any distribution exists
    suite.delegate(member, 1_000, 100).unwrap();

    // add distribution
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![(100, Decimal::one())],
        )
        .unwrap();

    // unbond
    suite.unbond("member", 1_000, 100).unwrap();
}

#[test]
fn distribution_respects_min_bond() {
    let executor = "executor";
    let members = ["member0", "member1"];
    let mut suite = SuiteBuilder::new()
        .with_admin("admin")
        .with_unbonding_periods(vec![100])
        .with_min_bond(2000)
        .with_initial_balances(vec![(members[0], 1_000), (members[1], 3_000)])
        .with_native_balances("juno", vec![(executor, 1_000)])
        .build();

    // delegate less than min_bond with one account
    suite.delegate(members[0], 1000, 100).unwrap();
    // delegate more than min_bond with another account, such that the total is >= min_bond
    suite.delegate(members[1], 3000, 100).unwrap();

    // add distribution
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![(100, Decimal::one())],
        )
        .unwrap();

    // distribute
    suite
        .distribute_funds(executor, executor, Some(juno(300)))
        .unwrap();

    // withdraw
    suite.withdraw_funds(members[0], None, None).unwrap();
    suite.withdraw_funds(members[1], None, None).unwrap();

    assert_eq!(
        suite.query_balance(members[0], "juno").unwrap(),
        0,
        "member0 should be below min_bond"
    );
    assert_eq!(
        suite.query_balance(members[1], "juno").unwrap(),
        300,
        "member1 should be above min_bond and get everything"
    );
}

#[test]
fn withdraw_adjustment_handled_lazily() {
    // This tests the case that a user bonds before a distribution is created and does not bond again after that.
    // To pass this test, one cannot rely on `WITHDRAW_ADJUSTMENT` being set when bonding.
    let executor = "executor";
    let member = "member";
    let mut suite = SuiteBuilder::new()
        .with_admin("admin")
        .with_unbonding_periods(vec![100])
        .with_min_bond(0)
        .with_initial_balances(vec![(member, 1_000)])
        .with_native_balances("juno", vec![(executor, 1_000)])
        .build();

    // delegate before any distribution exists
    suite.delegate(member, 1_000, 100).unwrap();

    // add distribution
    suite
        .create_distribution_flow(
            "admin",
            executor,
            AssetInfo::Native("juno".to_string()),
            vec![(100, Decimal::one())],
        )
        .unwrap();

    // distribute
    suite
        .distribute_funds(executor, None, Some(juno(500)))
        .unwrap();

    // withdraw
    suite.withdraw_funds(member, member, None).unwrap();
    // member should get rewards
    assert_eq!(suite.query_balance(member, "juno").unwrap(), 500);
}
