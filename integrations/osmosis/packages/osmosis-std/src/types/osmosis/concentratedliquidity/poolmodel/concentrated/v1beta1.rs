use osmosis_std_derive::CosmwasmExt;
/// ===================== MsgCreateConcentratedPool
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
#[proto_message(
    type_url = "/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPool"
)]
pub struct MsgCreateConcentratedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub denom1: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_spacing: u64,
    #[prost(string, tag = "5")]
    pub spread_factor: ::prost::alloc::string::String,
}
/// Returns a unique poolID to identify the pool with.
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
#[proto_message(
    type_url = "/osmosis.concentratedliquidity.poolmodel.concentrated.v1beta1.MsgCreateConcentratedPoolResponse"
)]
pub struct MsgCreateConcentratedPoolResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
}
