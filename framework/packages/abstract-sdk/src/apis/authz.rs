//! # AuthZ
//! This module provides functionality to interact with the authz module of CosmosSDK Chains.
//! It allows for granting authorizations to perform actions on behalf of an account to other accounts.

use cosmos_sdk_proto::{cosmos::authz, traits::Message};
use cosmwasm_std::{Addr, Binary, Coin, CosmosMsg, Timestamp};

use super::stargate::authz::{
    AuthZAuthorization, AuthorizationType, GenericAuthorization, Policy, SendAuthorization,
    StakeAuthorization,
};
use crate::{features::AccountIdentification, AbstractSdkResult};

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

    /// Generate cosmwasm message for the AuthZAuthorization type
    pub fn grant_authorization<A: AuthZAuthorization>(
        &self,
        grantee: &Addr,
        expiration: Option<Timestamp>,
        authorization: A,
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

        self.grant_authorization(grantee, expiration, generic)
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

        self.grant_authorization(grantee, expiration, send)
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

        self.grant_authorization(grantee, expiration, stake)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::mock_dependencies;
    use prost_types::Any;

    use super::*;
    use crate::{apis::stargate::convert_stamp, mock_module::*};

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
