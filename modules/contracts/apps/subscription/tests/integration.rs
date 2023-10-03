use abstract_core::objects::{gov_type::GovernanceDetails, AccountId};
use abstract_interface::{Abstract, AbstractAccount, AppDeployer, VCExecFns};
use abstract_subscription::{
    contract::{interface::SubscriptionInterface, CONTRACT_VERSION},
    msg::{SubscriptionExecuteMsgFns, SubscriptionInstantiateMsg, SubscriptionQueryMsgFns},
    state::{EmissionType, SubscriptionConfig},
};

use abstract_subscription_contributors::{
    contract::APP_VERSION,
    msg::{ContributorsExecuteMsgFns, ContributorsInstantiateMsg, ContributorsQueryMsgFns},
    state::ContributorsConfig,
    ContributorsInterface,
};
use abstract_subscription_interface::contributors::msg as contr_msg;
use abstract_subscription_interface::subscription::msg as subscr_msg;
use abstract_subscription_interface::{CONTRIBUTORS_ID, SUBSCRIPTION_ID};
use cw20::Cw20Coin;
use cw20_base::contract::Cw20Base;
use cw_asset::{AssetInfoBase, AssetInfoUnchecked};
// Use prelude to get all the necessary imports
use cw_orch::{anyhow, deploy::Deploy, prelude::*};

use cosmwasm_std::{Addr, Decimal, Uint128};

// consts for testing
const ADMIN: &str = "admin";

struct Subscription {
    chain: Mock,
    account: AbstractAccount<Mock>,
    abstr: Abstract<Mock>,
    subscription_app: SubscriptionInterface<Mock>,
    payment_cw20: Cw20Base<Mock>,
}

fn deploy_emission(subscibers: &Subscription) -> anyhow::Result<Cw20Base<Mock>> {
    let emission_cw20 = Cw20Base::new("abstract:emission_cw20", subscibers.chain.clone());
    let sender = subscibers.chain.sender();
    emission_cw20.set_code_id(subscibers.payment_cw20.code_id()?);
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

fn enable_contributors(
    subscibers: &Subscription,
    cw20_emission: &Cw20Base<Mock>,
) -> anyhow::Result<ContributorsInterface<Mock>> {
    let contributors_app = ContributorsInterface::new(CONTRIBUTORS_ID, subscibers.chain.clone());
    contributors_app.deploy(APP_VERSION.parse()?)?;
    subscibers.account.install_app(
        contributors_app.clone(),
        &ContributorsInstantiateMsg {
            protocol_income_share: Default::default(),
            emission_user_share: Default::default(),
            max_emissions_multiple: Default::default(),
            emissions_amp_factor: Default::default(),
            emissions_offset: Default::default(),
            token_info: cw_asset::AssetInfoBase::Native("test".to_owned()),
        },
        None,
    )?;

    subscibers.account.manager.execute_on_module(
        SUBSCRIPTION_ID,
        subscr_msg::ExecuteMsg::from(
            subscr_msg::SubscriptionExecuteMsg::UpdateSubscriptionConfig {
                payment_asset: None,
                factory_address: None,
                subscription_cost_per_week: None,
                contributors_enabled: Some(true),
                subscription_per_week_emissions: Some(EmissionType::IncomeBased(
                    AssetInfoUnchecked::Cw20(cw20_emission.addr_str()?),
                )),
            },
        ),
    )?;
    Ok(contributors_app)
}

/// Set up the test environment with the contract installed
fn setup() -> anyhow::Result<Subscription> {
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

    subscription_app.deploy(APP_VERSION.parse()?)?;

    let cw20_addr = cw20.addr_str()?;
    account.install_app(
        subscription_app.clone(),
        &SubscriptionInstantiateMsg {
            factory_addr: cw20_addr.clone(),
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
        payment_cw20: cw20,
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
        payment_cw20,
    } = setup()?;

    let config = subscription_app.config()?;
    assert_eq!(
        config,
        ConfigResponse {
            subscription: SubscriptionConfig {
                factory_address: payment_cw20.address()?,
                payment_asset: cw_asset::AssetInfoBase::Cw20(payment_cw20.address()?),
                subscription_cost_per_week: Decimal::percent(1),
                subscription_per_week_emissions: EmissionType::None,
                contributors_enabled: false
            }
        }
    );
    Ok(())
}

#[test]
fn successful_install_with_contributors() -> anyhow::Result<()> {
    // Set up the environment and contract
    let subscription = setup()?;

    let cw20_emissions = deploy_emission(&subscription)?;
    let contributors = enable_contributors(&subscription, &cw20_emissions)?;

    let subscr_config = subscription.subscription_app.config()?;
    assert_eq!(
        subscr_config,
        SubscriptionConfig {
            factory_address: subscription.payment_cw20.address()?,
            payment_asset: cw_asset::AssetInfoBase::Cw20(subscription.payment_cw20.address()?),
            subscription_cost_per_week: Decimal::percent(1),
            subscription_per_week_emissions: EmissionType::IncomeBased(AssetInfoBase::Cw20(
                cw20_emissions.address()?
            )),
            contributors_enabled: true
        }
    );

    let contr_config = contributors.config()?;
    assert_eq!(
        contr_config,
        ContributorsConfig {
            protocol_income_share: todo!(),
            emission_user_share: todo!(),
            max_emissions_multiple: todo!(),
            emissions_amp_factor: todo!(),
            emissions_offset: todo!(),
            token_info: todo!()
        }
    );
    Ok(())
}
