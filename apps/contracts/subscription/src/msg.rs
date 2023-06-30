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

use crate::state::UncheckedEmissionType;
use crate::state::{
    Compensation, ContributionConfig, ContributionState, Subscriber, SubscriptionConfig,
    SubscriptionState,
};
use abstract_core::{app, objects::AccountId};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Uint128, Uint64};
use cw_asset::{Asset, AssetInfoUnchecked};

pub type ExecuteMsg = app::ExecuteMsg<SubscriptionExecuteMsg>;
pub type QueryMsg = app::QueryMsg<SubscriptionQueryMsg>;
impl app::AppExecuteMsg for SubscriptionExecuteMsg {}
impl app::AppQueryMsg for SubscriptionQueryMsg {}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub subscription: SubscriptionInstantiateMsg,
    pub contribution: Option<ContributionInstantiateMsg>,
}
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionInstantiateMsg {
    /// Payment asset for
    pub payment_asset: AssetInfoUnchecked,
    pub subscription_cost_per_block: Decimal,
    pub version_control_addr: String,
    pub factory_addr: String,
    pub subscription_per_block_emissions: UncheckedEmissionType,
}

#[cosmwasm_schema::cw_serde]
pub struct ContributionInstantiateMsg {
    pub protocol_income_share: Decimal,
    pub emission_user_share: Decimal,
    pub max_emissions_multiple: Decimal,
    pub token_info: AssetInfoUnchecked,
    pub emissions_amp_factor: Uint128,
    pub emissions_offset: Uint128,
    pub income_averaging_period: Uint64,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
#[cfg_attr(feature = "boot", impl_into(ExecuteMsg))]
pub enum SubscriptionExecuteMsg {
    Pay {
        os_id: AccountId,
    },
    Unsubscribe {
        os_ids: Vec<u32>,
    },
    ClaimCompensation {
        // os_id the OS
        os_id: AccountId,
    },
    ClaimEmissions {
        os_id: AccountId,
    },
    UpdateContributor {
        contributor_os_id: AccountId,
        base_per_block: Option<Decimal>,
        weight: Option<Uint64>,
        expiration_block: Option<Uint64>,
    },
    RemoveContributor {
        os_id: AccountId,
    },
    UpdateSubscriptionConfig {
        payment_asset: Option<AssetInfoUnchecked>,
        version_control_address: Option<String>,
        factory_address: Option<String>,
        subscription_cost: Option<Decimal>,
    },
    UpdateContributionConfig {
        protocol_income_share: Option<Decimal>,
        emission_user_share: Option<Decimal>,
        max_emissions_multiple: Option<Decimal>,
        project_token_info: Option<AssetInfoUnchecked>,
        emissions_amp_factor: Option<Uint128>,
        emissions_offset: Option<Uint128>,
    },
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
#[cfg_attr(feature = "boot", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum SubscriptionQueryMsg {
    #[returns(StateResponse)]
    State {},
    #[returns(ConfigResponse)]
    Config {},
    #[returns(SubscriptionFeeResponse)]
    Fee {},
    #[returns(SubscriberStateResponse)]
    SubscriberState { os_id: AccountId },
    #[returns(ContributorStateResponse)]
    ContributorState { os_id: AccountId },
}

#[cosmwasm_schema::cw_serde]
pub enum DepositHookMsg {
    Pay { os_id: AccountId },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub contribution: ContributionConfig,
    pub subscription: SubscriptionConfig,
}

#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    pub contribution: ContributionState,
    pub subscription: SubscriptionState,
}

#[cosmwasm_schema::cw_serde]
pub struct SubscriptionFeeResponse {
    pub fee: Asset,
}

#[cosmwasm_schema::cw_serde]
pub struct SubscriberStateResponse {
    pub currently_subscribed: bool,
    pub subscriber_details: Subscriber,
}

#[cosmwasm_schema::cw_serde]
pub struct ContributorStateResponse {
    pub compensation: Compensation,
}
