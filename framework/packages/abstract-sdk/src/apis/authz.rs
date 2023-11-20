//! # AuthZ
//! This module provides functionality to interact with the authz module of CosmosSDK Chains.
//! It allows for granting authorizations to perform actions on behalf of an account to other accounts.

use cosmos_sdk_proto::{cosmos::authz, traits::Message, Any};
use cosmwasm_std::{Addr, Binary, CosmosMsg, Timestamp};
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

    /// Grants generic authorization to a **grantee**.
    ///
    /// # Arguments
    ///
    /// * `grantee` - The address of the grantee.
    /// * `msg` - Allowed message type url. These are protobuf URLs defined in the Cosmos SDK.
    pub fn grant_generic(
        &self,
        grantee: &Addr,
        msg_type_url: String,
        expiration: Option<Timestamp>,
    ) -> CosmosMsg {
        let generic = GenericAuthorization::new(msg_type_url);

        let msg = authz::v1beta1::MsgGrant {
            granter: self.granter().to_string(),
            grantee: grantee.to_string(),
            grant: Some(generic.grant(expiration)),
        }
        .encode_to_vec();

        CosmosMsg::Stargate {
            type_url: "/cosmos.authz.v1beta1.MsgGrant".to_string(),
            value: Binary(msg),
        }
    }
}

fn convert_stamp(stamp: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: stamp.seconds() as i64,
        nanos: stamp.nanos() as i32,
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
