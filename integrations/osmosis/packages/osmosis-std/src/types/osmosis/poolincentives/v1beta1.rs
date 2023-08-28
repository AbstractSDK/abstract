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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.Params")]
pub struct Params {
    /// minted_denom is the denomination of the coin expected to be minted by the
    /// minting module. Pool-incentives module doesn’t actually mint the coin
    /// itself, but rather manages the distribution of coins that matches the
    /// defined minted_denom.
    #[prost(string, tag = "1")]
    pub minted_denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.LockableDurationsInfo")]
pub struct LockableDurationsInfo {
    #[prost(message, repeated, tag = "1")]
    pub lockable_durations: ::prost::alloc::vec::Vec<crate::shim::Duration>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.DistrInfo")]
pub struct DistrInfo {
    #[prost(string, tag = "1")]
    pub total_weight: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub records: ::prost::alloc::vec::Vec<DistrRecord>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.DistrRecord")]
pub struct DistrRecord {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "gaugeID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub gauge_id: u64,
    #[prost(string, tag = "2")]
    pub weight: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.PoolToGauge")]
pub struct PoolToGauge {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "gaugeID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub gauge_id: u64,
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.PoolToGauges")]
pub struct PoolToGauges {
    #[prost(message, repeated, tag = "2")]
    pub pool_to_gauge: ::prost::alloc::vec::Vec<PoolToGauge>,
}
/// GenesisState defines the pool incentives module's genesis state.
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.GenesisState")]
pub struct GenesisState {
    /// params defines all the paramaters of the module.
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    #[prost(message, repeated, tag = "2")]
    pub lockable_durations: ::prost::alloc::vec::Vec<crate::shim::Duration>,
    #[prost(message, optional, tag = "3")]
    pub distr_info: ::core::option::Option<DistrInfo>,
    #[prost(message, optional, tag = "4")]
    pub pool_to_gauges: ::core::option::Option<PoolToGauges>,
}
/// ReplacePoolIncentivesProposal is a gov Content type for updating the pool
/// incentives. If a ReplacePoolIncentivesProposal passes, the proposal’s records
/// override the existing DistrRecords set in the module. Each record has a
/// specified gauge id and weight, and the incentives are distributed to each
/// gauge according to weight/total_weight. The incentives are put in the fee
/// pool and it is allocated to gauges and community pool by the DistrRecords
/// configuration. Note that gaugeId=0 represents the community pool.
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.ReplacePoolIncentivesProposal")]
pub struct ReplacePoolIncentivesProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub records: ::prost::alloc::vec::Vec<DistrRecord>,
}
/// For example: if the existing DistrRecords were:
/// [(Gauge 0, 5), (Gauge 1, 6), (Gauge 2, 6)]
/// An UpdatePoolIncentivesProposal includes
/// [(Gauge 1, 0), (Gauge 2, 4), (Gauge 3, 10)]
/// This would delete Gauge 1, Edit Gauge 2, and Add Gauge 3
/// The result DistrRecords in state would be:
/// [(Gauge 0, 5), (Gauge 2, 4), (Gauge 3, 10)]
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.UpdatePoolIncentivesProposal")]
pub struct UpdatePoolIncentivesProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub records: ::prost::alloc::vec::Vec<DistrRecord>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryGaugeIdsRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/GaugeIds",
    response_type = QueryGaugeIdsResponse
)]
pub struct QueryGaugeIdsRequest {
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryGaugeIdsResponse")]
pub struct QueryGaugeIdsResponse {
    #[prost(message, repeated, tag = "1")]
    #[serde(alias = "gaugeIDs_with_duration")]
    pub gauge_ids_with_duration:
        ::prost::alloc::vec::Vec<query_gauge_ids_response::GaugeIdWithDuration>,
}
/// Nested message and enum types in `QueryGaugeIdsResponse`.
pub mod query_gauge_ids_response {
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
    #[proto_message(
        type_url = "/osmosis.poolincentives.v1beta1.QueryGaugeIdsResponse.GaugeIdWithDuration"
    )]
    pub struct GaugeIdWithDuration {
        #[prost(uint64, tag = "1")]
        #[serde(alias = "gaugeID")]
        #[serde(
            serialize_with = "crate::serde::as_str::serialize",
            deserialize_with = "crate::serde::as_str::deserialize"
        )]
        pub gauge_id: u64,
        #[prost(message, optional, tag = "2")]
        pub duration: ::core::option::Option<crate::shim::Duration>,
        #[prost(string, tag = "3")]
        pub gauge_incentive_percentage: ::prost::alloc::string::String,
    }
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryDistrInfoRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/DistrInfo",
    response_type = QueryDistrInfoResponse
)]
pub struct QueryDistrInfoRequest {}
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryDistrInfoResponse")]
pub struct QueryDistrInfoResponse {
    #[prost(message, optional, tag = "1")]
    pub distr_info: ::core::option::Option<DistrInfo>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryParamsRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/Params",
    response_type = QueryParamsResponse
)]
pub struct QueryParamsRequest {}
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryParamsResponse")]
pub struct QueryParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryLockableDurationsRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/LockableDurations",
    response_type = QueryLockableDurationsResponse
)]
pub struct QueryLockableDurationsRequest {}
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryLockableDurationsResponse")]
pub struct QueryLockableDurationsResponse {
    #[prost(message, repeated, tag = "1")]
    pub lockable_durations: ::prost::alloc::vec::Vec<crate::shim::Duration>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryIncentivizedPoolsRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/IncentivizedPools",
    response_type = QueryIncentivizedPoolsResponse
)]
pub struct QueryIncentivizedPoolsRequest {}
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.IncentivizedPool")]
pub struct IncentivizedPool {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(message, optional, tag = "2")]
    pub lockable_duration: ::core::option::Option<crate::shim::Duration>,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "gaugeID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub gauge_id: u64,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryIncentivizedPoolsResponse")]
pub struct QueryIncentivizedPoolsResponse {
    #[prost(message, repeated, tag = "1")]
    pub incentivized_pools: ::prost::alloc::vec::Vec<IncentivizedPool>,
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryExternalIncentiveGaugesRequest")]
#[proto_query(
    path = "/osmosis.poolincentives.v1beta1.Query/ExternalIncentiveGauges",
    response_type = QueryExternalIncentiveGaugesResponse
)]
pub struct QueryExternalIncentiveGaugesRequest {}
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.QueryExternalIncentiveGaugesResponse")]
pub struct QueryExternalIncentiveGaugesResponse {
    #[prost(message, repeated, tag = "1")]
    pub data: ::prost::alloc::vec::Vec<super::super::incentives::Gauge>,
}
/// MigrationRecords contains all the links between balancer and concentrated
/// pools.
///
/// This is copied over from the gamm proto file in order to circumnavigate
/// the circular dependency between the two modules.
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.MigrationRecords")]
pub struct MigrationRecords {
    #[prost(message, repeated, tag = "1")]
    pub balancer_to_concentrated_pool_links:
        ::prost::alloc::vec::Vec<BalancerToConcentratedPoolLink>,
}
/// BalancerToConcentratedPoolLink defines a single link between a single
/// balancer pool and a single concentrated liquidity pool. This link is used to
/// allow a balancer pool to migrate to a single canonical full range
/// concentrated liquidity pool position
/// A balancer pool can be linked to a maximum of one cl pool, and a cl pool can
/// be linked to a maximum of one balancer pool.
///
/// This is copied over from the gamm proto file in order to circumnavigate
/// the circular dependency between the two modules.
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
#[proto_message(type_url = "/osmosis.poolincentives.v1beta1.BalancerToConcentratedPoolLink")]
pub struct BalancerToConcentratedPoolLink {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "balancer_poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub balancer_pool_id: u64,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "cl_poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub cl_pool_id: u64,
}
pub struct PoolincentivesQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> PoolincentivesQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn gauge_ids(&self, pool_id: u64) -> Result<QueryGaugeIdsResponse, cosmwasm_std::StdError> {
        QueryGaugeIdsRequest { pool_id }.query(self.querier)
    }
    pub fn distr_info(&self) -> Result<QueryDistrInfoResponse, cosmwasm_std::StdError> {
        QueryDistrInfoRequest {}.query(self.querier)
    }
    pub fn params(&self) -> Result<QueryParamsResponse, cosmwasm_std::StdError> {
        QueryParamsRequest {}.query(self.querier)
    }
    pub fn lockable_durations(
        &self,
    ) -> Result<QueryLockableDurationsResponse, cosmwasm_std::StdError> {
        QueryLockableDurationsRequest {}.query(self.querier)
    }
    pub fn incentivized_pools(
        &self,
    ) -> Result<QueryIncentivizedPoolsResponse, cosmwasm_std::StdError> {
        QueryIncentivizedPoolsRequest {}.query(self.querier)
    }
    pub fn external_incentive_gauges(
        &self,
    ) -> Result<QueryExternalIncentiveGaugesResponse, cosmwasm_std::StdError> {
        QueryExternalIncentiveGaugesRequest {}.query(self.querier)
    }
}
