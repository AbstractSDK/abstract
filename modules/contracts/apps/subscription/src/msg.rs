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
//! We use a [`TimeWeightedAverage`](crate::objects::time_weighted_average::TimeWeightedAverage) of the ongoing income to to determine a per-block income.
//! We average the income over a monthly basis.
//!
//! ## Emissions
//! Protocol emissions are an important part of creating a tight community of users and contributors around your product. The emissions feature of this
//! module allows you to easily configure emission parameters based on your needs.
//! These emission parameters are set when creating the module and are described on the [`UncheckedEmissionType`] struct.
//!
//! ## Contributions
//! The contribution feature of this contract can be used to provide direct incentives for users to join in building out your product.
//! Each contributor is registered with his own [`Compensation`] parameters.
//! * The total income of the system is shared between the DAO and the contributors. See [`ContributionConfig`].
//! * (optional) Token emissions to contributor (and users) are dynamically set based on the protocol's income. Meaning that the token emissions will rise if demand/income falls and vice-versa.

use crate::contract::SubscriptionApp;
use crate::state::UncheckedEmissionType;
use crate::state::{
    Compensation, ContributionState, ContributorsConfig, Subscriber, SubscribersConfig,
    SubscriptionState,
};
use abstract_core::objects::AccountId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128, Uint64};
use cw_asset::{Asset, AssetInfoUnchecked};

abstract_app::app_msg_types!(
    SubscriptionApp,
    SubscriptionExecuteMsg,
    SubscriptionQueryMsg
);

/// Subscription migration message
#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}

/// Subscription instantiation message
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionInstantiateMsg {
    /// Instantiation message for subscribers
    pub subscribers: SubscribersInstantiateMsg,
    /// Optional instantiation message for setting up contributions
    pub contributors: Option<ContributorsInstantiateMsg>,
}

/// Subscribers instantiation message
#[cosmwasm_schema::cw_serde]
pub struct SubscribersInstantiateMsg {
    /// Asset for payment
    pub payment_asset: AssetInfoUnchecked,
    /// Only addr that can register Abstract Account
    pub factory_addr: String,
    /// Cost of the subscription on a per-block basis.
    pub subscription_cost_per_block: Decimal,
    /// Subscription emissions per block
    pub subscription_per_block_emissions: UncheckedEmissionType,
}

/// Subscribers instantiation message
#[cosmwasm_schema::cw_serde]
pub struct ContributorsInstantiateMsg {
    /// Percentage of income that is redirected to the protocol
    pub protocol_income_share: Decimal,
    /// Percentage of emissions allocated to users
    pub emission_user_share: Decimal,
    /// Max emissions (when income = 0) = max_emissions_multiple * floor_emissions
    pub max_emissions_multiple: Decimal,
    /// Emissions amplification factor in inverse emissions <-> target equation
    pub emissions_amp_factor: Uint128,
    /// Emissions offset factor in inverse emissions <-> target equation
    pub emissions_offset: Uint128,
    /// How often update income average
    pub income_averaging_period: Uint64,
    /// token: TODO
    pub token_info: AssetInfoUnchecked,
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
    /// Claim the compensation for contributor
    ClaimCompensation {
        /// Abstract account id of contributor
        os_id: AccountId,
    },
    /// Claim the emissions for subscriber
    ClaimEmissions {
        /// Abstract account id of subscriber
        os_id: AccountId,
    },
    /// Update/add the contributor config
    UpdateContributor {
        /// Abstract account id of contributor
        os_id: AccountId,
        /// Base amount payment per block
        base_per_block: Option<Decimal>,
        /// Weight of the contributor
        weight: Option<Uint64>,
        /// Block id when "contract" with this contributor expires
        expiration_block: Option<Uint64>,
    },
    /// Remove the contributor
    RemoveContributor {
        /// Abstract account id of contributor
        os_id: AccountId,
    },
    /// Update config of subscription
    UpdateSubscriptionConfig {
        /// New asset for payment
        payment_asset: Option<AssetInfoUnchecked>,
        /// New asset for payment
        factory_address: Option<String>,
        /// new subscription_cost_per_block
        subscription_cost_per_block: Option<Decimal>,
        // TODO?: subscription_per_block_emissions
    },
    /// Update config of contributors
    UpdateContributionConfig {
        /// New ercentage of income that is redirected to the protocol
        protocol_income_share: Option<Decimal>,
        /// New ercentage of emissions allocated to users
        emission_user_share: Option<Decimal>,
        /// New max emissions (when income = 0) = max_emissions_multiple * floor_emissions
        max_emissions_multiple: Option<Decimal>,
        /// New emissions amplification factor in inverse emissions <-> target equation
        emissions_amp_factor: Option<Uint128>,
        /// New emissions offset factor in inverse emissions <-> target equation
        emissions_offset: Option<Uint128>,
        /// Change project token
        project_token_info: Option<AssetInfoUnchecked>,
    },
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
    #[returns(ConfigResponse)]
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
    /// Get state of the contributor
    #[returns(ContributorStateResponse)]
    ContributorState {
        /// Abstract Account Id of contributor
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

/// Query response for [`SubscriptionQueryMsg::Config`]
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// Config for the contributors
    pub contribution: ContributorsConfig,
    /// Config for the subscribers
    pub subscription: SubscribersConfig,
}

/// Query response for [`SubscriptionQueryMsg::State`]
#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    /// State of contributors
    pub contribution: ContributionState,
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

/// Query response for [`SubscriptionQueryMsg::ContributorState`]
#[cosmwasm_schema::cw_serde]
pub struct ContributorStateResponse {
    /// Compensation details for contributors
    pub compensation: Compensation,
}
