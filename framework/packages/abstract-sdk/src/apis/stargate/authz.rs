use cosmos_sdk_proto::cosmos::{authz, bank, staking};
use cosmwasm_std::{Addr, Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{convert_coin, convert_coins, convert_stamp, StargateMessage};

/// Represents a generic authorization grant.
/// @see [authz::v1beta1::GenericAuthorization]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct GenericAuthorization {
    /// Allowed msg type_url
    /// @see [authz::v1beta1::GenericAuthorization::msg]
    pub msg_type_url: String,
}

impl GenericAuthorization {
    /// Create new generic authorization
    pub fn new(msg_type_url: String) -> Self {
        Self { msg_type_url }
    }
}

/// Represents send authorization grant
/// @see [bank::v1beta1::SendAuthorization]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct SendAuthorization {
    /// Allowed spend limit
    pub spend_limit: Vec<Coin>,
}

impl SendAuthorization {
    /// create new send authorization
    pub fn new(spend_limit: Vec<Coin>) -> Self {
        Self { spend_limit }
    }
}

/// (de)serializable representation of [AuthorizationType](staking::v1beta1::AuthorizationType)
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum AuthorizationType {
    /// @see [staking::v1beta1::AuthorizationType::Unspecified]
    Unspecified,
    /// @see [staking::v1beta1::AuthorizationType::Delegate]
    Delegate,
    /// @see [staking::v1beta1::AuthorizationType::Undelegate]
    Undelegate,
    /// @see [staking::v1beta1::AuthorizationType::Redelegate]
    Redelegate,
}

impl From<AuthorizationType> for staking::v1beta1::AuthorizationType {
    fn from(value: AuthorizationType) -> Self {
        use staking::v1beta1::AuthorizationType as StAuthT;
        match value {
            AuthorizationType::Unspecified => StAuthT::Unspecified,
            AuthorizationType::Delegate => StAuthT::Delegate,
            AuthorizationType::Undelegate => StAuthT::Undelegate,
            AuthorizationType::Redelegate => StAuthT::Redelegate,
        }
    }
}

/// (de)serializable representation of [Policy](staking::v1beta1::stake_authorization::Policy)
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub enum Policy {
    /// @see [staking::v1beta1::stake_authorization::Policy::AllowList]
    AllowList(Vec<Addr>),
    /// @see [staking::v1beta1::stake_authorization::Policy::DenyList]
    DenyList(Vec<Addr>),
}

impl From<Policy> for staking::v1beta1::stake_authorization::Policy {
    fn from(value: Policy) -> Self {
        use staking::v1beta1::stake_authorization::{Policy as StPolicy, Validators};
        match value {
            Policy::AllowList(allow_list) => StPolicy::AllowList(Validators {
                address: allow_list.into_iter().map(Addr::into_string).collect(),
            }),
            Policy::DenyList(deny_list) => StPolicy::DenyList(Validators {
                address: deny_list.into_iter().map(Addr::into_string).collect(),
            }),
        }
    }
}

/// Represents stake authorization grant
/// @see [staking::v1beta1::StakeAuthorization]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct StakeAuthorization {
    /// @see [staking::v1beta1::StakeAuthorization::max_tokens]
    pub max_tokens: Option<Coin>,
    /// @see [staking::v1beta1::StakeAuthorization::authorization_type]
    pub authorization_type: AuthorizationType,
    /// @see [staking::v1beta1::StakeAuthorization::validators]
    pub validators: Option<Policy>,
}

impl StakeAuthorization {
    /// create new send authorization
    pub fn new(
        max_tokens: Option<Coin>,
        authorization_type: AuthorizationType,
        validators: Option<Policy>,
    ) -> Self {
        Self {
            max_tokens,
            authorization_type,
            validators,
        }
    }
}

pub trait AuthZAuthorization: StargateMessage {
    fn grant(&self, expiration: Option<Timestamp>) -> authz::v1beta1::Grant {
        authz::v1beta1::Grant {
            authorization: Some(self.to_any()),
            expiration: expiration.map(convert_stamp),
        }
    }
}

impl AuthZAuthorization for GenericAuthorization {}
impl AuthZAuthorization for SendAuthorization {}
impl AuthZAuthorization for StakeAuthorization {}

impl StargateMessage for GenericAuthorization {
    type ProtoType = authz::v1beta1::GenericAuthorization;

    fn type_url() -> String {
        "/cosmos.authz.v1beta1.GenericAuthorization".to_owned()
    }

    fn to_proto(&self) -> Self::ProtoType {
        authz::v1beta1::GenericAuthorization {
            msg: self.msg_type_url.clone(),
        }
    }
}

impl StargateMessage for SendAuthorization {
    type ProtoType = bank::v1beta1::SendAuthorization;

    fn type_url() -> String {
        "/cosmos.bank.v1beta1.SendAuthorization".to_owned()
    }

    fn to_proto(&self) -> Self::ProtoType {
        bank::v1beta1::SendAuthorization {
            spend_limit: convert_coins(self.spend_limit.clone()),
            allow_list: vec![],
        }
    }
}

impl StargateMessage for StakeAuthorization {
    type ProtoType = staking::v1beta1::StakeAuthorization;

    fn type_url() -> String {
        "/cosmos.staking.v1beta.StakeAuthorization".to_owned()
    }

    fn to_proto(&self) -> Self::ProtoType {
        let authorization_type: staking::v1beta1::AuthorizationType =
            self.authorization_type.into();
        staking::v1beta1::StakeAuthorization {
            max_tokens: self.max_tokens.clone().map(convert_coin),
            authorization_type: authorization_type.into(),
            validators: self.validators.clone().map(Into::into),
        }
    }
}
