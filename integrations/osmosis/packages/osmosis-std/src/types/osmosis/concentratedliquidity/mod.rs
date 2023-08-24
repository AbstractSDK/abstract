pub mod poolmodel;
pub mod v1beta1;
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
#[proto_message(type_url = "/osmosis.concentratedliquidity.Params")]
pub struct Params {
    /// authorized_tick_spacing is an array of uint64s that represents the tick
    /// spacing values concentrated-liquidity pools can be created with. For
    /// example, an authorized_tick_spacing of [1, 10, 30] allows for pools
    /// to be created with tick spacing of 1, 10, or 30.
    #[prost(uint64, repeated, packed = "false", tag = "1")]
    pub authorized_tick_spacing: ::prost::alloc::vec::Vec<u64>,
    #[prost(string, repeated, tag = "2")]
    pub authorized_spread_factors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// balancer_shares_reward_discount is the rate by which incentives flowing
    /// from CL to Balancer pools will be discounted to encourage LPs to migrate.
    /// e.g. a rate of 0.05 means Balancer LPs get 5% less incentives than full
    /// range CL LPs.
    /// This field can range from (0,1]. If set to 1, it indicates that all
    /// incentives stay at cl pool.
    #[prost(string, tag = "3")]
    pub balancer_shares_reward_discount: ::prost::alloc::string::String,
    /// authorized_quote_denoms is a list of quote denoms that can be used as
    /// token1 when creating a pool. We limit the quote assets to a small set for
    /// the purposes of having convinient price increments stemming from tick to
    /// price conversion. These increments are in a human readable magnitude only
    /// for token1 as a quote. For limit orders in the future, this will be a
    /// desirable property in terms of UX as to allow users to set limit orders at
    /// prices in terms of token1 (quote asset) that are easy to reason about.
    #[prost(string, repeated, tag = "4")]
    pub authorized_quote_denoms: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "5")]
    pub authorized_uptimes: ::prost::alloc::vec::Vec<crate::shim::Duration>,
    /// is_permissionless_pool_creation_enabled is a boolean that determines if
    /// concentrated liquidity pools can be created via message. At launch,
    /// we consider allowing only governance to create pools, and then later
    /// allowing permissionless pool creation by switching this flag to true
    /// with a governance proposal.
    #[prost(bool, tag = "6")]
    pub is_permissionless_pool_creation_enabled: bool,
}
