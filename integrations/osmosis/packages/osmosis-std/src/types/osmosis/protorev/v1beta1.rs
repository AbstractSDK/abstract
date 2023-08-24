use osmosis_std_derive::CosmwasmExt;
/// TokenPairArbRoutes tracks all of the hot routes for a given pair of tokens
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.TokenPairArbRoutes")]
pub struct TokenPairArbRoutes {
    /// Stores all of the possible hot paths for a given pair of tokens
    #[prost(message, repeated, tag = "1")]
    pub arb_routes: ::prost::alloc::vec::Vec<Route>,
    /// Token denomination of the first asset
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    /// Token denomination of the second asset
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
/// Route is a hot route for a given pair of tokens
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.Route")]
pub struct Route {
    /// The pool IDs that are travered in the directed cyclic graph (traversed left
    /// -> right)
    #[prost(message, repeated, tag = "1")]
    pub trades: ::prost::alloc::vec::Vec<Trade>,
    /// The step size that will be used to find the optimal swap amount in the
    /// binary search
    #[prost(string, tag = "2")]
    pub step_size: ::prost::alloc::string::String,
}
/// Trade is a single trade in a route
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.Trade")]
pub struct Trade {
    /// The pool id of the pool that is traded on
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool: u64,
    /// The denom of the token that is traded
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    /// The denom of the token that is received
    #[prost(string, tag = "3")]
    pub token_out: ::prost::alloc::string::String,
}
/// RouteStatistics contains the number of trades the module has executed after a
/// swap on a given route and the profits from the trades
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.RouteStatistics")]
pub struct RouteStatistics {
    /// profits is the total profit from all trades on this route
    #[prost(message, repeated, tag = "1")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// number_of_trades is the number of trades the module has executed using this
    /// route
    #[prost(string, tag = "2")]
    pub number_of_trades: ::prost::alloc::string::String,
    /// route is the route that was used (pool ids along the arbitrage route)
    #[prost(uint64, repeated, packed = "false", tag = "3")]
    pub route: ::prost::alloc::vec::Vec<u64>,
}
/// PoolWeights contains the weights of all of the different pool types. This
/// distinction is made and necessary because the execution time ranges
/// significantly between the different pool types. Each weight roughly
/// corresponds to the amount of time (in ms) it takes to execute a swap on that
/// pool type.
///
/// DEPRECATED: This field is deprecated and will be removed in the next
/// release. It is replaced by the `info_by_pool_type` field.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.PoolWeights")]
pub struct PoolWeights {
    /// The weight of a stableswap pool
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub stable_weight: u64,
    /// The weight of a balancer pool
    #[prost(uint64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub balancer_weight: u64,
    /// The weight of a concentrated pool
    #[prost(uint64, tag = "3")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub concentrated_weight: u64,
    /// The weight of a cosmwasm pool
    #[prost(uint64, tag = "4")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub cosmwasm_weight: u64,
}
/// InfoByPoolType contains information pertaining to how expensive (in terms of
/// gas and time) it is to execute a swap on a given pool type. This distinction
/// is made and necessary because the execution time ranges significantly between
/// the different pool types.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.InfoByPoolType")]
pub struct InfoByPoolType {
    /// The stable pool info
    #[prost(message, optional, tag = "1")]
    pub stable: ::core::option::Option<StablePoolInfo>,
    /// The balancer pool info
    #[prost(message, optional, tag = "2")]
    pub balancer: ::core::option::Option<BalancerPoolInfo>,
    /// The concentrated pool info
    #[prost(message, optional, tag = "3")]
    pub concentrated: ::core::option::Option<ConcentratedPoolInfo>,
    /// The cosmwasm pool info
    #[prost(message, optional, tag = "4")]
    pub cosmwasm: ::core::option::Option<CosmwasmPoolInfo>,
}
/// StablePoolInfo contains meta data pertaining to a stableswap pool type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.StablePoolInfo")]
pub struct StablePoolInfo {
    /// The weight of a stableswap pool
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub weight: u64,
}
/// BalancerPoolInfo contains meta data pertaining to a balancer pool type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.BalancerPoolInfo")]
pub struct BalancerPoolInfo {
    /// The weight of a balancer pool
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub weight: u64,
}
/// ConcentratedPoolInfo contains meta data pertaining to a concentrated pool
/// type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.ConcentratedPoolInfo")]
pub struct ConcentratedPoolInfo {
    /// The weight of a concentrated pool
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub weight: u64,
    /// The maximum number of ticks we can move when rebalancing
    #[prost(uint64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_ticks_crossed: u64,
}
/// CosmwasmPoolInfo contains meta data pertaining to a cosmwasm pool type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.CosmwasmPoolInfo")]
pub struct CosmwasmPoolInfo {
    /// The weight of a cosmwasm pool (by contract address)
    #[prost(message, repeated, tag = "1")]
    pub weight_maps: ::prost::alloc::vec::Vec<WeightMap>,
}
/// WeightMap maps a contract address to a weight. The weight of an address
/// corresponds to the amount of ms required to execute a swap on that contract.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.WeightMap")]
pub struct WeightMap {
    /// The weight of a cosmwasm pool (by contract address)
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub weight: u64,
    /// The contract address
    #[prost(string, tag = "2")]
    pub contract_address: ::prost::alloc::string::String,
}
/// BaseDenom represents a single base denom that the module uses for its
/// arbitrage trades. It contains the denom name alongside the step size of the
/// binary search that is used to find the optimal swap amount
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.BaseDenom")]
pub struct BaseDenom {
    /// The denom i.e. name of the base denom (ex. uosmo)
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// The step size of the binary search that is used to find the optimal swap
    /// amount
    #[prost(string, tag = "2")]
    pub step_size: ::prost::alloc::string::String,
}
/// Params defines the parameters for the module.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.Params")]
pub struct Params {
    /// Boolean whether the protorev module is enabled.
    #[prost(bool, tag = "1")]
    pub enabled: bool,
    /// The admin account (settings manager) of the protorev module.
    #[prost(string, tag = "2")]
    pub admin: ::prost::alloc::string::String,
}
/// GenesisState defines the protorev module's genesis state.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.GenesisState")]
pub struct GenesisState {
    /// Parameters for the protorev module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// Token pair arb routes for the protorev module (hot routes).
    #[prost(message, repeated, tag = "2")]
    pub token_pair_arb_routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
    /// The base denominations being used to create cyclic arbitrage routes via the
    /// highest liquidity method.
    #[prost(message, repeated, tag = "3")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
    /// The pool weights that are being used to calculate the weight (compute cost)
    /// of each route.
    ///
    /// DEPRECATED: This field is deprecated and will be removed in the next
    /// release. It is replaced by the `info_by_pool_type` field.
    #[prost(message, optional, tag = "4")]
    pub pool_weights: ::core::option::Option<PoolWeights>,
    /// The number of days since module genesis.
    #[prost(uint64, tag = "5")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub days_since_module_genesis: u64,
    /// The fees the developer account has accumulated over time.
    #[prost(message, repeated, tag = "6")]
    pub developer_fees: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// The latest block height that the module has processed.
    #[prost(uint64, tag = "7")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub latest_block_height: u64,
    /// The developer account address of the module.
    #[prost(string, tag = "8")]
    pub developer_address: ::prost::alloc::string::String,
    /// Max pool points per block i.e. the maximum compute time (in ms)
    /// that protorev can use per block.
    #[prost(uint64, tag = "9")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_block: u64,
    /// Max pool points per tx i.e. the maximum compute time (in ms) that
    /// protorev can use per tx.
    #[prost(uint64, tag = "10")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_tx: u64,
    /// The number of pool points that have been consumed in the current block.
    #[prost(uint64, tag = "11")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub point_count_for_block: u64,
    /// All of the profits that have been accumulated by the module.
    #[prost(message, repeated, tag = "12")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    /// Information that is used to estimate execution time / gas
    /// consumption of a swap on a given pool type.
    #[prost(message, optional, tag = "13")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
}
/// SetProtoRevEnabledProposal is a gov Content type to update whether the
/// protorev module is enabled
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.SetProtoRevEnabledProposal")]
pub struct SetProtoRevEnabledProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(bool, tag = "3")]
    pub enabled: bool,
}
/// SetProtoRevAdminAccountProposal is a gov Content type to set the admin
/// account that will receive permissions to alter hot routes and set the
/// developer address that will be receiving a share of profits from the module
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.SetProtoRevAdminAccountProposal")]
pub struct SetProtoRevAdminAccountProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account: ::prost::alloc::string::String,
}
/// QueryParamsRequest is request type for the Query/Params RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryParamsRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/Params",
    response_type = QueryParamsResponse
)]
pub struct QueryParamsRequest {}
/// QueryParamsResponse is response type for the Query/Params RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryParamsResponse")]
pub struct QueryParamsResponse {
    /// params holds all the parameters of this module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
}
/// QueryGetProtoRevNumberOfTradesRequest is request type for the
/// Query/GetProtoRevNumberOfTrades RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevNumberOfTradesRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevNumberOfTrades",
    response_type = QueryGetProtoRevNumberOfTradesResponse
)]
pub struct QueryGetProtoRevNumberOfTradesRequest {}
/// QueryGetProtoRevNumberOfTradesResponse is response type for the
/// Query/GetProtoRevNumberOfTrades RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevNumberOfTradesResponse")]
pub struct QueryGetProtoRevNumberOfTradesResponse {
    /// number_of_trades is the number of trades the module has executed
    #[prost(string, tag = "1")]
    pub number_of_trades: ::prost::alloc::string::String,
}
/// QueryGetProtoRevProfitsByDenomRequest is request type for the
/// Query/GetProtoRevProfitsByDenom RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevProfitsByDenomRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevProfitsByDenom",
    response_type = QueryGetProtoRevProfitsByDenomResponse
)]
pub struct QueryGetProtoRevProfitsByDenomRequest {
    /// denom is the denom to query profits by
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
}
/// QueryGetProtoRevProfitsByDenomResponse is response type for the
/// Query/GetProtoRevProfitsByDenom RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevProfitsByDenomResponse")]
pub struct QueryGetProtoRevProfitsByDenomResponse {
    /// profit is the profits of the module by the selected denom
    #[prost(message, optional, tag = "1")]
    pub profit: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// QueryGetProtoRevAllProfitsRequest is request type for the
/// Query/GetProtoRevAllProfits RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAllProfitsRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevAllProfits",
    response_type = QueryGetProtoRevAllProfitsResponse
)]
pub struct QueryGetProtoRevAllProfitsRequest {}
/// QueryGetProtoRevAllProfitsResponse is response type for the
/// Query/GetProtoRevAllProfits RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAllProfitsResponse")]
pub struct QueryGetProtoRevAllProfitsResponse {
    /// profits is a list of all of the profits from the module
    #[prost(message, repeated, tag = "1")]
    pub profits: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// QueryGetProtoRevStatisticsByPoolRequest is request type for the
/// Query/GetProtoRevStatisticsByRoute RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevStatisticsByRouteRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevStatisticsByRoute",
    response_type = QueryGetProtoRevStatisticsByRouteResponse
)]
pub struct QueryGetProtoRevStatisticsByRouteRequest {
    /// route is the set of pool ids to query statistics by i.e. 1,2,3
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    pub route: ::prost::alloc::vec::Vec<u64>,
}
/// QueryGetProtoRevStatisticsByRouteResponse is response type for the
/// Query/GetProtoRevStatisticsByRoute RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevStatisticsByRouteResponse")]
pub struct QueryGetProtoRevStatisticsByRouteResponse {
    /// statistics contains the number of trades the module has executed after a
    /// swap on a given pool and the profits from the trades
    #[prost(message, optional, tag = "1")]
    pub statistics: ::core::option::Option<RouteStatistics>,
}
/// QueryGetProtoRevAllRouteStatisticsRequest is request type for the
/// Query/GetProtoRevAllRouteStatistics RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAllRouteStatisticsRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevAllRouteStatistics",
    response_type = QueryGetProtoRevAllRouteStatisticsResponse
)]
pub struct QueryGetProtoRevAllRouteStatisticsRequest {}
/// QueryGetProtoRevAllRouteStatisticsResponse is response type for the
/// Query/GetProtoRevAllRouteStatistics RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAllRouteStatisticsResponse")]
pub struct QueryGetProtoRevAllRouteStatisticsResponse {
    /// statistics contains the number of trades/profits the module has executed on
    /// all routes it has successfully executed a trade on
    #[prost(message, repeated, tag = "1")]
    pub statistics: ::prost::alloc::vec::Vec<RouteStatistics>,
}
/// QueryGetProtoRevTokenPairArbRoutesRequest is request type for the
/// Query/GetProtoRevTokenPairArbRoutes RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevTokenPairArbRoutesRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevTokenPairArbRoutes",
    response_type = QueryGetProtoRevTokenPairArbRoutesResponse
)]
pub struct QueryGetProtoRevTokenPairArbRoutesRequest {}
/// QueryGetProtoRevTokenPairArbRoutesResponse is response type for the
/// Query/GetProtoRevTokenPairArbRoutes RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevTokenPairArbRoutesResponse")]
pub struct QueryGetProtoRevTokenPairArbRoutesResponse {
    /// routes is a list of all of the hot routes that the module is currently
    /// arbitraging
    #[prost(message, repeated, tag = "1")]
    pub routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
}
/// QueryGetProtoRevAdminAccountRequest is request type for the
/// Query/GetProtoRevAdminAccount RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAdminAccountRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevAdminAccount",
    response_type = QueryGetProtoRevAdminAccountResponse
)]
pub struct QueryGetProtoRevAdminAccountRequest {}
/// QueryGetProtoRevAdminAccountResponse is response type for the
/// Query/GetProtoRevAdminAccount RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevAdminAccountResponse")]
pub struct QueryGetProtoRevAdminAccountResponse {
    /// admin_account is the admin account of the module
    #[prost(string, tag = "1")]
    pub admin_account: ::prost::alloc::string::String,
}
/// QueryGetProtoRevDeveloperAccountRequest is request type for the
/// Query/GetProtoRevDeveloperAccount RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevDeveloperAccountRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevDeveloperAccount",
    response_type = QueryGetProtoRevDeveloperAccountResponse
)]
pub struct QueryGetProtoRevDeveloperAccountRequest {}
/// QueryGetProtoRevDeveloperAccountResponse is response type for the
/// Query/GetProtoRevDeveloperAccount RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevDeveloperAccountResponse")]
pub struct QueryGetProtoRevDeveloperAccountResponse {
    /// developer_account is the developer account of the module
    #[prost(string, tag = "1")]
    pub developer_account: ::prost::alloc::string::String,
}
/// QueryGetProtoRevInfoByPoolTypeRequest is request type for the
/// Query/GetProtoRevInfoByPoolType RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevInfoByPoolTypeRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevInfoByPoolType",
    response_type = QueryGetProtoRevInfoByPoolTypeResponse
)]
pub struct QueryGetProtoRevInfoByPoolTypeRequest {}
/// QueryGetProtoRevInfoByPoolTypeResponse is response type for the
/// Query/GetProtoRevInfoByPoolType RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevInfoByPoolTypeResponse")]
pub struct QueryGetProtoRevInfoByPoolTypeResponse {
    /// InfoByPoolType contains all information pertaining to how different
    /// pool types are handled by the module.
    #[prost(message, optional, tag = "1")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
}
/// QueryGetProtoRevMaxPoolPointsPerBlockRequest is request type for the
/// Query/GetProtoRevMaxPoolPointsPerBlock RPC method.
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
    type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevMaxPoolPointsPerBlockRequest"
)]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevMaxPoolPointsPerBlock",
    response_type = QueryGetProtoRevMaxPoolPointsPerBlockResponse
)]
pub struct QueryGetProtoRevMaxPoolPointsPerBlockRequest {}
/// QueryGetProtoRevMaxPoolPointsPerBlockResponse is response type for the
/// Query/GetProtoRevMaxPoolPointsPerBlock RPC method.
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
    type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevMaxPoolPointsPerBlockResponse"
)]
pub struct QueryGetProtoRevMaxPoolPointsPerBlockResponse {
    /// max_pool_points_per_block is the maximum number of pool points that can be
    /// consumed per block
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_block: u64,
}
/// QueryGetProtoRevMaxPoolPointsPerTxRequest is request type for the
/// Query/GetProtoRevMaxPoolPointsPerTx RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevMaxPoolPointsPerTxRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevMaxPoolPointsPerTx",
    response_type = QueryGetProtoRevMaxPoolPointsPerTxResponse
)]
pub struct QueryGetProtoRevMaxPoolPointsPerTxRequest {}
/// QueryGetProtoRevMaxPoolPointsPerTxResponse is response type for the
/// Query/GetProtoRevMaxPoolPointsPerTx RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevMaxPoolPointsPerTxResponse")]
pub struct QueryGetProtoRevMaxPoolPointsPerTxResponse {
    /// max_pool_points_per_tx is the maximum number of pool points that can be
    /// consumed per transaction
    #[prost(uint64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_tx: u64,
}
/// QueryGetProtoRevBaseDenomsRequest is request type for the
/// Query/GetProtoRevBaseDenoms RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevBaseDenomsRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevBaseDenoms",
    response_type = QueryGetProtoRevBaseDenomsResponse
)]
pub struct QueryGetProtoRevBaseDenomsRequest {}
/// QueryGetProtoRevBaseDenomsResponse is response type for the
/// Query/GetProtoRevBaseDenoms RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevBaseDenomsResponse")]
pub struct QueryGetProtoRevBaseDenomsResponse {
    /// base_denoms is a list of all of the base denoms and step sizes
    #[prost(message, repeated, tag = "1")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
}
/// QueryGetProtoRevEnabledRequest is request type for the
/// Query/GetProtoRevEnabled RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevEnabledRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevEnabled",
    response_type = QueryGetProtoRevEnabledResponse
)]
pub struct QueryGetProtoRevEnabledRequest {}
/// QueryGetProtoRevEnabledResponse is response type for the
/// Query/GetProtoRevEnabled RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevEnabledResponse")]
pub struct QueryGetProtoRevEnabledResponse {
    /// enabled is whether the module is enabled
    #[prost(bool, tag = "1")]
    pub enabled: bool,
}
/// QueryGetProtoRevPoolRequest is request type for the
/// Query/GetProtoRevPool RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevPoolRequest")]
#[proto_query(
    path = "/osmosis.protorev.v1beta1.Query/GetProtoRevPool",
    response_type = QueryGetProtoRevPoolResponse
)]
pub struct QueryGetProtoRevPoolRequest {
    /// base_denom is the base denom set in protorev for the denom pair to pool
    /// mapping
    #[prost(string, tag = "1")]
    pub base_denom: ::prost::alloc::string::String,
    /// other_denom is the other denom for the denom pair to pool mapping
    #[prost(string, tag = "2")]
    pub other_denom: ::prost::alloc::string::String,
}
/// QueryGetProtoRevPoolResponse is response type for the
/// Query/GetProtoRevPool RPC method.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.QueryGetProtoRevPoolResponse")]
pub struct QueryGetProtoRevPoolResponse {
    /// pool_id is the pool_id stored for the denom pair
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
}
/// MsgSetHotRoutes defines the Msg/SetHotRoutes request type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetHotRoutes")]
pub struct MsgSetHotRoutes {
    /// admin is the account that is authorized to set the hot routes.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// hot_routes is the list of hot routes to set.
    #[prost(message, repeated, tag = "2")]
    pub hot_routes: ::prost::alloc::vec::Vec<TokenPairArbRoutes>,
}
/// MsgSetHotRoutesResponse defines the Msg/SetHotRoutes response type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetHotRoutesResponse")]
pub struct MsgSetHotRoutesResponse {}
/// MsgSetDeveloperAccount defines the Msg/SetDeveloperAccount request type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetDeveloperAccount")]
pub struct MsgSetDeveloperAccount {
    /// admin is the account that is authorized to set the developer account.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// developer_account is the account that will receive a portion of the profits
    /// from the protorev module.
    #[prost(string, tag = "2")]
    pub developer_account: ::prost::alloc::string::String,
}
/// MsgSetDeveloperAccountResponse defines the Msg/SetDeveloperAccount response
/// type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetDeveloperAccountResponse")]
pub struct MsgSetDeveloperAccountResponse {}
/// MsgSetInfoByPoolType defines the Msg/SetInfoByPoolType request type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetInfoByPoolType")]
pub struct MsgSetInfoByPoolType {
    /// admin is the account that is authorized to set the pool weights.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// info_by_pool_type contains information about the pool types.
    #[prost(message, optional, tag = "2")]
    pub info_by_pool_type: ::core::option::Option<InfoByPoolType>,
}
/// MsgSetInfoByPoolTypeResponse defines the Msg/SetInfoByPoolType response type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetInfoByPoolTypeResponse")]
pub struct MsgSetInfoByPoolTypeResponse {}
/// MsgSetMaxPoolPointsPerTx defines the Msg/SetMaxPoolPointsPerTx request type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetMaxPoolPointsPerTx")]
pub struct MsgSetMaxPoolPointsPerTx {
    /// admin is the account that is authorized to set the max pool points per tx.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// max_pool_points_per_tx is the maximum number of pool points that can be
    /// consumed per transaction.
    #[prost(uint64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_tx: u64,
}
/// MsgSetMaxPoolPointsPerTxResponse defines the Msg/SetMaxPoolPointsPerTx
/// response type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetMaxPoolPointsPerTxResponse")]
pub struct MsgSetMaxPoolPointsPerTxResponse {}
/// MsgSetMaxPoolPointsPerBlock defines the Msg/SetMaxPoolPointsPerBlock request
/// type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetMaxPoolPointsPerBlock")]
pub struct MsgSetMaxPoolPointsPerBlock {
    /// admin is the account that is authorized to set the max pool points per
    /// block.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// max_pool_points_per_block is the maximum number of pool points that can be
    /// consumed per block.
    #[prost(uint64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub max_pool_points_per_block: u64,
}
/// MsgSetMaxPoolPointsPerBlockResponse defines the
/// Msg/SetMaxPoolPointsPerBlock response type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetMaxPoolPointsPerBlockResponse")]
pub struct MsgSetMaxPoolPointsPerBlockResponse {}
/// MsgSetBaseDenoms defines the Msg/SetBaseDenoms request type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetBaseDenoms")]
pub struct MsgSetBaseDenoms {
    /// admin is the account that is authorized to set the base denoms.
    #[prost(string, tag = "1")]
    pub admin: ::prost::alloc::string::String,
    /// base_denoms is the list of base denoms to set.
    #[prost(message, repeated, tag = "2")]
    pub base_denoms: ::prost::alloc::vec::Vec<BaseDenom>,
}
/// MsgSetBaseDenomsResponse defines the Msg/SetBaseDenoms response type.
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
#[proto_message(type_url = "/osmosis.protorev.v1beta1.MsgSetBaseDenomsResponse")]
pub struct MsgSetBaseDenomsResponse {}
pub struct ProtorevQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> ProtorevQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn params(&self) -> Result<QueryParamsResponse, cosmwasm_std::StdError> {
        QueryParamsRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_number_of_trades(
        &self,
    ) -> Result<QueryGetProtoRevNumberOfTradesResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevNumberOfTradesRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_profits_by_denom(
        &self,
        denom: ::prost::alloc::string::String,
    ) -> Result<QueryGetProtoRevProfitsByDenomResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevProfitsByDenomRequest { denom }.query(self.querier)
    }
    pub fn get_proto_rev_all_profits(
        &self,
    ) -> Result<QueryGetProtoRevAllProfitsResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevAllProfitsRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_statistics_by_route(
        &self,
        route: ::prost::alloc::vec::Vec<u64>,
    ) -> Result<QueryGetProtoRevStatisticsByRouteResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevStatisticsByRouteRequest { route }.query(self.querier)
    }
    pub fn get_proto_rev_all_route_statistics(
        &self,
    ) -> Result<QueryGetProtoRevAllRouteStatisticsResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevAllRouteStatisticsRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_token_pair_arb_routes(
        &self,
    ) -> Result<QueryGetProtoRevTokenPairArbRoutesResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevTokenPairArbRoutesRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_admin_account(
        &self,
    ) -> Result<QueryGetProtoRevAdminAccountResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevAdminAccountRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_developer_account(
        &self,
    ) -> Result<QueryGetProtoRevDeveloperAccountResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevDeveloperAccountRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_info_by_pool_type(
        &self,
    ) -> Result<QueryGetProtoRevInfoByPoolTypeResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevInfoByPoolTypeRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_max_pool_points_per_tx(
        &self,
    ) -> Result<QueryGetProtoRevMaxPoolPointsPerTxResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevMaxPoolPointsPerTxRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_max_pool_points_per_block(
        &self,
    ) -> Result<QueryGetProtoRevMaxPoolPointsPerBlockResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevMaxPoolPointsPerBlockRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_base_denoms(
        &self,
    ) -> Result<QueryGetProtoRevBaseDenomsResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevBaseDenomsRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_enabled(
        &self,
    ) -> Result<QueryGetProtoRevEnabledResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevEnabledRequest {}.query(self.querier)
    }
    pub fn get_proto_rev_pool(
        &self,
        base_denom: ::prost::alloc::string::String,
        other_denom: ::prost::alloc::string::String,
    ) -> Result<QueryGetProtoRevPoolResponse, cosmwasm_std::StdError> {
        QueryGetProtoRevPoolRequest {
            base_denom,
            other_denom,
        }
        .query(self.querier)
    }
}
