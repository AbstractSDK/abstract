use cosmos_sdk_proto::traits::{Message, Name};
use cosmwasm_std::{Coin, Timestamp};
use prost_types::Any;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(crate) mod utils;

mod feegrant_impls;

/// # Fee Granter
/// This module provides functionality to interact with the feegrant module of Cosmos.
/// It allows for granting fee expenditure rights to other accounts.
pub mod feegrant {
    use super::*;

    pub use super::feegrant_impls::*;

    /// Represents a basic fee allowance grant.
    /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance]
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
    pub struct BasicAllowance {
        /// Maximum amount of tokens that can be spent
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance::spend_limit]
        pub spend_limit: Vec<Coin>,
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::BasicAllowance::expiration]
        pub expiration: Option<Timestamp>,
    }

    impl BasicAllowance {
        /// Create new basic allowance
        pub fn new(spend_limit: Vec<Coin>, expiration: Option<Timestamp>) -> Self {
            Self {
                spend_limit,
                expiration,
            }
        }
    }

    /// Details for a periodic fee allowance grant
    /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance]
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
    pub struct PeriodicAllowance {
        /// basic is the instance of [BasicAllowance] which is optional for periodic fee allowance. If empty, the grant will have no expiration and no spend_limit.
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance::basic]
        pub basic: Option<BasicAllowance>,
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance::period]
        pub period: Option<Duration>,
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance::period_spend_limit]
        pub period_spend_limit: Vec<Coin>,
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance::period_spend_limit]
        pub period_can_spend: Vec<Coin>,
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::PeriodicAllowance::period_reset]
        pub period_reset: Option<Timestamp>,
    }

    impl PeriodicAllowance {
        /// Create new periodic allowance
        pub fn new(
            basic: Option<BasicAllowance>,
            period: Option<Duration>,
            period_spend_limit: Vec<Coin>,
            period_can_spend: Vec<Coin>,
            period_reset: Option<Timestamp>,
        ) -> Self {
            Self {
                basic,
                period,
                period_spend_limit,
                period_can_spend,
                period_reset,
            }
        }
    }

    /// Allowance and list of allowed messages
    /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::AllowedMsgAllowance]
    pub struct AllowedMsgAllowance<A: BasicOrPeriodicAllowance> {
        /// [BasicAllowance] or [PeriodicAllowance]
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::AllowedMsgAllowance::allowance]
        pub allowance: Option<A>,
        /// List of msg_type that allowed
        /// @see [cosmos_sdk_proto::cosmos::feegrant::v1beta1::AllowedMsgAllowance::allowed_messages]
        pub allowed_messages: Vec<String>,
    }

    impl<A: BasicOrPeriodicAllowance> AllowedMsgAllowance<A> {
        /// Create new allowed messages allowance
        pub fn new(allowance: Option<A>, allowed_messages: Vec<String>) -> Self {
            Self {
                allowance,
                allowed_messages,
            }
        }
    }
}

/// This trait allows generate `Any` and proto message from Stargate API message
pub trait StargateMessage {
    /// Returned proto type
    type ProtoType: Message + Name + Sized;

    /// Get `Any`
    fn to_any(&self) -> Any {
        Any {
            type_url: Self::ProtoType::type_url(),
            value: self.to_proto().encode_to_vec(),
        }
    }

    /// Get `Self::ProtoType`
    fn to_proto(&self) -> Self::ProtoType;
}
