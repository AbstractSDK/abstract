use osmosis_std_derive::CosmwasmExt;
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SwapAmountInRoute")]
pub struct SwapAmountInRoute {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_out_denom: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SwapAmountOutRoute")]
pub struct SwapAmountOutRoute {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in_denom: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SwapAmountInSplitRoute")]
pub struct SwapAmountInSplitRoute {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
    #[prost(string, tag = "2")]
    pub token_in_amount: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SwapAmountOutSplitRoute")]
pub struct SwapAmountOutSplitRoute {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "2")]
    pub token_out_amount: ::prost::alloc::string::String,
}
/// ModuleRouter defines a route encapsulating pool type.
/// It is used as the value of a mapping from pool id to the pool type,
/// allowing the pool manager to know which module to route swaps to given the
/// pool id.
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.ModuleRoute")]
pub struct ModuleRoute {
    /// pool_type specifies the type of the pool
    #[prost(enumeration = "PoolType", tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_type: i32,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
}
/// PoolType is an enumeration of all supported pool types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
pub enum PoolType {
    /// Balancer is the standard xy=k curve. Its pool model is defined in x/gamm.
    Balancer = 0,
    /// Stableswap is the Solidly cfmm stable swap curve. Its pool model is defined
    /// in x/gamm.
    Stableswap = 1,
    /// Concentrated is the pool model specific to concentrated liquidity. It is
    /// defined in x/concentrated-liquidity.
    Concentrated = 2,
    /// CosmWasm is the pool model specific to CosmWasm. It is defined in
    /// x/cosmwasmpool.
    CosmWasm = 3,
}
impl PoolType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            PoolType::Balancer => "Balancer",
            PoolType::Stableswap => "Stableswap",
            PoolType::Concentrated => "Concentrated",
            PoolType::CosmWasm => "CosmWasm",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Balancer" => Some(Self::Balancer),
            "Stableswap" => Some(Self::Stableswap),
            "Concentrated" => Some(Self::Concentrated),
            "CosmWasm" => Some(Self::CosmWasm),
            _ => None,
        }
    }
}
/// Params holds parameters for the poolmanager module
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.Params")]
pub struct Params {
    #[prost(message, repeated, tag = "1")]
    pub pool_creation_fee:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// GenesisState defines the poolmanager module's genesis state.
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.GenesisState")]
pub struct GenesisState {
    /// the next_pool_id
    #[prost(uint64, tag = "1")]
    #[serde(alias = "next_poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub next_pool_id: u64,
    /// params is the container of poolmanager parameters.
    #[prost(message, optional, tag = "2")]
    pub params: ::core::option::Option<Params>,
    /// pool_routes is the container of the mappings from pool id to pool type.
    #[prost(message, repeated, tag = "3")]
    pub pool_routes: ::prost::alloc::vec::Vec<ModuleRoute>,
}
/// ===================== MsgSwapExactAmountIn
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSwapExactAmountIn")]
pub struct MsgSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
    #[prost(message, optional, tag = "3")]
    pub token_in: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "4")]
    pub token_out_min_amount: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSwapExactAmountInResponse")]
pub struct MsgSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
/// ===================== MsgSplitRouteSwapExactAmountIn
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSplitRouteSwapExactAmountIn")]
pub struct MsgSplitRouteSwapExactAmountIn {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInSplitRoute>,
    #[prost(string, tag = "3")]
    pub token_in_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub token_out_min_amount: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSplitRouteSwapExactAmountInResponse")]
pub struct MsgSplitRouteSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
/// ===================== MsgSwapExactAmountOut
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSwapExactAmountOut")]
pub struct MsgSwapExactAmountOut {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "3")]
    pub token_in_max_amount: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub token_out: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSwapExactAmountOutResponse")]
pub struct MsgSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
/// ===================== MsgSplitRouteSwapExactAmountOut
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSplitRouteSwapExactAmountOut")]
pub struct MsgSplitRouteSwapExactAmountOut {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutSplitRoute>,
    #[prost(string, tag = "3")]
    pub token_out_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub token_in_max_amount: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.MsgSplitRouteSwapExactAmountOutResponse")]
pub struct MsgSplitRouteSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
/// =============================== Params
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.ParamsRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/Params",
    response_type = ParamsResponse
)]
pub struct ParamsRequest {}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.ParamsResponse")]
pub struct ParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// =============================== EstimateSwapExactAmountIn
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountInRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountIn",
    response_type = EstimateSwapExactAmountInResponse
)]
pub struct EstimateSwapExactAmountInRequest {
    #[prost(uint64, tag = "2")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "3")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "4")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
}
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
    type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountInWithPrimitiveTypesRequest"
)]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountInWithPrimitiveTypes",
    response_type = EstimateSwapExactAmountInResponse
)]
pub struct EstimateSwapExactAmountInWithPrimitiveTypesRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    #[serde(alias = "routes_poolID")]
    pub routes_pool_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, repeated, tag = "4")]
    pub routes_token_out_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
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
    type_url = "/osmosis.poolmanager.v1beta1.EstimateSinglePoolSwapExactAmountInRequest"
)]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSinglePoolSwapExactAmountIn",
    response_type = EstimateSwapExactAmountInResponse
)]
pub struct EstimateSinglePoolSwapExactAmountInRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub token_out_denom: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountInResponse")]
pub struct EstimateSwapExactAmountInResponse {
    #[prost(string, tag = "1")]
    pub token_out_amount: ::prost::alloc::string::String,
}
/// =============================== EstimateSwapExactAmountOut
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountOutRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountOut",
    response_type = EstimateSwapExactAmountOutResponse
)]
pub struct EstimateSwapExactAmountOutRequest {
    #[prost(uint64, tag = "2")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(message, repeated, tag = "3")]
    pub routes: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
    #[prost(string, tag = "4")]
    pub token_out: ::prost::alloc::string::String,
}
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
    type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountOutWithPrimitiveTypesRequest"
)]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSwapExactAmountOutWithPrimitiveTypes",
    response_type = EstimateSwapExactAmountOutResponse
)]
pub struct EstimateSwapExactAmountOutWithPrimitiveTypesRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(uint64, repeated, packed = "false", tag = "2")]
    #[serde(alias = "routes_poolID")]
    pub routes_pool_id: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, repeated, tag = "3")]
    pub routes_token_in_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag = "4")]
    pub token_out: ::prost::alloc::string::String,
}
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
    type_url = "/osmosis.poolmanager.v1beta1.EstimateSinglePoolSwapExactAmountOutRequest"
)]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/EstimateSinglePoolSwapExactAmountOut",
    response_type = EstimateSwapExactAmountOutResponse
)]
pub struct EstimateSinglePoolSwapExactAmountOutRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.EstimateSwapExactAmountOutResponse")]
pub struct EstimateSwapExactAmountOutResponse {
    #[prost(string, tag = "1")]
    pub token_in_amount: ::prost::alloc::string::String,
}
/// =============================== NumPools
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.NumPoolsRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/NumPools",
    response_type = NumPoolsResponse
)]
pub struct NumPoolsRequest {}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.NumPoolsResponse")]
pub struct NumPoolsResponse {
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub num_pools: u64,
}
/// =============================== Pool
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.PoolRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/Pool",
    response_type = PoolResponse
)]
pub struct PoolRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.PoolResponse")]
pub struct PoolResponse {
    #[prost(message, optional, tag = "1")]
    pub pool: ::core::option::Option<crate::shim::Any>,
}
/// =============================== AllPools
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.AllPoolsRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/AllPools",
    response_type = AllPoolsResponse
)]
pub struct AllPoolsRequest {}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.AllPoolsResponse")]
pub struct AllPoolsResponse {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<crate::shim::Any>,
}
/// SpotPriceRequest defines the gRPC request structure for a SpotPrice
/// query.
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SpotPriceRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/SpotPrice",
    response_type = SpotPriceResponse
)]
pub struct SpotPriceRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub base_asset_denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub quote_asset_denom: ::prost::alloc::string::String,
}
/// SpotPriceResponse defines the gRPC response structure for a SpotPrice
/// query.
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.SpotPriceResponse")]
pub struct SpotPriceResponse {
    /// String of the Dec. Ex) 10.203uatom
    #[prost(string, tag = "1")]
    pub spot_price: ::prost::alloc::string::String,
}
/// =============================== TotalPoolLiquidity
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.TotalPoolLiquidityRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/TotalPoolLiquidity",
    response_type = TotalPoolLiquidityResponse
)]
pub struct TotalPoolLiquidityRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.TotalPoolLiquidityResponse")]
pub struct TotalPoolLiquidityResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// =============================== TotalLiquidity
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.TotalLiquidityRequest")]
#[proto_query(
    path = "/osmosis.poolmanager.v1beta1.Query/TotalLiquidity",
    response_type = TotalLiquidityResponse
)]
pub struct TotalLiquidityRequest {}
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
#[proto_message(type_url = "/osmosis.poolmanager.v1beta1.TotalLiquidityResponse")]
pub struct TotalLiquidityResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
pub struct PoolmanagerQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> PoolmanagerQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn params(&self) -> Result<ParamsResponse, cosmwasm_std::StdError> {
        ParamsRequest {}.query(self.querier)
    }
    pub fn estimate_swap_exact_amount_in(
        &self,
        pool_id: u64,
        token_in: ::prost::alloc::string::String,
        routes: ::prost::alloc::vec::Vec<SwapAmountInRoute>,
    ) -> Result<EstimateSwapExactAmountInResponse, cosmwasm_std::StdError> {
        EstimateSwapExactAmountInRequest {
            pool_id,
            token_in,
            routes,
        }
        .query(self.querier)
    }
    pub fn estimate_swap_exact_amount_in_with_primitive_types(
        &self,
        pool_id: u64,
        token_in: ::prost::alloc::string::String,
        routes_pool_id: ::prost::alloc::vec::Vec<u64>,
        routes_token_out_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    ) -> Result<EstimateSwapExactAmountInResponse, cosmwasm_std::StdError> {
        EstimateSwapExactAmountInWithPrimitiveTypesRequest {
            pool_id,
            token_in,
            routes_pool_id,
            routes_token_out_denom,
        }
        .query(self.querier)
    }
    pub fn estimate_single_pool_swap_exact_amount_in(
        &self,
        pool_id: u64,
        token_in: ::prost::alloc::string::String,
        token_out_denom: ::prost::alloc::string::String,
    ) -> Result<EstimateSwapExactAmountInResponse, cosmwasm_std::StdError> {
        EstimateSinglePoolSwapExactAmountInRequest {
            pool_id,
            token_in,
            token_out_denom,
        }
        .query(self.querier)
    }
    pub fn estimate_swap_exact_amount_out(
        &self,
        pool_id: u64,
        routes: ::prost::alloc::vec::Vec<SwapAmountOutRoute>,
        token_out: ::prost::alloc::string::String,
    ) -> Result<EstimateSwapExactAmountOutResponse, cosmwasm_std::StdError> {
        EstimateSwapExactAmountOutRequest {
            pool_id,
            routes,
            token_out,
        }
        .query(self.querier)
    }
    pub fn estimate_swap_exact_amount_out_with_primitive_types(
        &self,
        pool_id: u64,
        routes_pool_id: ::prost::alloc::vec::Vec<u64>,
        routes_token_in_denom: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        token_out: ::prost::alloc::string::String,
    ) -> Result<EstimateSwapExactAmountOutResponse, cosmwasm_std::StdError> {
        EstimateSwapExactAmountOutWithPrimitiveTypesRequest {
            pool_id,
            routes_pool_id,
            routes_token_in_denom,
            token_out,
        }
        .query(self.querier)
    }
    pub fn estimate_single_pool_swap_exact_amount_out(
        &self,
        pool_id: u64,
        token_in_denom: ::prost::alloc::string::String,
        token_out: ::prost::alloc::string::String,
    ) -> Result<EstimateSwapExactAmountOutResponse, cosmwasm_std::StdError> {
        EstimateSinglePoolSwapExactAmountOutRequest {
            pool_id,
            token_in_denom,
            token_out,
        }
        .query(self.querier)
    }
    pub fn num_pools(&self) -> Result<NumPoolsResponse, cosmwasm_std::StdError> {
        NumPoolsRequest {}.query(self.querier)
    }
    pub fn pool(&self, pool_id: u64) -> Result<PoolResponse, cosmwasm_std::StdError> {
        PoolRequest { pool_id }.query(self.querier)
    }
    pub fn all_pools(&self) -> Result<AllPoolsResponse, cosmwasm_std::StdError> {
        AllPoolsRequest {}.query(self.querier)
    }
    pub fn spot_price(
        &self,
        pool_id: u64,
        base_asset_denom: ::prost::alloc::string::String,
        quote_asset_denom: ::prost::alloc::string::String,
    ) -> Result<SpotPriceResponse, cosmwasm_std::StdError> {
        SpotPriceRequest {
            pool_id,
            base_asset_denom,
            quote_asset_denom,
        }
        .query(self.querier)
    }
    pub fn total_pool_liquidity(
        &self,
        pool_id: u64,
    ) -> Result<TotalPoolLiquidityResponse, cosmwasm_std::StdError> {
        TotalPoolLiquidityRequest { pool_id }.query(self.querier)
    }
    pub fn total_liquidity(&self) -> Result<TotalLiquidityResponse, cosmwasm_std::StdError> {
        TotalLiquidityRequest {}.query(self.querier)
    }
}
