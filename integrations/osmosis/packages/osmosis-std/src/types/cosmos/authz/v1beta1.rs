use osmosis_std_derive::CosmwasmExt;
/// GenericAuthorization gives the grantee unrestricted permissions to execute
/// the provided method on behalf of the granter's account.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.GenericAuthorization")]
pub struct GenericAuthorization {
    /// Msg, identified by it's type URL, to grant unrestricted permissions to execute
    #[prost(string, tag = "1")]
    pub msg: ::prost::alloc::string::String,
}
/// Grant gives permissions to execute
/// the provide method with expiration time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.Grant")]
pub struct Grant {
    #[prost(message, optional, tag = "1")]
    pub authorization: ::core::option::Option<crate::shim::Any>,
    #[prost(message, optional, tag = "2")]
    pub expiration: ::core::option::Option<crate::shim::Timestamp>,
}
/// GrantAuthorization extends a grant with both the addresses of the grantee and granter.
/// It is used in genesis.proto and query.proto
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.GrantAuthorization")]
pub struct GrantAuthorization {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub authorization: ::core::option::Option<crate::shim::Any>,
    #[prost(message, optional, tag = "4")]
    pub expiration: ::core::option::Option<crate::shim::Timestamp>,
}
/// EventGrant is emitted on Msg/Grant
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.EventGrant")]
pub struct EventGrant {
    /// Msg type URL for which an autorization is granted
    #[prost(string, tag = "2")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// Granter account address
    #[prost(string, tag = "3")]
    pub granter: ::prost::alloc::string::String,
    /// Grantee account address
    #[prost(string, tag = "4")]
    pub grantee: ::prost::alloc::string::String,
}
/// EventRevoke is emitted on Msg/Revoke
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.EventRevoke")]
pub struct EventRevoke {
    /// Msg type URL for which an autorization is revoked
    #[prost(string, tag = "2")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// Granter account address
    #[prost(string, tag = "3")]
    pub granter: ::prost::alloc::string::String,
    /// Grantee account address
    #[prost(string, tag = "4")]
    pub grantee: ::prost::alloc::string::String,
}
/// GenesisState defines the authz module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.GenesisState")]
pub struct GenesisState {
    #[prost(message, repeated, tag = "1")]
    pub authorization: ::prost::alloc::vec::Vec<GrantAuthorization>,
}
/// QueryGrantsRequest is the request type for the Query/Grants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGrantsRequest")]
#[proto_query(
    path = "/cosmos.authz.v1beta1.Query/Grants",
    response_type = QueryGrantsResponse
)]
pub struct QueryGrantsRequest {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    /// Optional, msg_type_url, when set, will query only grants matching given msg type.
    #[prost(string, tag = "3")]
    pub msg_type_url: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "4")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
/// QueryGrantsResponse is the response type for the Query/Authorizations RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGrantsResponse")]
pub struct QueryGrantsResponse {
    /// authorizations is a list of grants granted for grantee by granter.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<Grant>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
/// QueryGranterGrantsRequest is the request type for the Query/GranterGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGranterGrantsRequest")]
#[proto_query(
    path = "/cosmos.authz.v1beta1.Query/GranterGrants",
    response_type = QueryGranterGrantsResponse
)]
pub struct QueryGranterGrantsRequest {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
/// QueryGranterGrantsResponse is the response type for the Query/GranterGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGranterGrantsResponse")]
pub struct QueryGranterGrantsResponse {
    /// grants is a list of grants granted by the granter.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<GrantAuthorization>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
/// QueryGranteeGrantsRequest is the request type for the Query/IssuedGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGranteeGrantsRequest")]
#[proto_query(
    path = "/cosmos.authz.v1beta1.Query/GranteeGrants",
    response_type = QueryGranteeGrantsResponse
)]
pub struct QueryGranteeGrantsRequest {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// pagination defines an pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
}
/// QueryGranteeGrantsResponse is the response type for the Query/GranteeGrants RPC method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.QueryGranteeGrantsResponse")]
pub struct QueryGranteeGrantsResponse {
    /// grants is a list of grants granted to the grantee.
    #[prost(message, repeated, tag = "1")]
    pub grants: ::prost::alloc::vec::Vec<GrantAuthorization>,
    /// pagination defines an pagination for the response.
    #[prost(message, optional, tag = "2")]
    pub pagination: ::core::option::Option<super::super::base::query::v1beta1::PageResponse>,
}
/// MsgGrant is a request type for Grant method. It declares authorization to the grantee
/// on behalf of the granter with the provided expiration time.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgGrant")]
pub struct MsgGrant {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub grant: ::core::option::Option<Grant>,
}
/// MsgExecResponse defines the Msg/MsgExecResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgExecResponse")]
pub struct MsgExecResponse {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub results: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
/// MsgExec attempts to execute the provided messages using
/// authorizations granted to the grantee. Each message should have only
/// one signer corresponding to the granter of the authorization.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgExec")]
pub struct MsgExec {
    #[prost(string, tag = "1")]
    pub grantee: ::prost::alloc::string::String,
    /// Authorization Msg requests to execute. Each msg must implement Authorization interface
    /// The x/authz will try to find a grant matching (msg.signers\[0\], grantee, MsgTypeURL(msg))
    /// triple and validate it.
    #[prost(message, repeated, tag = "2")]
    pub msgs: ::prost::alloc::vec::Vec<crate::shim::Any>,
}
/// MsgGrantResponse defines the Msg/MsgGrant response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgGrantResponse")]
pub struct MsgGrantResponse {}
/// MsgRevoke revokes any authorization with the provided sdk.Msg type on the
/// granter's account with that has been granted to the grantee.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgRevoke")]
pub struct MsgRevoke {
    #[prost(string, tag = "1")]
    pub granter: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub grantee: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub msg_type_url: ::prost::alloc::string::String,
}
/// MsgRevokeResponse defines the Msg/MsgRevokeResponse response type.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/cosmos.authz.v1beta1.MsgRevokeResponse")]
pub struct MsgRevokeResponse {}
pub struct AuthzQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> AuthzQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn grants(
        &self,
        granter: ::prost::alloc::string::String,
        grantee: ::prost::alloc::string::String,
        msg_type_url: ::prost::alloc::string::String,
        pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
    ) -> Result<QueryGrantsResponse, cosmwasm_std::StdError> {
        QueryGrantsRequest {
            granter,
            grantee,
            msg_type_url,
            pagination,
        }
        .query(self.querier)
    }
    pub fn granter_grants(
        &self,
        granter: ::prost::alloc::string::String,
        pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
    ) -> Result<QueryGranterGrantsResponse, cosmwasm_std::StdError> {
        QueryGranterGrantsRequest {
            granter,
            pagination,
        }
        .query(self.querier)
    }
    pub fn grantee_grants(
        &self,
        grantee: ::prost::alloc::string::String,
        pagination: ::core::option::Option<super::super::base::query::v1beta1::PageRequest>,
    ) -> Result<QueryGranteeGrantsResponse, cosmwasm_std::StdError> {
        QueryGranteeGrantsRequest {
            grantee,
            pagination,
        }
        .query(self.querier)
    }
}
