//! # AuthZ
//! This module provides functionality to interact with the authz module of CosmosSDK Chains.
//! It allows for granting authorizations to perform actions on behalf of an account to other accounts.

use cosmos_sdk_proto::cosmos::{bank, base, staking};
use cosmos_sdk_proto::{cosmos::authz, traits::Message, Any};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::features::AccountIdentification;
use crate::AbstractSdkResult;

/// An interface to the CosmosSDK AuthZ module which allows for granting authorizations to perform actions on behalf of one account to other accounts.
pub trait AuthZInterface: AccountIdentification {
    /// API for accessing the Cosmos SDK AuthZ module.
    /// The **granter** is the address of the user **granting** an authorization to perform an action on their behalf.
    /// By default, it is the proxy address of the Account.

    /// ```
    /// use abstract_sdk::prelude::*;
    /// # use cosmwasm_std::testing::mock_dependencies;
    /// # use abstract_sdk::mock_module::MockModule;
    /// # let module = MockModule::new();
    /// # let deps = mock_dependencies();

    /// let authz: AuthZ = module.auth_z(deps.as_ref(), None)?;
    /// ```
    fn auth_z<'a>(
        &'a self,
        deps: cosmwasm_std::Deps<'a>,
        granter: Option<Addr>,
    ) -> AbstractSdkResult<AuthZ> {
        let granter = granter.unwrap_or(self.proxy_address(deps)?);
        Ok(AuthZ { granter })
    }
}

impl<T> AuthZInterface for T where T: AccountIdentification {}

/// This struct provides methods to grant message authorizations and interact with the authz module.
///
/// # Example
/// ```
/// use abstract_sdk::prelude::*;
/// # use cosmwasm_std::testing::mock_dependencies;
/// # use abstract_sdk::mock_module::MockModule;
/// # let module = MockModule::new();
///
/// let authz: Authz  = module.auth_z(deps.as_ref(), None)?;
/// ```
/// */
#[derive(Clone)]
pub struct AuthZ {
    granter: Addr,
}

impl AuthZ {
    /// Retrieve the granter's address.
    /// By default, this is the proxy address of the Account.
    fn granter(&self) -> Addr {
        self.granter.clone()
    }

    /// Removes msg type authorization from the granter to the **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `type_url` - The msg type url to revoke authorization.
    pub fn revoke(&self, grantee: &Addr, type_url: String) -> CosmosMsg {
        let msg = authz::v1beta1::MsgRevoke {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            msg_type_url: type_url,
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            // TODO: `Name` implementation is missing for authz
            // type_url: authz::v1beta1::MsgRevoke::type_url(),
            type_url: "/cosmos.authz.v1beta1.MsgRevoke".to_string(),
            value: Binary(msg),
        }
    }

    fn grant(
        &self,
        grantee: &Addr,
        expiration: Option<Timestamp>,
        authorization: impl AuthZAuthorization,
    ) -> CosmosMsg {
        let msg = authz::v1beta1::MsgGrant {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            grant: Some(authorization.grant(expiration)),
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary(msg),
        }
    }

    /// Grants generic authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `msg` - Allowed message type url. These are protobuf URLs defined in the Cosmos SDK.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_generic(
        &self,
        grantee: &Addr,
        msg_type_url: String,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let generic = GenericAuthorization::new(msg_type_url);

        self.grant(grantee, expiration, generic)
    }

    /// Grants send authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `spend_limits` - The maximum amount the grantee can spend.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_send(
        &self,
        grantee: &Addr,
        spend_limit: Vec<Coin>,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let send = SendAuthorization::new(spend_limit);

        self.grant(grantee, expiration, send)
    }

    /// Grants stake authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `max_tokens` - The maximum amount the grantee can stake. Empty means any amount of coins can be delegated.
    /// * `authorization_type` - The allowed delegate type.
    /// * `validators` - The list of validators to allow or deny.
    /// * `expiration` - The expiration timestamp of the grant.
    pub fn grant_stake(
        &self,
        grantee: &Addr,
        max_tokens: Option<Coin>,
        authorization_type: AuthorizationType,
        validators: Option<Policy>,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let stake = StakeAuthorization::new(max_tokens, authorization_type, validators);

        self.grant(grantee, expiration, stake)
    }
}

fn convert_stamp(stamp: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
    }
}

fn convert_coins(coins: Vec<Coin>) -> Vec<base::v1beta1::Coin> {
    coins.into_iter().map(convert_coin).collect()
}

fn convert_coin(coin: Coin) -> base::v1beta1::Coin {
    base::v1beta1::Coin {
        denom: coin.denom,
        amount: coin.amount.to_string(),
    }
}

/// Represents a generic authorization grant.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct GenericAuthorization {
    /// Allowed msg type_url
    pub msg_type_url: String,
}

impl GenericAuthorization {
    /// Create new generic authorization
    pub fn new(msg_type_url: String) -> Self {
        Self { msg_type_url }
    }
}

/// Represents send authorization grant
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
    /// see [staking::v1beta1::AuthorizationType::Unspecified]
    Unspecified,
    /// see [staking::v1beta1::AuthorizationType::Delegate]
    Delegate,
    /// see [staking::v1beta1::AuthorizationType::Undelegate]
    Undelegate,
    /// see [staking::v1beta1::AuthorizationType::Redelegate]
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
    /// see [staking::v1beta1::stake_authorization::Policy::AllowList]
    AllowList(Vec<Addr>),
    /// see [staking::v1beta1::stake_authorization::Policy::DenyList]
    DenyList(Vec<Addr>),
}

impl From<Policy> for staking::v1beta1::stake_authorization::Policy {
    fn from(value: Policy) -> Self {
        use staking::v1beta1::stake_authorization::Policy as StPolicy;
        use staking::v1beta1::stake_authorization::Validators;
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
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct StakeAuthorization {
    /// see [staking::v1beta1::StakeAuthorization::max_tokens]
    pub max_tokens: Option<Coin>,
    /// see [staking::v1beta1::StakeAuthorization::authorization_type]
    pub authorization_type: AuthorizationType,
    /// see [staking::v1beta1::StakeAuthorization::validators]
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
trait AuthZAuthorization {
    type ProtoType: Message;

    const TYPE_URL: &'static str;

    /// Get `Any`
    fn to_any(&self) -> Any {
        Any {
            type_url: Self::TYPE_URL.to_owned(),
            value: self.to_proto().encode_to_vec(),
        }
    }

    /// Get `Self::ProtoType`
    fn to_proto(&self) -> Self::ProtoType;

    fn grant(&self, expiration: Option<Timestamp>) -> authz::v1beta1::Grant {
        authz::v1beta1::Grant {
            authorization: Some(self.to_any()),
            expiration: expiration.map(convert_stamp),
        }
    }
}

impl AuthZAuthorization for GenericAuthorization {
    type ProtoType = authz::v1beta1::GenericAuthorization;

    const TYPE_URL: &'static str = "/cosmos.authz.v1beta1.GenericAuthorization";

    fn to_proto(&self) -> Self::ProtoType {
        authz::v1beta1::GenericAuthorization {
            msg: self.msg_type_url.clone(),
        }
    }
}

impl AuthZAuthorization for SendAuthorization {
    type ProtoType = bank::v1beta1::SendAuthorization;

    const TYPE_URL: &'static str = "/cosmos.bank.v1beta1.SendAuthorization";

    fn to_proto(&self) -> Self::ProtoType {
        bank::v1beta1::SendAuthorization {
            spend_limit: convert_coins(self.spend_limit.clone()),
        }
    }
}

impl AuthZAuthorization for StakeAuthorization {
    type ProtoType = staking::v1beta1::StakeAuthorization;

    const TYPE_URL: &'static str = "/cosmos.staking.v1beta.StakeAuthorization";

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_module::*;

    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn generic_authorization() {
        let app = MockModule::new();
        let deps = mock_dependencies();

        let granter = Addr::unchecked("granter");
        let grantee = Addr::unchecked("grantee");

        let auth_z = app.auth_z(deps.as_ref(), Some(granter.clone())).unwrap();
        let expiration = Some(Timestamp::from_seconds(10));

        let generic_authorization_msg = auth_z.grant_generic(
            &grantee,
            "/cosmos.gov.v1beta1.MsgVote".to_string(),
            expiration,
        );

        let expected_msg = CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary(
                authz::v1beta1::MsgGrant {
                    granter: granter.into_string(),
                    grantee: grantee.into_string(),
                    grant: Some(authz::v1beta1::Grant {
                        authorization: Some(Any {
                            type_url: "/cosmos.authz.v1beta1.GenericAuthorization".to_string(),
                            value: authz::v1beta1::GenericAuthorization {
                                msg: "/cosmos.gov.v1beta1.MsgVote".to_string(),
                            }
                            .encode_to_vec(),
                        }),
                        expiration: expiration.map(convert_stamp),
                    }),
                }
                .encode_to_vec(),
            ),
        };

        assert_eq!(generic_authorization_msg, expected_msg);
    }

    #[test]
    fn revoke_authorization() {
        let app = MockModule::new();
        let deps = mock_dependencies();

        let granter = Addr::unchecked("granter");
        let grantee = Addr::unchecked("grantee");

        let auth_z = app.auth_z(deps.as_ref(), Some(granter.clone())).unwrap();

        let generic_authorization_msg =
            auth_z.revoke(&grantee, "/cosmos.gov.v1beta1.MsgVote".to_string());

        let expected_msg = CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgRevoke".to_string(),
            value: Binary(
                authz::v1beta1::MsgRevoke {
                    granter: granter.into_string(),
                    grantee: grantee.into_string(),
                    msg_type_url: "/cosmos.gov.v1beta1.MsgVote".to_string(),
                }
                .encode_to_vec(),
            ),
        };

        assert_eq!(generic_authorization_msg, expected_msg);
    }
}
