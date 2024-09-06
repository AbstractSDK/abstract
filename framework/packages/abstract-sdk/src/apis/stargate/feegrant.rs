// use std::time::Duration;

use cosmos_sdk_proto::{
    cosmos::{base, feegrant},
    traits::Name,
};
use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::StargateMessage;

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
    pub period: Option<std::time::Duration>,
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
        period: Option<std::time::Duration>,
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

/// Trait for types that can be used as allowances in the FeeGranter.
pub trait BasicOrPeriodicAllowance: MsgAllowance {}

impl BasicOrPeriodicAllowance for BasicAllowance {}
impl BasicOrPeriodicAllowance for PeriodicAllowance {}

/// Trait for types that can be used as feegrant type
pub trait MsgAllowance: StargateMessage {}

impl MsgAllowance for BasicAllowance {}
impl MsgAllowance for PeriodicAllowance {}
impl<A: BasicOrPeriodicAllowance> MsgAllowance for AllowedMsgAllowance<A> {}

// Stargate Msg implementations

impl StargateMessage for BasicAllowance {
    type ProtoType = feegrant::v1beta1::BasicAllowance;

    fn type_url() -> String {
        Self::ProtoType::type_url()
    }

    fn to_proto(&self) -> feegrant::v1beta1::BasicAllowance {
        feegrant::v1beta1::BasicAllowance {
            spend_limit: self
                .spend_limit
                .iter()
                .map(|item| base::v1beta1::Coin {
                    denom: item.denom.clone(),
                    amount: item.amount.to_string(),
                })
                .collect(),
            expiration: self.expiration.map(super::convert_stamp),
        }
    }
}

impl StargateMessage for PeriodicAllowance {
    type ProtoType = feegrant::v1beta1::PeriodicAllowance;

    fn type_url() -> String {
        Self::ProtoType::type_url()
    }

    fn to_proto(&self) -> feegrant::v1beta1::PeriodicAllowance {
        feegrant::v1beta1::PeriodicAllowance {
            basic: self.basic.clone().map(|b| b.to_proto()),
            period: self.period.map(
                |p| cosmos_sdk_proto::tendermint::google::protobuf::Duration {
                    seconds: p.as_secs() as i64,
                    nanos: 0,
                },
            ),
            period_spend_limit: super::convert_coins(self.period_spend_limit.clone()),
            period_can_spend: super::convert_coins(self.period_can_spend.clone()),
            period_reset: self.period_reset.map(super::convert_stamp),
        }
    }
}

impl<A: BasicOrPeriodicAllowance> StargateMessage for AllowedMsgAllowance<A> {
    type ProtoType = feegrant::v1beta1::AllowedMsgAllowance;

    fn type_url() -> String {
        Self::ProtoType::type_url()
    }

    fn to_proto(&self) -> feegrant::v1beta1::AllowedMsgAllowance {
        feegrant::v1beta1::AllowedMsgAllowance {
            allowance: self.allowance.as_ref().map(|a| a.to_any()),
            allowed_messages: self.allowed_messages.clone(),
        }
    }
}
