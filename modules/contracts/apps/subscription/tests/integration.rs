use std::str::FromStr;

use abstract_core::objects::{
    gov_type::GovernanceDetails, time_weighted_average::TimeWeightedAverageData, AccountId,
};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy, VCExecFns};
use abstract_subscription::{
    contract::{interface::SubscriptionInterface, CONTRACT_VERSION},
    msg::{
        SubscriberStateResponse, SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg,
        SubscriptionQueryMsgFns,
    },
    state::{EmissionType, Subscriber, SubscriptionConfig},
    WEEK_IN_SECONDS,
};

use abstract_subscription::contract::SUBSCRIPTION_ID;
use abstract_subscription::msg as subscr_msg;
use cw20::{Cw20Coin, Cw20ExecuteMsgFns};
use cw20_base::{contract::Cw20Base, msg::QueryMsgFns};
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetInfoUnchecked};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{coins, Addr, Decimal, Querier, Uint128, Uint64};

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
        }
    );

    // Set up the environment and contract
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
        }
    );
    Ok(())
}

#[test]
fn subscribe() -> anyhow::Result<()> {
    // Set up the environment and contract
    let Subscription {
        chain,
        account: _account,
        abstr,
        subscription_app,
        payment_asset,
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
        .pay(None, None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, None, &sub_amount)?;
    let twa = query_twa(&chain, subscription_addr.clone());
    // No income yet
    assert_eq!(twa.cumulative_value, 0);
    assert_eq!(twa.average_value, Decimal::zero());
    // wait the period
    chain.wait_seconds(INCOME_AVERAGING_PERIOD.u64())?;

    // Third user subscribes
    subscription_app
        .call_as(&subscriber3)
        .pay(None, None, &sub_amount)?;
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
        .pay(None, None, &sub_amount)?;
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
fn claim_emissions() -> anyhow::Result<()> {
    // Set up the environment and contract
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
        .pay(None, None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, None, &sub_amount)?;

    chain.wait_seconds(WEEK_IN_SECONDS)?;

    let emis = Cw20Base::new("abstract:emission_cw20", chain.clone());

    subscription_app.claim_emissions(subscriber1.to_string())?;
    // check balance
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, Uint128::one());

    // wait 20 seconds
    chain.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, Uint128::one());

    // -20 seconds because he claimed nothing last time
    chain.wait_seconds(WEEK_IN_SECONDS - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    // check balance
    let balance = emis.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, Uint128::one() + Uint128::one());
    Ok(())
}

#[test]
fn unsubscribe() -> anyhow::Result<()> {
    // Set up the environment and contract
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
    chain.set_balances(&[(&subscriber1, &sub_amount), (&subscriber2, &sub_amount)])?;

    subscription_app
        .call_as(&subscriber1)
        .pay(None, None, &sub_amount)?;

    let subscriber = subscription_app.subscriber_state(subscriber1.to_string())?;

    println!("subscriber: {subscriber:?}");

    let current_time = chain.block_info()?.time;
    assert_eq!(
        subscriber,
        SubscriberStateResponse {
            currently_subscribed: true,
            subscriber_details: Subscriber {
                expiration_timestamp: current_time.plus_seconds(WEEK_IN_SECONDS * 10),
                last_emission_claim_timestamp: current_time,
                unsubscribe_hook_addr: None
            }
        }
    );

    // wait until subscription expires
    chain.wait_seconds(WEEK_IN_SECONDS * 10)?;
    subscription_app.unsubscribe(vec![subscriber1.to_string()])?;
    let subscriber = subscription_app.subscriber_state(subscriber1.to_string())?;

    let current_time = chain.block_info()?.time;

    println!("subscriber: {subscriber:?}");
    assert_eq!(
        subscriber,
        SubscriberStateResponse {
            currently_subscribed: false,
            subscriber_details: Subscriber {
                expiration_timestamp: current_time,
                last_emission_claim_timestamp: current_time,
                unsubscribe_hook_addr: None
            }
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

    subscription_app
        .call_as(&subscriber2)
        .pay(None, None, &sub_amount)?;

    // TODO: do we want to error on falsy unsub?

    // 1 out of 10 weeks wait
    chain.wait_seconds(WEEK_IN_SECONDS * 1)?;
    // Un-sub on not-expired user shouldn't do anything
    subscription_app.unsubscribe(vec![subscriber2.to_string()])?;

    let subscriber = subscription_app.subscriber_state(subscriber2.to_string())?;

    let current_time = chain.block_info()?.time;
    assert_eq!(
        subscriber,
        SubscriberStateResponse {
            currently_subscribed: true,
            subscriber_details: Subscriber {
                expiration_timestamp: current_time.plus_seconds(WEEK_IN_SECONDS * 9),
                // Not even claim emission
                last_emission_claim_timestamp: current_time.minus_seconds(WEEK_IN_SECONDS * 1),
                unsubscribe_hook_addr: None
            }
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
