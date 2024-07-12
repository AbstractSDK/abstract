use std::str::FromStr;

use abstract_app::std::objects::{
    namespace::Namespace, time_weighted_average::TimeWeightedAverageData,
};
use abstract_client::{builder::cw20_builder, AbstractClient, Application, Environment, Publisher};
use abstract_subscription::{
    contract::interface::SubscriptionInterface,
    msg::{SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg, SubscriptionQueryMsgFns},
    state::{EmissionType, Subscriber, SubscriptionConfig},
    SubscriptionError,
};

pub const WEEK_IN_SECONDS: u64 = 7 * 24 * 60 * 60;

use cosmwasm_std::{coins, Decimal, StdError, Uint128, Uint64};
use cw20_builder::{Cw20Base, Cw20Coin, Cw20ExecuteMsgFns, Cw20QueryMsgFns};
use cw_asset::{AssetInfo, AssetInfoBase, AssetInfoUnchecked};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, prelude::*};

// consts for testing
const DENOM: &str = "abstr";
// 3 days
const INCOME_AVERAGING_PERIOD: Uint64 = Uint64::new(259200);

struct NativeSubscription {
    mock: MockBech32,
    client: AbstractClient<MockBech32>,
    subscription_app: Application<MockBech32, SubscriptionInterface<MockBech32>>,
    payment_asset: AssetInfo,
    emission_cw20: Cw20Base<MockBech32>,
}

#[allow(dead_code)]
struct Cw20Subscription {
    client: AbstractClient<MockBech32>,
    subscription_app: Application<MockBech32, SubscriptionInterface<MockBech32>>,
    payment_asset: AssetInfo,
}

fn deploy_emission(client: &AbstractClient<MockBech32>) -> anyhow::Result<Cw20Base<MockBech32>> {
    let sender = client.sender();
    Ok(client
        .cw20_builder("test", "test", 6)
        .initial_balance(Cw20Coin {
            address: sender.to_string(),
            amount: Uint128::new(1_000_000),
        })
        .admin(sender.to_string())
        .instantiate_with_id("abstract:emission_cw20")?)
}

/// Set up the test environment with the contract installed
fn setup_cw20() -> anyhow::Result<Cw20Subscription> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build()?;

    // Deploy factory_token
    let cw20 = client
        .cw20_builder("test", "test", 6)
        .initial_balance(Cw20Coin {
            address: chain.sender_addr().to_string(),
            amount: Uint128::new(1_000_000),
        })
        .admin(chain.sender_addr())
        .instantiate_with_id("abstract:cw20")?;

    let publisher: Publisher<_> = client
        .publisher_builder(Namespace::new("abstract")?)
        .build()?;
    publisher.publish_app::<SubscriptionInterface<_>>()?;

    let cw20_addr = cw20.address()?;
    let subscription_app: Application<_, SubscriptionInterface<_>> =
        publisher.account().install_app(
            &SubscriptionInstantiateMsg {
                payment_asset: AssetInfoUnchecked::cw20(cw20_addr.clone()),
                subscription_cost_per_second: Decimal::from_str("0.000037")?,
                subscription_per_second_emissions: EmissionType::None,
                // 3 days
                income_averaging_period: INCOME_AVERAGING_PERIOD,
                unsubscribe_hook_addr: None,
            },
            &[],
        )?;

    Ok(Cw20Subscription {
        client,
        subscription_app,
        payment_asset: AssetInfo::cw20(cw20_addr),
    })
}

/// Set up the test environment with the contract installed
fn setup_native(balances: Vec<(&str, &[Coin])>) -> anyhow::Result<NativeSubscription> {
    let chain = MockBech32::new("mock");
    let client = AbstractClient::builder(chain.clone()).build()?;
    client.set_balances(balances.into_iter().map(|(a, b)| (chain.addr_make(a), b)))?;
    let publisher: Publisher<MockBech32> = client
        .publisher_builder(Namespace::new("abstract")?)
        .build()?;
    publisher.publish_app::<SubscriptionInterface<_>>()?;

    let emissions = deploy_emission(&client)?;

    let subscription_app: Application<_, SubscriptionInterface<_>> =
        publisher.account().install_app(
            &SubscriptionInstantiateMsg {
                payment_asset: AssetInfoUnchecked::native(DENOM),
                // https://github.com/AbstractSDK/abstract/pull/92#discussion_r1371693550
                subscription_cost_per_second: Decimal::from_str("0.000037")?,
                subscription_per_second_emissions: EmissionType::SecondShared(
                    Decimal::from_str("0.00005")?,
                    AssetInfoBase::Cw20(emissions.addr_str()?),
                ),
                income_averaging_period: INCOME_AVERAGING_PERIOD,
                unsubscribe_hook_addr: None,
            },
            &[],
        )?;

    emissions.transfer(
        Uint128::new(1_000_000),
        subscription_app.account().proxy()?.to_string(),
    )?;

    Ok(NativeSubscription {
        mock: chain,
        client,
        subscription_app,
        payment_asset: AssetInfo::native(DENOM),
        emission_cw20: emissions,
    })
}

#[test]
fn successful_install() -> anyhow::Result<()> {
    // Set up the environment and contract
    let NativeSubscription {
        client: _,
        subscription_app,
        payment_asset,
        emission_cw20,
        mock: _,
    } = setup_native(vec![])?;

    let addr = emission_cw20.address()?;
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
            unsubscribe_hook_addr: None
        }
    );

    let Cw20Subscription {
        client: _,
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
            unsubscribe_hook_addr: None
        }
    );
    Ok(())
}

#[test]
fn subscribe() -> anyhow::Result<()> {
    let subscriber1 = "subscriber1";
    let subscriber2 = "subscriber2";
    let subscriber3 = "subscriber3";
    let subscriber4 = "subscriber4";

    let sub_amount = coins(500, DENOM);
    let NativeSubscription {
        mock,
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20: _,
    } = setup_native(vec![
        (subscriber1, &sub_amount),
        (subscriber2, &sub_amount),
        (subscriber3, &sub_amount),
        (subscriber4, &sub_amount),
    ])?;

    let subscription_addr = subscription_app.address()?;

    // 2 people subscribe
    subscription_app
        .call_as(&mock.addr_make(subscriber1))
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&mock.addr_make(subscriber2))
        .pay(None, &sub_amount)?;
    let twa = query_twa(&client.environment(), subscription_addr.clone());
    // No income yet
    assert_eq!(twa.cumulative_value, 0);
    assert_eq!(twa.average_value, Decimal::zero());
    // wait the period
    client.wait_seconds(INCOME_AVERAGING_PERIOD.u64())?;

    // Third user subscribes
    subscription_app
        .call_as(&mock.addr_make(subscriber3))
        .pay(None, &sub_amount)?;
    // refresh twa
    subscription_app.refresh_twa()?;
    // It should contain income of previous 2 subscribers
    let twa = query_twa(&client.environment(), subscription_addr.clone());

    // expected value for 2 subscribers (cost * period)
    let two_subs_per_second = Decimal::from_str("0.000037")? * Decimal::from_str("2.0")?;
    let expected_cum = two_subs_per_second * Uint128::from(INCOME_AVERAGING_PERIOD);
    // assert it's equal to the 2 subscribers(rounded)
    assert_eq!(twa.cumulative_value, expected_cum.u128());
    // cum_over_period / time passed
    let expected_average = Decimal::from_ratio(expected_cum, INCOME_AVERAGING_PERIOD);
    assert_eq!(twa.average_value, expected_average);

    // wait the period
    client.wait_seconds(INCOME_AVERAGING_PERIOD.u64())?;
    subscription_app.refresh_twa()?;

    let twa = query_twa(&client.environment(), subscription_addr.clone());

    // 0 new subscribers in this period
    assert_eq!(twa.average_value, Decimal::percent(0));

    // Fourth user subscribes
    subscription_app
        .call_as(&mock.addr_make(subscriber4))
        .pay(None, &sub_amount)?;
    // two subscribers were subbed for two periods
    let first_two_subs =
        two_subs_per_second * Uint128::from(INCOME_AVERAGING_PERIOD * Uint64::new(2));
    // and last one only for one
    let third_sub = Decimal::from_str("0.000037")? * Uint128::from(INCOME_AVERAGING_PERIOD);

    let expected_value = first_two_subs + third_sub;

    let twa = query_twa(&client.environment(), subscription_addr);
    // assert it's equal to the 3 subscribers
    assert_eq!(twa.cumulative_value, expected_value.u128());
    Ok(())
}

#[test]
fn claim_emissions_week_shared() -> anyhow::Result<()> {
    let subscriber1 = "subscriber1";
    let subscriber2 = "subscriber2";
    let sub_amount = coins(500, DENOM);
    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20,
        mock,
    } = setup_native(vec![(subscriber1, &sub_amount), (subscriber2, &sub_amount)])?;
    let subscriber1 = mock.addr_make(subscriber1);
    let subscriber2 = mock.addr_make(subscriber2);

    // 2 users subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &sub_amount)?;

    client.wait_seconds(WEEK_IN_SECONDS)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;
    // check balances
    let balance1 = emission_cw20.balance(subscriber1.to_string())?;
    let total_amount = Decimal::from_str("0.00005")? * Uint128::from(WEEK_IN_SECONDS);
    // 2 users
    let expected_balance = total_amount / Uint128::new(2);
    assert_eq!(balance1.balance, expected_balance);
    let balance2 = emission_cw20.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance);

    // wait 20 seconds
    client.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emission_cw20.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, expected_balance);

    // two weeks -20 seconds because he claimed nothing last time
    client.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;
    // check balances
    let balance1 = emission_cw20.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        // 3 weeks in total
        expected_balance * Uint128::new(3)
    );

    let balance2 = emission_cw20.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance * Uint128::new(3));
    Ok(())
}

#[test]
fn claim_emissions_none() -> anyhow::Result<()> {
    let subscriber1 = "subscriber1";
    let sub_amount = coins(500, DENOM);
    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20: _,
        mock,
    } = setup_native(vec![(subscriber1, &sub_amount)])?;
    let subscriber1 = mock.addr_make(subscriber1);

    subscription_app
        .call_as(&subscription_app.account().manager()?)
        .update_subscription_config(None, None, Some(EmissionType::None), None)?;

    // 1 user subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    client.wait_seconds(WEEK_IN_SECONDS)?;

    let err = subscription_app
        .claim_emissions(subscriber1.to_string())
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();
    assert_eq!(err, SubscriptionError::SubscriberEmissionsNotEnabled {});
    Ok(())
}

#[test]
fn claim_emissions_week_per_user() -> anyhow::Result<()> {
    let subscriber1 = "subscriber1";
    let subscriber2 = "subscriber2";
    let sub_amount = coins(500, DENOM);

    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20,
        mock,
    } = setup_native(vec![(subscriber1, &sub_amount), (subscriber2, &sub_amount)])?;
    let subscriber1 = mock.addr_make(subscriber1);
    let subscriber2 = mock.addr_make(subscriber2);

    subscription_app
        .call_as(&subscription_app.account().manager()?)
        .update_subscription_config(
            None,
            None,
            Some(EmissionType::SecondPerUser(
                Decimal::from_str("0.00005")?,
                AssetInfoBase::Cw20(emission_cw20.addr_str()?),
            )),
            None,
        )?;

    // 2 users subscribe
    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &sub_amount)?;

    client.wait_seconds(WEEK_IN_SECONDS)?;

    // Both users claim emissions
    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;

    let expected_balance = Decimal::from_str("0.00005")? * Uint128::from(WEEK_IN_SECONDS);

    // check balance of user1
    let balance1 = emission_cw20.balance(subscriber1.to_string())?;
    assert_eq!(balance1.balance, expected_balance);

    // check balance of user2
    let balance2 = emission_cw20.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance);

    // wait 20 seconds
    client.wait_seconds(20)?;
    // no double-claims
    subscription_app.claim_emissions(subscriber1.to_string())?;
    let balance = emission_cw20.balance(subscriber1.to_string())?;
    assert_eq!(balance.balance, expected_balance);

    // two weeks -20 seconds because he claimed nothing last time
    client.wait_seconds(WEEK_IN_SECONDS * 2 - 20)?;

    subscription_app.claim_emissions(subscriber1.to_string())?;
    subscription_app.claim_emissions(subscriber2.to_string())?;

    // check balances
    let balance1 = emission_cw20.balance(subscriber1.to_string())?;
    assert_eq!(
        balance1.balance,
        // tree weeks in total
        expected_balance * Uint128::new(3)
    );
    let balance2 = emission_cw20.balance(subscriber2.to_string())?;
    assert_eq!(balance2.balance, expected_balance * Uint128::new(3));

    Ok(())
}

#[test]
fn claim_emissions_errors() -> anyhow::Result<()> {
    let subscriber1 = "subscriber1";

    let sub_amount = coins(500, DENOM);
    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20: _,
        mock,
    } = setup_native(vec![(subscriber1, &sub_amount)])?;
    let subscriber1 = mock.addr_make(subscriber1);

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

    client.wait_seconds(WEEK_IN_SECONDS)?;

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
    let subscriber1 = "subscriber1";
    let subscriber2 = "subscriber2";

    // For 4 weeks with few hours
    let sub_amount = coins(90, DENOM);

    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20,
        mock,
    } = setup_native(vec![(subscriber1, &sub_amount)])?;
    let subscriber1 = mock.addr_make(subscriber1);
    let subscriber2 = mock.addr_make(subscriber2);

    subscription_app
        .call_as(&subscriber1)
        .pay(None, &sub_amount)?;

    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;
    let current_time = client.block_info()?.time;
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
    client.wait_seconds(WEEK_IN_SECONDS * 5)?;
    subscription_app.unsubscribe(vec![subscriber1.to_string()])?;
    let subscriber = subscription_app.subscriber(subscriber1.to_string())?;

    let current_time = client.block_info()?.time;

    assert!(!subscriber.currently_subscribed);

    let subscriber_details: Subscriber = subscriber.subscriber_details.unwrap();
    assert_eq!(
        subscriber_details.last_emission_claim_timestamp,
        current_time
    );

    let b = emission_cw20.balance(subscriber1.to_string())?;
    // 5 weeks passed until unsubscribe
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
    let subscriber1 = "subscriber1";
    let subscriber2 = "subscriber2";
    let NativeSubscription {
        client,
        subscription_app,
        payment_asset: _,
        emission_cw20: _,
        mock,
    } = setup_native(vec![
        (subscriber1, coins(2200, DENOM).as_slice()),
        (subscriber2, coins(220, DENOM).as_slice()),
    ])?;
    let subscriber1 = mock.addr_make(subscriber1);
    let subscriber2 = mock.addr_make(subscriber2);

    subscription_app
        .call_as(&subscriber1)
        .pay(None, &coins(2200, DENOM))?;
    subscription_app
        .call_as(&subscriber2)
        .pay(None, &coins(220, DENOM))?;
    // 1 out of 10 weeks wait
    client.wait_seconds(WEEK_IN_SECONDS)?;
    // Un-sub on not-expired users should error
    let err = subscription_app
        .unsubscribe(vec![subscriber1.to_string(), subscriber2.to_string()])
        .unwrap_err();
    let err: SubscriptionError = err.downcast().unwrap();
    assert_eq!(err, SubscriptionError::NoOneUnsubbed {});

    // wait rest of 9 weeks for the second subscription
    client.wait_seconds(WEEK_IN_SECONDS * 9)?;

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
fn query_twa(chain: &MockBech32, subscription_addr: Addr) -> TimeWeightedAverageData {
    let app = chain.app.borrow();
    let querier = app.wrap();
    abstract_subscription::state::INCOME_TWA
        .query(&querier, subscription_addr)
        .unwrap()
}
