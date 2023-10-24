use std::str::FromStr;

use abstract_core::objects::{
    gov_type::GovernanceDetails, time_weighted_average::TimeWeightedAverageData,
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy};
use abstract_subscription::{
    contract::{interface::SubscriptionInterface, CONTRACT_VERSION},
    msg::{
        SubscriberResponse, SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg,
        SubscriptionQueryMsgFns,
    },
    state::{EmissionType, Subscriber, SubscriptionConfig},
    SubscriptionError, WEEK_IN_SECONDS,
};

use abstract_subscription::contract::SUBSCRIPTION_ID;
use cw20::{Cw20Coin, Cw20ExecuteMsgFns};
use cw20_base::{contract::Cw20Base, msg::QueryMsgFns};
use cw_asset::{AssetInfo, AssetInfoBase, AssetInfoUnchecked};
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
            subscription_cost_per_week: Decimal::from_str("0.1")?,
            subscription_per_week_emissions: EmissionType::None,
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
            subscription_cost_per_week: Decimal::from_str("0.1")?,
            subscription_per_week_emissions: EmissionType::WeekShared(
                Decimal::from_str("2.0")?,
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
            subscription_cost_per_week: Decimal::from_str("0.1")?,
            subscription_per_week_emissions: EmissionType::WeekShared(
                Decimal::from_str("2.0")?,
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
            subscription_cost_per_week: Decimal::from_str("0.1")?,
            subscription_per_week_emissions: EmissionType::None,
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
    let expected_value = Decimal::from_str("0.2")? * Uint128::from(INCOME_AVERAGING_PERIOD);
    // assert it's equal to the 2 subscribers
    assert_eq!(twa.cumulative_value, expected_value.u128());
    assert_eq!(twa.average_value, Decimal::from_str("0.2")?);

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
        Decimal::from_str("0.2")? * Uint128::from(INCOME_AVERAGING_PERIOD * Uint64::new(2));
    // and last one only for one
    let third_sub = Decimal::from_str("0.1")? * Uint128::from(INCOME_AVERAGING_PERIOD);

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
    assert_eq!(balance1.balance, Uint128::one());
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, Uint128::one());

    // wait 20 seconds
    chain.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, Uint128::one());

    // two weeks -20 seconds because he claimed nothing last time
    chain.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;
    // check balances
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        Uint128::one() + Uint128::one() + Uint128::one()
    );

    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(
        balance2.balance,
        Uint128::one() + Uint128::one() + Uint128::one()
    );
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
            Some(EmissionType::WeekPerUser(
                Decimal::one(),
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

    // check balance of user1
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance1.balance, Uint128::one());

    // check balance of user2
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, Uint128::one());

    // wait 20 seconds
    chain.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, Uint128::one());

    // two weeks -20 seconds because he claimed nothing last time
    chain.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;

    // check balances
    let balance1 = emis.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        // tree weeks in total
        Uint128::one() + Uint128::one() + Uint128::one()
    );
    let balance2 = emis.balance(subscriber2.to_string())?;
    assert_eq!(
        balance2.balance,
        Uint128::one() + Uint128::one() + Uint128::one()
    );

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

    let sub_amount = coins(1, DENOM);
    chain.set_balance(&subscriber1, sub_amount.clone())?;

    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;

    let current_time = chain.block_info()?.time;
    assert_eq!(
        subscriber,
        SubscriberResponse {
            currently_subscribed: true,
            subscriber_details: Some(Subscriber {
                expiration_timestamp: current_time.plus_seconds(WEEK_IN_SECONDS * 10),
                last_emission_claim_timestamp: current_time,
            })
        }
    );

    // wait until subscription expires
    chain.wait_seconds(WEEK_IN_SECONDS * 10)?;
    subscription_app.unsubscribe(vec![subscriber1.to_string()])?;
    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;

    let current_time = chain.block_info()?.time;

    assert_eq!(
        subscriber,
        SubscriberResponse {
            currently_subscribed: false,
            subscriber_details: Some(Subscriber {
                expiration_timestamp: current_time,
                last_emission_claim_timestamp: current_time,
            })
        }
    );

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());
    let b = emis.balance(subscriber1.to_string())?;
    // 10 weeks passed 2 tokens shared for sub
    assert_eq!(b.balance, Uint128::new(2 * 10));
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
        (&subscriber1, &coins(10, DENOM)),
        (&subscriber2, &coins(1, DENOM)),
    ])?;
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &coins(10, DENOM))?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &coins(1, DENOM))?;

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

    let current_time = chain.block_info()?.time;

    let subscriber2 = subscription_app.subscriber(subscriber2.to_string())?;

    assert_eq!(
        subscriber2,
        SubscriberResponse {
            currently_subscribed: false,
            subscriber_details: Some(Subscriber {
                expiration_timestamp: current_time,
                last_emission_claim_timestamp: current_time,
            })
        }
    );

    // subscriber1 not yet unsubscribed
    let subscriber1 = subscription_app.subscriber(subscriber1.to_string())?;

    assert_eq!(
        subscriber1,
        SubscriberResponse {
            currently_subscribed: true,
            subscriber_details: Some(Subscriber {
                // 90 more weeks
                expiration_timestamp: current_time.plus_seconds(WEEK_IN_SECONDS * 90),
                // 10 weeks ago subbed, and unsub of other user didn't affect this user
                last_emission_claim_timestamp: current_time.minus_seconds(WEEK_IN_SECONDS * 10),
            })
        }
    );
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
