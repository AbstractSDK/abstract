use osmosis_std_derive::CosmwasmExt;
/// Position contains position's id, address, pool id, lower tick, upper tick
/// join time, and liquidity.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.Position")]
pub struct Position {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(int64, tag = "4")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lower_tick: i64,
    #[prost(int64, tag = "5")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub upper_tick: i64,
    #[prost(message, optional, tag = "6")]
    pub join_time: ::core::option::Option<crate::shim::Timestamp>,
    #[prost(string, tag = "7")]
    pub liquidity: ::prost::alloc::string::String,
}
/// FullPositionBreakdown returns:
/// - the position itself
/// - the amount the position translates in terms of asset0 and asset1
/// - the amount of claimable fees
/// - the amount of claimable incentives
/// - the amount of incentives that would be forfeited if the position was closed
/// now
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.FullPositionBreakdown")]
pub struct FullPositionBreakdown {
    #[prost(message, optional, tag = "1")]
    pub position: ::core::option::Option<Position>,
    #[prost(message, optional, tag = "2")]
    pub asset0: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "3")]
    pub asset1: ::core::option::Option<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "4")]
    pub claimable_spread_rewards:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "5")]
    pub claimable_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "6")]
    pub forfeited_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PositionWithPeriodLock")]
pub struct PositionWithPeriodLock {
    #[prost(message, optional, tag = "1")]
    pub position: ::core::option::Option<Position>,
    #[prost(message, optional, tag = "2")]
    pub locks: ::core::option::Option<super::super::lockup::PeriodLock>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.TickInfo")]
pub struct TickInfo {
    #[prost(string, tag = "1")]
    pub liquidity_gross: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub liquidity_net: ::prost::alloc::string::String,
    /// Total spread rewards accumulated in the opposite direction that the tick
    /// was last crossed. i.e. if the current tick is to the right of this tick
    /// (meaning its currently a greater price), then this is the total spread
    /// rewards accumulated below the tick. If the current tick is to the left of
    /// this tick (meaning its currently at a lower price), then this is the total
    /// spread rewards accumulated above the tick.
    ///
    /// Note: the way this value is used depends on the direction of spread rewards
    /// we are calculating for. If we are calculating spread rewards below the
    /// lower tick and the lower tick is the active tick, then this is the
    /// spreadRewardGrowthGlobal - the lower tick's
    /// spreadRewardGrowthOppositeDirectionOfLastTraversal. If we are calculating
    /// spread rewards above the upper tick and the upper tick is the active tick,
    /// then this is just the tick's
    /// spreadRewardGrowthOppositeDirectionOfLastTraversal value.
    #[prost(message, repeated, tag = "3")]
    pub spread_reward_growth_opposite_direction_of_last_traversal:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    /// uptime_trackers is a container encapsulating the uptime trackers.
    /// We use a container instead of a "repeated UptimeTracker" directly
    /// because we need the ability to serialize and deserialize the
    /// container easily for events when crossing a tick.
    #[prost(message, optional, tag = "4")]
    pub uptime_trackers: ::core::option::Option<UptimeTrackers>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UptimeTrackers")]
pub struct UptimeTrackers {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<UptimeTracker>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UptimeTracker")]
pub struct UptimeTracker {
    #[prost(message, repeated, tag = "1")]
    pub uptime_growth_outside:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
}
/// IncentiveRecord is the high-level struct we use to deal with an independent
/// incentive being distributed on a pool. Note that PoolId, Denom, and MinUptime
/// are included in the key so we avoid storing them in state, hence the
/// distinction between IncentiveRecord and IncentiveRecordBody.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.IncentiveRecord")]
pub struct IncentiveRecord {
    /// incentive_id is the id uniquely identifying this incentive record.
    #[prost(uint64, tag = "1")]
    #[serde(alias = "incentiveID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub incentive_id: u64,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    /// incentive record body holds necessary
    #[prost(message, optional, tag = "4")]
    pub incentive_record_body: ::core::option::Option<IncentiveRecordBody>,
    /// min_uptime is the minimum uptime required for liquidity to qualify for this
    /// incentive. It should be always be one of the supported uptimes in
    /// types.SupportedUptimes
    #[prost(message, optional, tag = "5")]
    pub min_uptime: ::core::option::Option<crate::shim::Duration>,
}
/// IncentiveRecordBody represents the body stored in state for each individual
/// record.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.IncentiveRecordBody")]
pub struct IncentiveRecordBody {
    /// remaining_coin is the total amount of incentives to be distributed
    #[prost(message, optional, tag = "1")]
    pub remaining_coin: ::core::option::Option<super::super::super::cosmos::base::v1beta1::DecCoin>,
    /// emission_rate is the incentive emission rate per second
    #[prost(string, tag = "2")]
    pub emission_rate: ::prost::alloc::string::String,
    /// start_time is the time when the incentive starts distributing
    #[prost(message, optional, tag = "3")]
    pub start_time: ::core::option::Option<crate::shim::Timestamp>,
}
/// FullTick contains tick index and pool id along with other tick model
/// information.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.FullTick")]
pub struct FullTick {
    /// pool id associated with the tick.
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    /// tick's index.
    #[prost(int64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_index: i64,
    /// tick's info.
    #[prost(message, optional, tag = "3")]
    pub info: ::core::option::Option<TickInfo>,
}
/// PoolData represents a serialized pool along with its ticks
/// for genesis state.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolData")]
pub struct PoolData {
    /// pool struct
    #[prost(message, optional, tag = "1")]
    pub pool: ::core::option::Option<crate::shim::Any>,
    /// pool's ticks
    #[prost(message, repeated, tag = "2")]
    pub ticks: ::prost::alloc::vec::Vec<FullTick>,
    #[prost(message, optional, tag = "3")]
    pub spread_reward_accumulator: ::core::option::Option<AccumObject>,
    #[prost(message, repeated, tag = "4")]
    pub incentives_accumulators: ::prost::alloc::vec::Vec<AccumObject>,
    /// incentive records to be set
    #[prost(message, repeated, tag = "5")]
    pub incentive_records: ::prost::alloc::vec::Vec<IncentiveRecord>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PositionData")]
pub struct PositionData {
    #[prost(message, optional, tag = "1")]
    pub position: ::core::option::Option<Position>,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(message, optional, tag = "3")]
    pub spread_reward_accum_record: ::core::option::Option<super::super::accum::v1beta1::Record>,
    #[prost(message, repeated, tag = "4")]
    pub uptime_accum_records: ::prost::alloc::vec::Vec<super::super::accum::v1beta1::Record>,
}
/// GenesisState defines the concentrated liquidity module's genesis state.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.GenesisState")]
pub struct GenesisState {
    /// params are all the parameters of the module
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<super::Params>,
    /// pool data containining serialized pool struct and ticks.
    #[prost(message, repeated, tag = "2")]
    pub pool_data: ::prost::alloc::vec::Vec<PoolData>,
    #[prost(message, repeated, tag = "3")]
    pub position_data: ::prost::alloc::vec::Vec<PositionData>,
    #[prost(uint64, tag = "4")]
    #[serde(alias = "next_positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub next_position_id: u64,
    #[prost(uint64, tag = "5")]
    #[serde(alias = "next_incentive_recordID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub next_incentive_record_id: u64,
}
/// In original struct of Accum object, store.KVStore is stored together.
/// For handling genesis, we do not need to include store.KVStore since we use
/// CL module's KVStore.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.AccumObject")]
pub struct AccumObject {
    /// Accumulator's name (pulled from AccumulatorContent)
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub accum_content: ::core::option::Option<super::super::accum::v1beta1::AccumulatorContent>,
}
/// CreateConcentratedLiquidityPoolsProposal is a gov Content type for creating
/// concentrated liquidity pools. If a CreateConcentratedLiquidityPoolsProposal
/// passes, the pools are created via pool manager module account.
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.CreateConcentratedLiquidityPoolsProposal"
)]
pub struct CreateConcentratedLiquidityPoolsProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    pub pool_records: ::prost::alloc::vec::Vec<PoolRecord>,
}
/// TickSpacingDecreaseProposal is a gov Content type for proposing a tick
/// spacing decrease for a pool. The proposal will fail if one of the pools do
/// not exist, or if the new tick spacing is not less than the current tick
/// spacing.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.TickSpacingDecreaseProposal")]
pub struct TickSpacingDecreaseProposal {
    #[prost(string, tag = "1")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub description: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "3")]
    #[serde(alias = "poolID_to_tick_spacing_records")]
    pub pool_id_to_tick_spacing_records: ::prost::alloc::vec::Vec<PoolIdToTickSpacingRecord>,
}
/// PoolIdToTickSpacingRecord is a struct that contains a pool id to new tick
/// spacing pair.
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolIdToTickSpacingRecord")]
pub struct PoolIdToTickSpacingRecord {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(uint64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub new_tick_spacing: u64,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolRecord")]
pub struct PoolRecord {
    #[prost(string, tag = "1")]
    pub denom0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom1: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_spacing: u64,
    #[prost(string, tag = "5")]
    pub spread_factor: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.Pool")]
pub struct Pool {
    /// pool's address holding all liquidity tokens.
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    /// address holding the incentives liquidity.
    #[prost(string, tag = "2")]
    pub incentives_address: ::prost::alloc::string::String,
    /// address holding spread rewards from swaps.
    #[prost(string, tag = "3")]
    pub spread_rewards_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
    /// Amount of total liquidity
    #[prost(string, tag = "5")]
    pub current_tick_liquidity: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub token0: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub token1: ::prost::alloc::string::String,
    #[prost(string, tag = "8")]
    pub current_sqrt_price: ::prost::alloc::string::String,
    #[prost(int64, tag = "9")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub current_tick: i64,
    /// tick_spacing must be one of the authorized_tick_spacing values set in the
    /// concentrated-liquidity parameters
    #[prost(uint64, tag = "10")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_spacing: u64,
    #[prost(int64, tag = "11")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub exponent_at_price_one: i64,
    /// spread_factor is the ratio that is charged on the amount of token in.
    #[prost(string, tag = "12")]
    pub spread_factor: ::prost::alloc::string::String,
    /// last_liquidity_update is the last time either the pool liquidity or the
    /// active tick changed
    #[prost(message, optional, tag = "13")]
    pub last_liquidity_update: ::core::option::Option<crate::shim::Timestamp>,
}
/// =============================== UserPositions
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UserPositionsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/UserPositions",
    response_type = UserPositionsResponse
)]
pub struct UserPositionsRequest {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(message, optional, tag = "3")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UserPositionsResponse")]
pub struct UserPositionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub positions: ::prost::alloc::vec::Vec<FullPositionBreakdown>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// =============================== PositionById
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PositionByIdRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/PositionById",
    response_type = PositionByIdResponse
)]
pub struct PositionByIdRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PositionByIdResponse")]
pub struct PositionByIdResponse {
    #[prost(message, optional, tag = "1")]
    pub position: ::core::option::Option<FullPositionBreakdown>,
}
/// =============================== Pools
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/Pools",
    response_type = PoolsResponse
)]
pub struct PoolsRequest {
    /// pagination defines an optional pagination for the request.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolsResponse")]
pub struct PoolsResponse {
    #[prost(message, repeated, tag = "1")]
    pub pools: ::prost::alloc::vec::Vec<crate::shim::Any>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// =============================== ModuleParams
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ParamsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/Params",
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ParamsResponse")]
pub struct ParamsResponse {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<super::Params>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.TickLiquidityNet")]
pub struct TickLiquidityNet {
    #[prost(string, tag = "1")]
    pub liquidity_net: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_index: i64,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.LiquidityDepthWithRange")]
pub struct LiquidityDepthWithRange {
    #[prost(string, tag = "1")]
    pub liquidity_amount: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lower_tick: i64,
    #[prost(int64, tag = "3")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub upper_tick: i64,
}
/// =============================== LiquidityNetInDirection
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.LiquidityNetInDirectionRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/LiquidityNetInDirection",
    response_type = LiquidityNetInDirectionResponse
)]
pub struct LiquidityNetInDirectionRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub token_in: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub start_tick: i64,
    #[prost(bool, tag = "4")]
    pub use_cur_tick: bool,
    #[prost(int64, tag = "5")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub bound_tick: i64,
    #[prost(bool, tag = "6")]
    pub use_no_bound: bool,
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.LiquidityNetInDirectionResponse"
)]
pub struct LiquidityNetInDirectionResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity_depths: ::prost::alloc::vec::Vec<TickLiquidityNet>,
    #[prost(int64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub current_tick: i64,
    #[prost(string, tag = "3")]
    pub current_liquidity: ::prost::alloc::string::String,
}
/// =============================== LiquidityPerTickRange
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.LiquidityPerTickRangeRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/LiquidityPerTickRange",
    response_type = LiquidityPerTickRangeResponse
)]
pub struct LiquidityPerTickRangeRequest {
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.LiquidityPerTickRangeResponse")]
pub struct LiquidityPerTickRangeResponse {
    #[prost(message, repeated, tag = "1")]
    pub liquidity: ::prost::alloc::vec::Vec<LiquidityDepthWithRange>,
}
/// ===================== QueryClaimableSpreadRewards
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ClaimableSpreadRewardsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/ClaimableSpreadRewards",
    response_type = ClaimableSpreadRewardsResponse
)]
pub struct ClaimableSpreadRewardsRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ClaimableSpreadRewardsResponse")]
pub struct ClaimableSpreadRewardsResponse {
    #[prost(message, repeated, tag = "1")]
    pub claimable_spread_rewards:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// ===================== QueryClaimableIncentives
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ClaimableIncentivesRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/ClaimableIncentives",
    response_type = ClaimableIncentivesResponse
)]
pub struct ClaimableIncentivesRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.ClaimableIncentivesResponse")]
pub struct ClaimableIncentivesResponse {
    #[prost(message, repeated, tag = "1")]
    pub claimable_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "2")]
    pub forfeited_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// ===================== QueryPoolAccumulatorRewards
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolAccumulatorRewardsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/PoolAccumulatorRewards",
    response_type = PoolAccumulatorRewardsResponse
)]
pub struct PoolAccumulatorRewardsRequest {
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.PoolAccumulatorRewardsResponse")]
pub struct PoolAccumulatorRewardsResponse {
    #[prost(message, repeated, tag = "1")]
    pub spread_reward_growth_global:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    #[prost(message, repeated, tag = "2")]
    pub uptime_growth_global: ::prost::alloc::vec::Vec<UptimeTracker>,
}
/// ===================== QueryTickAccumulatorTrackers
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.TickAccumulatorTrackersRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/TickAccumulatorTrackers",
    response_type = TickAccumulatorTrackersResponse
)]
pub struct TickAccumulatorTrackersRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(int64, tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub tick_index: i64,
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.TickAccumulatorTrackersResponse"
)]
pub struct TickAccumulatorTrackersResponse {
    #[prost(message, repeated, tag = "1")]
    pub spread_reward_growth_opposite_direction_of_last_traversal:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    #[prost(message, repeated, tag = "2")]
    pub uptime_trackers: ::prost::alloc::vec::Vec<UptimeTracker>,
}
/// ===================== QueryIncentiveRecords
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.IncentiveRecordsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/IncentiveRecords",
    response_type = IncentiveRecordsResponse
)]
pub struct IncentiveRecordsRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageRequest>,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.IncentiveRecordsResponse")]
pub struct IncentiveRecordsResponse {
    #[prost(message, repeated, tag = "1")]
    pub incentive_records: ::prost::alloc::vec::Vec<IncentiveRecord>,
    /// pagination defines the pagination in the response.
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::super::cosmos::base::query::v1beta1::PageResponse>,
}
/// =============================== CFMMPoolIdLinkFromConcentratedPoolId
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.CFMMPoolIdLinkFromConcentratedPoolIdRequest"
)]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/CFMMPoolIdLinkFromConcentratedPoolId",
    response_type = CfmmPoolIdLinkFromConcentratedPoolIdResponse
)]
pub struct CfmmPoolIdLinkFromConcentratedPoolIdRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "concentrated_poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub concentrated_pool_id: u64,
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.CFMMPoolIdLinkFromConcentratedPoolIdResponse"
)]
pub struct CfmmPoolIdLinkFromConcentratedPoolIdResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "cfmm_poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub cfmm_pool_id: u64,
}
/// =============================== UserUnbondingPositions
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UserUnbondingPositionsRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/UserUnbondingPositions",
    response_type = UserUnbondingPositionsResponse
)]
pub struct UserUnbondingPositionsRequest {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.UserUnbondingPositionsResponse")]
pub struct UserUnbondingPositionsResponse {
    #[prost(message, repeated, tag = "1")]
    pub positions_with_period_lock: ::prost::alloc::vec::Vec<PositionWithPeriodLock>,
}
/// =============================== GetTotalLiquidity
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.GetTotalLiquidityRequest")]
#[proto_query(
    path = "/osmosis.concentratedliquidity.v1beta1.Query/GetTotalLiquidity",
    response_type = GetTotalLiquidityResponse
)]
pub struct GetTotalLiquidityRequest {}
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.GetTotalLiquidityResponse")]
pub struct GetTotalLiquidityResponse {
    #[prost(message, repeated, tag = "1")]
    pub total_liquidity: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// ===================== MsgCreatePosition
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCreatePosition")]
pub struct MsgCreatePosition {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "poolID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub pool_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lower_tick: i64,
    #[prost(int64, tag = "4")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub upper_tick: i64,
    /// tokens_provided is the amount of tokens provided for the position.
    /// It must at a minimum be of length 1 (for a single sided position)
    /// and at a maximum be of length 2 (for a position that straddles the current
    /// tick).
    #[prost(message, repeated, tag = "5")]
    pub tokens_provided: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "6")]
    pub token_min_amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "7")]
    pub token_min_amount1: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCreatePositionResponse")]
pub struct MsgCreatePositionResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub liquidity_created: ::prost::alloc::string::String,
    /// the lower and upper tick are in the response because there are
    /// instances in which multiple ticks represent the same price, so
    /// we may move their provided tick to the canonical tick that represents
    /// the same price.
    #[prost(int64, tag = "6")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lower_tick: i64,
    #[prost(int64, tag = "7")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub upper_tick: i64,
}
/// ===================== MsgAddToPosition
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgAddToPosition")]
pub struct MsgAddToPosition {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    /// amount0 represents the amount of token0 willing to put in.
    #[prost(string, tag = "3")]
    pub amount0: ::prost::alloc::string::String,
    /// amount1 represents the amount of token1 willing to put in.
    #[prost(string, tag = "4")]
    pub amount1: ::prost::alloc::string::String,
    /// token_min_amount0 represents the minimum amount of token0 desired from the
    /// new position being created. Note that this field indicates the min amount0
    /// corresponding to the liquidity that is being added, not the total
    /// liquidity of the position.
    #[prost(string, tag = "5")]
    pub token_min_amount0: ::prost::alloc::string::String,
    /// token_min_amount1 represents the minimum amount of token1 desired from the
    /// new position being created. Note that this field indicates the min amount1
    /// corresponding to the liquidity that is being added, not the total
    /// liquidity of the position.
    #[prost(string, tag = "6")]
    pub token_min_amount1: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgAddToPositionResponse")]
pub struct MsgAddToPositionResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub amount1: ::prost::alloc::string::String,
}
/// ===================== MsgWithdrawPosition
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgWithdrawPosition")]
pub struct MsgWithdrawPosition {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub liquidity_amount: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgWithdrawPositionResponse")]
pub struct MsgWithdrawPositionResponse {
    #[prost(string, tag = "1")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount1: ::prost::alloc::string::String,
}
/// ===================== MsgCollectSpreadRewards
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCollectSpreadRewards")]
pub struct MsgCollectSpreadRewards {
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    #[serde(alias = "positionIDs")]
    pub position_ids: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCollectSpreadRewardsResponse"
)]
pub struct MsgCollectSpreadRewardsResponse {
    #[prost(message, repeated, tag = "1")]
    pub collected_spread_rewards:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// ===================== MsgCollectIncentives
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCollectIncentives")]
pub struct MsgCollectIncentives {
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    #[serde(alias = "positionIDs")]
    pub position_ids: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgCollectIncentivesResponse")]
pub struct MsgCollectIncentivesResponse {
    #[prost(message, repeated, tag = "1")]
    pub collected_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "2")]
    pub forfeited_incentives:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::Coin>,
}
/// ===================== MsgFungifyChargedPositions
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.v1beta1.MsgFungifyChargedPositions")]
pub struct MsgFungifyChargedPositions {
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    #[serde(alias = "positionIDs")]
    pub position_ids: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
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
    type_url = "/osmosis.concentratedliquidity.v1beta1.MsgFungifyChargedPositionsResponse"
)]
pub struct MsgFungifyChargedPositionsResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "new_positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub new_position_id: u64,
}
pub struct ConcentratedliquidityQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> ConcentratedliquidityQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn pools(
        &self,
        pagination: ::core::option::Option<
            super::super::super::cosmos::base::query::v1beta1::PageRequest,
        >,
    ) -> Result<PoolsResponse, cosmwasm_std::StdError> {
        PoolsRequest { pagination }.query(self.querier)
    }
    pub fn params(&self) -> Result<ParamsResponse, cosmwasm_std::StdError> {
        ParamsRequest {}.query(self.querier)
    }
    pub fn user_positions(
        &self,
        address: ::prost::alloc::string::String,
        pool_id: u64,
        pagination: ::core::option::Option<
            super::super::super::cosmos::base::query::v1beta1::PageRequest,
        >,
    ) -> Result<UserPositionsResponse, cosmwasm_std::StdError> {
        UserPositionsRequest {
            address,
            pool_id,
            pagination,
        }
        .query(self.querier)
    }
    pub fn liquidity_per_tick_range(
        &self,
        pool_id: u64,
    ) -> Result<LiquidityPerTickRangeResponse, cosmwasm_std::StdError> {
        LiquidityPerTickRangeRequest { pool_id }.query(self.querier)
    }
    pub fn liquidity_net_in_direction(
        &self,
        pool_id: u64,
        token_in: ::prost::alloc::string::String,
        start_tick: i64,
        use_cur_tick: bool,
        bound_tick: i64,
        use_no_bound: bool,
    ) -> Result<LiquidityNetInDirectionResponse, cosmwasm_std::StdError> {
        LiquidityNetInDirectionRequest {
            pool_id,
            token_in,
            start_tick,
            use_cur_tick,
            bound_tick,
            use_no_bound,
        }
        .query(self.querier)
    }
    pub fn claimable_spread_rewards(
        &self,
        position_id: u64,
    ) -> Result<ClaimableSpreadRewardsResponse, cosmwasm_std::StdError> {
        ClaimableSpreadRewardsRequest { position_id }.query(self.querier)
    }
    pub fn claimable_incentives(
        &self,
        position_id: u64,
    ) -> Result<ClaimableIncentivesResponse, cosmwasm_std::StdError> {
        ClaimableIncentivesRequest { position_id }.query(self.querier)
    }
    pub fn position_by_id(
        &self,
        position_id: u64,
    ) -> Result<PositionByIdResponse, cosmwasm_std::StdError> {
        PositionByIdRequest { position_id }.query(self.querier)
    }
    pub fn pool_accumulator_rewards(
        &self,
        pool_id: u64,
    ) -> Result<PoolAccumulatorRewardsResponse, cosmwasm_std::StdError> {
        PoolAccumulatorRewardsRequest { pool_id }.query(self.querier)
    }
    pub fn incentive_records(
        &self,
        pool_id: u64,
        pagination: ::core::option::Option<
            super::super::super::cosmos::base::query::v1beta1::PageRequest,
        >,
    ) -> Result<IncentiveRecordsResponse, cosmwasm_std::StdError> {
        IncentiveRecordsRequest {
            pool_id,
            pagination,
        }
        .query(self.querier)
    }
    pub fn tick_accumulator_trackers(
        &self,
        pool_id: u64,
        tick_index: i64,
    ) -> Result<TickAccumulatorTrackersResponse, cosmwasm_std::StdError> {
        TickAccumulatorTrackersRequest {
            pool_id,
            tick_index,
        }
        .query(self.querier)
    }
    pub fn cfmm_pool_id_link_from_concentrated_pool_id(
        &self,
        concentrated_pool_id: u64,
    ) -> Result<CfmmPoolIdLinkFromConcentratedPoolIdResponse, cosmwasm_std::StdError> {
        CfmmPoolIdLinkFromConcentratedPoolIdRequest {
            concentrated_pool_id,
        }
        .query(self.querier)
    }
    pub fn user_unbonding_positions(
        &self,
        address: ::prost::alloc::string::String,
    ) -> Result<UserUnbondingPositionsResponse, cosmwasm_std::StdError> {
        UserUnbondingPositionsRequest { address }.query(self.querier)
    }
    pub fn get_total_liquidity(&self) -> Result<GetTotalLiquidityResponse, cosmwasm_std::StdError> {
        GetTotalLiquidityRequest {}.query(self.querier)
    }
}
