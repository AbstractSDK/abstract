use osmosis_std_derive::CosmwasmExt;
/// AccumulatorContent is the state-entry for the global accumulator.
/// It contains the name of the global accumulator and the total value of
/// shares belonging to it from all positions.
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
#[proto_message(type_url = "/osmosis.accum.v1beta1.AccumulatorContent")]
pub struct AccumulatorContent {
    #[prost(message, repeated, tag = "1")]
    pub accum_value: ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    #[prost(string, tag = "2")]
    pub total_shares: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.accum.v1beta1.Options")]
pub struct Options {}
/// Record corresponds to an individual position value belonging to the
/// global accumulator.
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
#[proto_message(type_url = "/osmosis.accum.v1beta1.Record")]
pub struct Record {
    /// num_shares is the number of shares belonging to the position associated
    /// with this record.
    #[prost(string, tag = "1")]
    pub num_shares: ::prost::alloc::string::String,
    /// accum_value_per_share is the subset of coins per shar of the global
    /// accumulator value that allows to infer how much a position is entitled to
    /// per share that it owns.
    ///
    /// In the default case with no intervals, this value equals to the global
    /// accumulator value at the time of the position creation, the last update or
    /// reward claim.
    ///
    /// In the interval case such as concentrated liquidity, this value equals to
    /// the global growth of rewards inside the interval during one of: the time of
    /// the position creation, the last update or reward claim. Note, that
    /// immediately prior to claiming or updating rewards, this value must be
    /// updated to "the growth inside at the time of last update + the growth
    /// outside at the time of the current block". This is so that the claiming
    /// logic can subtract this updated value from the global accumulator value to
    /// get the growth inside the interval from the time of last update up until
    /// the current block time.
    #[prost(message, repeated, tag = "2")]
    pub accum_value_per_share:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    /// unclaimed_rewards_total is the total amount of unclaimed rewards that the
    /// position is entitled to. This value is updated whenever shares are added or
    /// removed from an existing position. We also expose API for manually updating
    /// this value for some custom use cases such as merging pre-existing positions
    /// into a single one.
    #[prost(message, repeated, tag = "3")]
    pub unclaimed_rewards_total:
        ::prost::alloc::vec::Vec<super::super::super::cosmos::base::v1beta1::DecCoin>,
    #[prost(message, optional, tag = "4")]
    pub options: ::core::option::Option<Options>,
}
