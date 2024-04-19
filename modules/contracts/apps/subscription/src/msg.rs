#![warn(missing_docs)]

//! # Subscription Add-On
//!
//! `abstract_std::subscription` provides OS owners with a tool to easily create smart-contract subscriptions for their products.
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
//! We use a [`TimeWeightedAverage`](crate::objects::time_weighted_average::TimeWeightedAverage) of the ongoing income to to determine a per-second income.
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

use abstract_app::sdk::cw_helpers::Clearable;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{to_json_binary, Addr, Binary, CosmosMsg, Decimal, StdResult, Uint64, WasmMsg};
use cw_asset::{Asset, AssetInfoUnchecked};

use super::state::{EmissionType, Subscriber, SubscriptionConfig, SubscriptionState};
use crate::contract::SubscriptionApp;

abstract_app::app_msg_types!(
    SubscriptionApp,
    SubscriptionExecuteMsg,
    SubscriptionQueryMsg
);

/// Subscription migration message
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionMigrateMsg {}

/// Subscription instantiation message
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionInstantiateMsg {
    /// Asset for payment
    pub payment_asset: AssetInfoUnchecked,
    /// Cost of the subscription on a per-second basis.
    pub subscription_cost_per_second: Decimal,
    /// Subscription emissions per second
    pub subscription_per_second_emissions: EmissionType<String>,
    /// How often update income average
    pub income_averaging_period: Uint64,
    /// Unsubscription hook addr to send [unsubscribe message](`crate::msg::UnsubscribedHookMsg`)
    pub unsubscribe_hook_addr: Option<String>,
}

/// App execution messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum SubscriptionExecuteMsg {
    #[cfg_attr(feature = "interface", payable)]
    /// Subscriber payment
    Pay {
        /// Address of new subscriber
        /// defaults to the sender
        subscriber_addr: Option<String>,
    },
    /// Unsubscribe inactive accounts
    Unsubscribe {
        /// List of inactive accounts to move to the `DORMANT_SUBSCRIBERS` list
        unsubscribe_addrs: Vec<String>,
    },
    /// Claim the emissions for subscriber
    ClaimEmissions {
        /// Address of subscriber
        addr: String,
    },
    /// Update config of subscription
    UpdateSubscriptionConfig {
        /// New asset for payment
        payment_asset: Option<AssetInfoUnchecked>,
        /// new subscription_cost_per_second
        subscription_cost_per_second: Option<Decimal>,
        /// Subscription emissions per second
        subscription_per_second_emissions: Option<EmissionType<String>>,
        /// New unsubscribe_hook_addr
        unsubscribe_hook_addr: Option<Clearable<String>>,
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
    /// Returns [`StateResponse`]
    #[returns(StateResponse)]
    State {},
    /// Get config of subscriptions and contributors
    /// Returns [`SubscriptionConfig`]
    #[returns(SubscriptionConfig)]
    Config {},
    /// Get minimum of one month's worth to (re)-subscribe.
    /// Returns [`SubscriptionFeeResponse`]
    #[returns(SubscriptionFeeResponse)]
    Fee {},
    /// Get state of the subscriber
    /// Returns [`SubscriberResponse`]
    #[returns(SubscriberResponse)]
    Subscriber {
        /// Address of subscriber  
        addr: String,
    },
    /// Get list of subscribers
    /// Returns [`SubscribersResponse`]
    #[returns(SubscribersResponse)]
    Subscribers {
        /// Start after subscriber address
        start_after: Option<Addr>,
        /// Limit
        limit: Option<u64>,
        /// Get list of expired(inactive) subscribers instead
        expired_subs: Option<bool>,
    },
}

/// Cw20 hook message
#[cosmwasm_schema::cw_serde]
pub enum DepositHookMsg {
    /// Subscriber payment
    Pay {
        /// Subscriber Addr
        /// defaults to the sender
        subscriber_addr: Option<String>,
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

/// Query response for [`SubscriptionQueryMsg::Subscriber`]
#[cosmwasm_schema::cw_serde]
pub struct SubscriberResponse {
    /// If the user currently active subscriber
    pub currently_subscribed: bool,
    /// State of the subscription
    pub subscriber_details: Option<Subscriber>,
}

/// Query response for [`SubscriptionQueryMsg::Subscribers`]
#[cosmwasm_schema::cw_serde]
pub struct SubscribersResponse {
    /// list of subscribers
    pub subscribers: Vec<(Addr, SubscriberResponse)>,
}

/// Hook message that contains list of just unsubscribed users
#[cosmwasm_schema::cw_serde]
pub struct UnsubscribedHookMsg {
    /// Unsubscribed users
    pub unsubscribed: Vec<String>,
}

// This is just a helper to properly serialize the Hook message
#[cosmwasm_schema::cw_serde]
pub(crate) enum HookReceiverExecuteMsg {
    Unsubscribed(UnsubscribedHookMsg),
}

impl UnsubscribedHookMsg {
    /// serializes the message
    pub fn into_json_binary(self) -> StdResult<Binary> {
        let msg = HookReceiverExecuteMsg::Unsubscribed(self);
        to_json_binary(&msg)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg<T: Into<String>>(self, contract_addr: T) -> StdResult<CosmosMsg> {
        let msg = self.into_json_binary()?;
        let execute = WasmMsg::Execute {
            contract_addr: contract_addr.into(),
            msg,
            funds: vec![],
        };
        Ok(execute.into())
    }
}
