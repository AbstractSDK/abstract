use std::str::FromStr;

use abstract_core::objects::{
    gov_type::GovernanceDetails, time_weighted_average::TimeWeightedAverageData,
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy};
use abstract_subscription::{
    contract::{interface::SubscriptionInterface, CONTRACT_VERSION},
    msg::{SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg, SubscriptionQueryMsgFns},
    state::{EmissionType, Subscriber, SubscriptionConfig},
    SubscriptionError,
};

pub const WEEK_IN_SECONDS: u64 = 7 * 24 * 60 * 60;

use abstract_subscription::contract::SUBSCRIPTION_ID;
use cw20::{msg::Cw20ExecuteMsgFns, Cw20Coin};
use cw20_base::msg::QueryMsgFns;
use cw_asset::{AssetInfo, AssetInfoBase, AssetInfoUnchecked};
use cw_plus_interface::cw20_base::Cw20Base;
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coins, Addr, Decimal, StdError, Uint128, Uint64};

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "abstr";
// 3 days
const INCOME_AVERAGING_PERIOD: Uint64 = Uint64::new(259200);

struct Subscription {
    chain: Mock,
    account: AbstractAccount<Mock>,
    abstr: Abstract<Mock>,
    subscription_app: SubscriptionInterface<Mock>,
    payment_asset: AssetInfo,
}

fn deploy_emission(chain: &Mock) -> anyhow::Result<Cw20Base<Mock>> {
    let emission_cw20 = Cw20Base::new("abstract:emission_cw20", chain.clone());
    let sender = chain.sender();

    emission_cw20.upload()?;
    emission_cw20.instantiate(
        &cw20_base::msg::InstantiateMsg {
            decimals: 6,
            mint: None,
            symbol: "test".to_string(),
            name: "test".to_string(),
            initial_balances: vec![Cw20Coin {
                address: sender.to_string(),
                amount: Uint128::new(1_000_000),
            }],
            marketing: None,
        },
        Some(&sender),
        None,
    )?;
    Ok(emission_cw20)
}

/// Set up the test environment with the contract installed
fn setup_cw20() -> anyhow::Result<Subscription> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Deploy factory_token
    let cw20 = Cw20Base::new("abstract:cw20", mock.clone());

    cw20.upload()?;
    cw20.instantiate(
        &cw20_base::msg::InstantiateMsg {
            decimals: 6,
            mint: None,
            symbol: "test".to_string(),
            name: "test".to_string(),
            initial_balances: vec![Cw20Coin {
                address: sender.clone().into(),
                amount: Uint128::new(1_000_000),
            }],
            marketing: None,
        },
        Some(&sender),
        None,
    )?;

    // Construct the contributors apps
    let subscription_app = SubscriptionInterface::new(SUBSCRIPTION_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    subscription_app.deploy(CONTRACT_VERSION.parse()?, DeployStrategy::Try)?;

    let cw20_addr = cw20.address()?;
    account.install_app(
        subscription_app.clone(),
        &SubscriptionInstantiateMsg {
            payment_asset: AssetInfoUnchecked::cw20(cw20_addr.clone()),
            subscription_cost_per_second: Decimal::from_str("0.000037")?,
            subscription_per_second_emissions: EmissionType::None,
            // 3 days
            income_averaging_period: INCOME_AVERAGING_PERIOD,
            unsubscription_hook_addr: None,
        },
        None,
    )?;

    Ok(Subscription {
        chain: mock,
        account,
        abstr: abstr_deployment,
        subscription_app,
        payment_asset: AssetInfo::cw20(cw20_addr),
    })
}

/// Set up the test environment with the contract installed
fn setup_native() -> anyhow::Result<Subscription> {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create the mock
    let mock = Mock::new(&sender);

    // Construct the contributors apps
    let subscription_app = SubscriptionInterface::new(SUBSCRIPTION_ID, mock.clone());

    // Deploy Abstract to the mock
    let abstr_deployment = Abstract::deploy_on(mock.clone(), sender.to_string())?;

    // Create a new account to install the app onto
    let account =
        abstr_deployment
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_string(),
            })?;

    let emissions = deploy_emission(&mock)?;
    subscription_app.deploy(CONTRACT_VERSION.parse()?, DeployStrategy::Try)?;

    account.install_app(
        subscription_app.clone(),
        &SubscriptionInstantiateMsg {
            payment_asset: AssetInfoUnchecked::native(DENOM),
            // https://github.com/AbstractSDK/abstract/pull/92#discussion_r1371693550
            subscription_cost_per_second: Decimal::from_str("0.000037")?,
            subscription_per_second_emissions: EmissionType::SecondShared(
                Decimal::from_str("0.00005")?,
                AssetInfoBase::Cw20(emissions.addr_str()?),
            ),
            income_averaging_period: INCOME_AVERAGING_PERIOD,
            unsubscription_hook_addr: None,
        },
        None,
    )?;

    emissions.transfer(Uint128::new(1_000_000), account.proxy.addr_str()?)?;

    Ok(Subscription {
        chain: mock,
        account,
        abstr: abstr_deployment,
        subscription_app,
        payment_asset: AssetInfo::native(DENOM),
    })
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let Subscription {
        chain,
        account: _account,
        abstr: _abstr,
        subscription_app,
        payment_asset,
    } = setup_native()?;

    let cw20_emis = Cw20Base::new("abstract:emission_cw20", chain.clone());
    let addr = cw20_emis.address()?;
    let config = subscription_app.config()?;
    assert_eq!(
        config,
        SubscriptionConfig {
            payment_asset,
            subscription_cost_per_second: Decimal::from_str("0.000037")?,
            subscription_per_second_emissions: EmissionType::SecondShared(
                Decimal::from_str("0.00005")?,
                AssetInfoBase::Cw20(addr)
            ),
            unsubscription_hook_addr: None
        }
    );

    let Subscription {
        chain: _,
        account: _account,
        abstr: _abstr,
        subscription_app,
        payment_asset,
    } = setup_cw20()?;

    let config = subscription_app.config()?;
    assert_eq!(
        config,
        SubscriptionConfig {
            payment_asset,
            subscription_cost_per_second: Decimal::from_str("0.000037")?,
            subscription_per_second_emissions: EmissionType::None,
            unsubscription_hook_addr: None
        }
    );
    Ok(())
}

#[test]
fn subscribe() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account: _account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscription_addr = subscription_app.address()?;

    let subscriber1 = Addr::unchecked("subscriber1");
    let subscriber2 = Addr::unchecked("subscriber2");
    let subscriber3 = Addr::unchecked("subscriber3");
    let subscriber4 = Addr::unchecked("subscriber4");

    let sub_amount = coins(500, DENOM);
    chain.set_balances(&[
        (&subscriber1, &sub_amount),
        (&subscriber2, &sub_amount),
        (&subscriber3, &sub_amount),
        (&subscriber4, &sub_amount),
    ])?;

    // 2 people subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &sub_amount)?;
    let twa = query_twa(&chain, subscription_addr.clone());
    // No income yet
    assert_eq!(twa.cumulative_value, 0);
    assert_eq!(twa.average_value, Decimal::zero());
    // wait the period
    chain.wait_seconds(INCOME_AVERAGING_PERIOD.u64())?;

    // Third user subscribes
    subscription_app
        .call_as(&subscriber3)
        .pay(None, &sub_amount)?;
    // refresh twa
    subscription_app.refresh_twa()?;
    // It should contain income of previous 2 subscribers
    let twa = query_twa(&chain, subscription_addr.clone());

    // expected value for 2 subscribers (cost * period)
    let two_subs_per_second = Decimal::from_str("0.000037")? * Decimal::from_str("2.0")?;
    let expected_cum = two_subs_per_second * Uint128::from(INCOME_AVERAGING_PERIOD);
    // assert it's equal to the 2 subscribers(rounded)
    assert_eq!(twa.cumulative_value, expected_cum.u128());
    // cum_over_period / time passed
    let expected_average = Decimal::from_ratio(expected_cum, INCOME_AVERAGING_PERIOD);
    assert_eq!(twa.average_value, expected_average);

    // wait the period
    chain.wait_seconds(INCOME_AVERAGING_PERIOD.u64())?;
    subscription_app.refresh_twa()?;

    let twa = query_twa(&chain, subscription_addr.clone());

    // 0 new subscribers in this period
    assert_eq!(twa.average_value, Decimal::percent(0));

    // Fourth user subscribes
    subscription_app
        .call_as(&subscriber4)
        .pay(None, &sub_amount)?;
    // two subscribers were subbed for two periods
    let first_two_subs =
        two_subs_per_second * Uint128::from(INCOME_AVERAGING_PERIOD * Uint64::new(2));
    // and last one only for one
    let third_sub = Decimal::from_str("0.000037")? * Uint128::from(INCOME_AVERAGING_PERIOD);

    let expected_value = first_two_subs + third_sub;

    let twa = query_twa(&chain, subscription_addr);
    // assert it's equal to the 3 subscribers
    assert_eq!(twa.cumulative_value, expected_value.u128());
    Ok(())
}

#[test]
fn claim_emissions_week_shared() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account: _account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");
    let subscriber2 = Addr::unchecked("subscriber2");

    let sub_amount = coins(500, DENOM);
    chain.set_balances(&[(&subscriber1, &sub_amount), (&subscriber2, &sub_amount)])?;

    // 2 users subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &sub_amount)?;

    chain.wait_seconds(WEEK_IN_SECONDS)?;

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;
    // check balances
    let balance1 = emis.balance(subscriber1.to_string())?;
    let total_amount = Decimal::from_str("0.00005")? * Uint128::from(WEEK_IN_SECONDS);
    // 2 users
    let expected_balance = total_amount / Uint128::new(2);
    assert_eq!(balance1.balance, expected_balance);
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance);

    // wait 20 seconds
    chain.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, expected_balance);

    // two weeks -20 seconds because he claimed nothing last time
    chain.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;
    // check balances
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        // 3 weeks in total
        expected_balance * Uint128::new(3)
    );

    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance * Uint128::new(3));
    Ok(())
}

#[test]
fn claim_emissions_none() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");

    subscription_app
        .call_as(&account.manager.address()?)
        .update_subscription_config(None, None, Some(EmissionType::None), None)?;
    let sub_amount = coins(500, DENOM);
    chain.set_balances(&[(&subscriber1, &sub_amount)])?;

    // 1 user subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    chain.wait_seconds(WEEK_IN_SECONDS)?;

    let err = subscription_app
        .claim_emissions(subscriber1.to_string())
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();
    assert_eq!(err, SubscriptionError::SubscriberEmissionsNotEnabled {});
    Ok(())
}

#[test]
fn claim_emissions_week_per_user() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");
    let subscriber2 = Addr::unchecked("subscriber2");

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());

    subscription_app
        .call_as(&account.manager.address()?)
        .update_subscription_config(
            None,
            None,
            Some(EmissionType::SecondPerUser(
                Decimal::from_str("0.00005")?,
                AssetInfoBase::Cw20(emis.addr_str()?),
            )),
            None,
        )?;

    let sub_amount = coins(500, DENOM);
    chain.set_balances(&[(&subscriber1, &sub_amount), (&subscriber2, &sub_amount)])?;

    // 2 users subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &sub_amount)?;

    chain.wait_seconds(WEEK_IN_SECONDS)?;

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());

    // Both users claim emissions
    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;

    let expected_balance = Decimal::from_str("0.00005")? * Uint128::from(WEEK_IN_SECONDS);

    // check balance of user1
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance1.balance, expected_balance);

    // check balance of user2
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance);

    // wait 20 seconds
    chain.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, expected_balance);

    // two weeks -20 seconds because he claimed nothing last time
    chain.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;

    // check balances
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        // tree weeks in total
        expected_balance * Uint128::new(3)
    );
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance * Uint128::new(3));

    Ok(())
}

#[test]
fn claim_emissions_errors() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account: _account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");

    let sub_amount = coins(500, DENOM);
    chain.set_balances(&[(&subscriber1, &sub_amount)])?;

    // no subs

    let err = subscription_app
        .claim_emissions(subscriber1.to_string())
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();
    assert!(matches!(
        err,
        // can't load subscriber
        SubscriptionError::Std(StdError::NotFound { .. })
    ));

    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    chain.wait_seconds(WEEK_IN_SECONDS)?;

    // double-claim
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let err = subscription_app
        .claim_emissions(subscriber1.to_string())
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();

    assert_eq!(err, SubscriptionError::EmissionsAlreadyClaimed {});
    Ok(())
}

#[test]
fn unsubscribe() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account: _account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");
    let subscriber2 = Addr::unchecked("subscriber2");

    // For 4 weeks with few hours
    let sub_amount = coins(90, DENOM);
    chain.set_balance(&subscriber1, sub_amount.clone())?;

    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;
    let current_time = chain.block_info()?.time;
    assert!(subscriber.currently_subscribed);
    let subscriber_details: Subscriber = subscriber.subscriber_details.unwrap();
    assert_eq!(
        subscriber_details.last_emission_claim_timestamp,
        current_time
    );
    assert!(
        subscriber_details.expiration_timestamp > current_time.plus_seconds(WEEK_IN_SECONDS * 4)
            && subscriber_details.expiration_timestamp
                < current_time.plus_seconds(WEEK_IN_SECONDS * 4).plus_days(1)
    );

    // wait until subscription expires
    chain.wait_seconds(WEEK_IN_SECONDS * 5)?;
    subscription_app.unsubscribe(vec![subscriber1.to_string()])?;
    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;

    let current_time = chain.block_info()?.time;

    assert!(!subscriber.currently_subscribed);

    let subscriber_details: Subscriber = subscriber.subscriber_details.unwrap();
    assert_eq!(
        subscriber_details.last_emission_claim_timestamp,
        current_time
    );

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());
    let b = emis.balance(subscriber1.to_string())?;
    // 5 weeks passed until unsubscription
    assert_eq!(
        b.balance,
        Decimal::from_str("0.00005")? * Uint128::from(WEEK_IN_SECONDS * 5)
    );
    // Unsubscribe on already unsubscribed user should fail
    assert!(subscription_app
        .unsubscribe(vec![subscriber1.to_string()])
        .is_err());

    // Same with not sub
    assert!(subscription_app
        .unsubscribe(vec![subscriber2.to_string()])
        .is_err());
    Ok(())
}

#[test]
fn unsubscribe_part_of_list() -> anyhow::Result<()> {
    let Subscription {
        chain,
        account: _account,
        abstr: _,
        subscription_app,
        payment_asset: _,
    } = setup_native()?;

    let subscriber1 = Addr::unchecked("subscriber1");
    let subscriber2 = Addr::unchecked("subscriber2");

    chain.set_balances(&[
        (&subscriber1, &coins(2200, DENOM)),
        (&subscriber2, &coins(220, DENOM)),
    ])?;
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &coins(2200, DENOM))?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &coins(220, DENOM))?;
    // 1 out of 10 weeks wait
    chain.wait_seconds(WEEK_IN_SECONDS)?;
    // Un-sub on not-expired users should error
    let err = subscription_app
        .unsubscribe(vec![subscriber1.to_string(), subscriber2.to_string()])
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();
    assert_eq!(err, SubscriptionError::NoOneUnsubbed {});

    // wait rest of 9 weeks for the second subscription
    chain.wait_seconds(WEEK_IN_SECONDS * 9)?;

    subscription_app.unsubscribe(vec![subscriber1.to_string(), subscriber2.to_string()])?;

    // sub2 unsubbed
    let subscriber2 = subscription_app.subscriber(subscriber2.to_string())?;
    assert!(!subscriber2.currently_subscribed);

    // subscriber1 not yet unsubscribed
    let subscriber1 = subscription_app.subscriber(subscriber1.to_string())?;
    assert!(subscriber1.currently_subscribed);

    Ok(())
}

// Helper to raw_query twa
fn query_twa(chain: &Mock, subscription_addr: Addr) -> TimeWeightedAverageData {
    let app = chain.app.borrow();
    let querier = app.wrap();
    abstract_subscription::state::INCOME_TWA
        .query(&querier, subscription_addr)
        .unwrap()
}
