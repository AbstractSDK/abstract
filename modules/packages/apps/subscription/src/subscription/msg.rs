#![warn(missing_docs)]

//! # Subscription Add-On
//!
//! `abstract_core::subscription` provides OS owners with a tool to easily create smart-contract subscriptions for their products.
//!
//! ## Description
//! The subscription contract has three main uses.
//! 1. Provide a way to earn income with a subscription-style modal.
//! 2. Distribute income and native assets to project contributors. (optional)
//! 3. Distribute a native asset to your active users. (optional)
//!
//! ## Income
//! The income of the instance can change over time as subscribers join and leave.
//! If we want our infrastructure to change parameters based on the income of the unit, then we need a way of keeping track of that income.
//! Because blockchains don't have a notion of monthly settlement we settled on a per-month payment schema.
//! We use a [`TimeWeightedAverage`](crate::objects::time_weighted_average::TimeWeightedAverage) of the ongoing income to to determine a per-week income.
//! We average the income over a monthly basis.
//!
//! ## Emissions
//! Protocol emissions are an important part of creating a tight community of users and contributors around your product. The emissions feature of this
//! module allows you to easily configure emission parameters based on your needs.
//! These emission parameters are set when creating the module and are described on the [`EmissionType`] struct.
//!
//! ## Contributions
//! The contribution feature of this contract can be used to provide direct incentives for users to join in building out your product.
//! Each contributor is registered with his own [`Compensation`] parameters.
//! * The total income of the system is shared between the DAO and the contributors. See [`ContributionConfig`].
//! * (optional) Token emissions to contributor (and users) are dynamically set based on the protocol's income. Meaning that the token emissions will rise if demand/income falls and vice-versa.

use super::state::{EmissionType, Subscriber, SubscriptionConfig, SubscriptionState};
use abstract_core::app;
use abstract_core::objects::AccountId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint64};
use cw_asset::{Asset, AssetInfoUnchecked};

/// Top-level Abstract App execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = app::ExecuteMsg<SubscriptionExecuteMsg>;
/// Top-level Abstract App instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = app::InstantiateMsg<SubscriptionInstantiateMsg>;
/// Top-level Abstract App query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = app::QueryMsg<SubscriptionQueryMsg>;
/// Top-level Abstract App migrate message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type MigrateMsg = app::MigrateMsg<SubscriptionMigrateMsg>;

impl app::AppExecuteMsg for SubscriptionExecuteMsg {}
impl app::AppQueryMsg for SubscriptionQueryMsg {}

/// Subscription migration message
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionMigrateMsg {}

/// Subscription instantiation message
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionInstantiateMsg {
    /// Asset for payment
    pub payment_asset: AssetInfoUnchecked,
    /// Only addr that can register Abstract Account
    pub factory_addr: String,
    /// Cost of the subscription on a per-week basis.
    pub subscription_cost_per_week: Decimal,
    /// Subscription emissions per week
    pub subscription_per_week_emissions: EmissionType<String>,
    /// How often update income average
    pub income_averaging_period: Uint64,
}

/// App execution messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum SubscriptionExecuteMsg {
    /// Subscriber payment
    /// TODO?: could be automated with cron
    Pay {
        /// Abstract account id of new subscriber
        /// You can subscribe for other abstract account
        /// TODO: make it optional to default to the proxy
        os_id: AccountId,
    },
    /// Unsubscribe inactive accounts
    /// TODO?: could be automated with cron
    Unsubscribe {
        /// List of inactive accounts to move to the `DORMANT_SUBSCRIBERS` list
        os_ids: Vec<AccountId>,
    },
    /// Claim the emissions for subscriber
    ClaimEmissions {
        /// Abstract account id of subscriber
        os_id: AccountId,
    },
    /// Update config of subscription
    UpdateSubscriptionConfig {
        /// New asset for payment
        payment_asset: Option<AssetInfoUnchecked>,
        /// New asset for payment
        factory_address: Option<String>,
        /// new subscription_cost_per_week
        subscription_cost_per_week: Option<Decimal>,
        /// Enable contributors
        contributors_enabled: Option<bool>,
        /// Subscription emissions per week
        subscription_per_week_emissions: Option<EmissionType<String>>,
    },
    /// Refresh TWA value
    RefreshTWA {},
}

/// Subscriptions query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum SubscriptionQueryMsg {
    /// Get state of subscriptions and contributors
    #[returns(StateResponse)]
    State {},
    /// Get config of subscriptions and contributors
    #[returns(SubscriptionConfig)]
    Config {},
    /// Get minimum of one month's worth to (re)-subscribe.
    #[returns(SubscriptionFeeResponse)]
    Fee {},
    /// Get state of the subscriber
    #[returns(SubscriberStateResponse)]
    SubscriberState {
        /// Abstract Account Id of subscriber  
        os_id: AccountId,
    },
}

/// Cw20 hook message
#[cosmwasm_schema::cw_serde]
pub enum DepositHookMsg {
    /// Subscriber payment
    Pay {
        /// Subscriber account id
        os_id: AccountId,
    },
}

/// Query response for [`SubscriptionQueryMsg::State`]
#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    /// State of subscribers
    pub subscription: SubscriptionState,
}

/// Query response for [`SubscriptionQueryMsg::Fee`]
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionFeeResponse {
    /// minimum of one month's worth to (re)-subscribe.
    pub fee: Asset,
}

/// Query response for [`SubscriptionQueryMsg::SubscriberState`]
#[cosmwasm_schema::cw_serde]
pub struct SubscriberStateResponse {
    /// If the user currently active subscriber
    pub currently_subscribed: bool,
    /// State of the subscription
    pub subscriber_details: Subscriber,
}
