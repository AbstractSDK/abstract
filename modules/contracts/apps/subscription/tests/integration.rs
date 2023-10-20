use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, DeployStrategy, VCExecFns};
use abstract_subscription::{
    contract::{interface::SubscriptionInterface, CONTRACT_VERSION},
    msg::{SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg, SubscriptionQueryMsgFns},
    state::{EmissionType, SubscriptionConfig},
};

use abstract_subscription::contract::SUBSCRIPTION_ID;
use abstract_subscription::msg as subscr_msg;
use cw20::Cw20Coin;
use cw20_base::contract::Cw20Base;
use cw_asset::{Asset, AssetInfo, AssetInfoBase, AssetInfoUnchecked};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{Addr, Decimal, Uint128};

// consts for testing
const ADMIN: &str = "admin";
const DENOM: &str = "abstr";

struct Subscription {
    chain: Mock,
    account: AbstractAccount<Mock>,
    abstr: Abstract<Mock>,
    subscription_app: SubscriptionInterface<Mock>,
    payment_asset: AssetInfo,
}

fn deploy_emission(subscibers: &Subscription) -> anyhow::Result<Cw20Base<Mock>> {
    let emission_cw20 = Cw20Base::new("abstract:emission_cw20", subscibers.chain.clone());
    let sender = subscibers.chain.sender();

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
            subscription_cost_per_week: Decimal::percent(1),
            subscription_per_week_emissions: EmissionType::None,
            // 3 days
            income_averaging_period: 259200u64.into(),
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

    subscription_app.deploy(CONTRACT_VERSION.parse()?, DeployStrategy::Try)?;

    account.install_app(
        subscription_app.clone(),
        &SubscriptionInstantiateMsg {
            payment_asset: AssetInfoUnchecked::native(DENOM),
            subscription_cost_per_week: Decimal::percent(1),
            subscription_per_week_emissions: EmissionType::None,
            // 3 days
            income_averaging_period: 259200u64.into(),
        },
        None,
    )?;

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
            subscription_cost_per_week: Decimal::percent(1),
            subscription_per_week_emissions: EmissionType::None,
        }
    );
    Ok(())
}

#[test]
fn subscribe() -> anyhow::Result<()> {
    // Set up the environment and contract
    let Subscription {
        chain: _,
        account: _account,
        abstr,
        subscription_app,
        payment_asset,
    } = setup_cw20()?;

    let new_subscriber =
        abstr
            .account_factory
            .create_default_account(GovernanceDetails::Monarchy {
                monarch: ADMIN.to_owned(),
            })?;
    let subscriber_proxy_addr = new_subscriber.proxy.address()?;
    // subscription_app.call_as(&subscriber_proxy_addr).pay(None)?;
    Ok(())
}
