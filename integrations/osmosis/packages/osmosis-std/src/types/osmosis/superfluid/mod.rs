pub mod v1beta1;
use osmosis_std_derive::CosmwasmExt;
/// SuperfluidAsset stores the pair of superfluid asset type and denom pair
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidAsset")]
pub struct SuperfluidAsset {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    /// AssetType indicates whether the superfluid asset is a native token or an lp
    /// share
    #[prost(enumeration = "SuperfluidAssetType", tag = "2")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub asset_type: i32,
}
/// SuperfluidIntermediaryAccount takes the role of intermediary between LP token
/// and OSMO tokens for superfluid staking. The intermediary account is the
/// actual account responsible for delegation, not the validator account itself.
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidIntermediaryAccount")]
pub struct SuperfluidIntermediaryAccount {
    /// Denom indicates the denom of the superfluid asset.
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub val_addr: ::prost::alloc::string::String,
    /// perpetual gauge for rewards distribution
    #[prost(uint64, tag = "3")]
    #[serde(alias = "gaugeID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub gauge_id: u64,
}
/// The Osmo-Equivalent-Multiplier Record for epoch N refers to the osmo worth we
/// treat an LP share as having, for all of epoch N. Eventually this is intended
/// to be set as the Time-weighted-average-osmo-backing for the entire duration
/// of epoch N-1. (Thereby locking whats in use for epoch N as based on the prior
/// epochs rewards) However for now, this is not the TWAP but instead the spot
/// price at the boundary. For different types of assets in the future, it could
/// change.
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
#[proto_message(type_url = "/osmosis.superfluid.OsmoEquivalentMultiplierRecord")]
pub struct OsmoEquivalentMultiplierRecord {
    #[prost(int64, tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub epoch_number: i64,
    /// superfluid asset denom, can be LP token or native token
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub multiplier: ::prost::alloc::string::String,
}
/// SuperfluidDelegationRecord is a struct used to indicate superfluid
/// delegations of an account in the state machine in a user friendly form.
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationRecord")]
pub struct SuperfluidDelegationRecord {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub delegation_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub equivalent_staked_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
/// LockIdIntermediaryAccountConnection is a struct used to indicate the
/// relationship between the underlying lock id and superfluid delegation done
/// via lp shares.
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
#[proto_message(type_url = "/osmosis.superfluid.LockIdIntermediaryAccountConnection")]
pub struct LockIdIntermediaryAccountConnection {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(string, tag = "2")]
    pub intermediary_account: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.UnpoolWhitelistedPools")]
pub struct UnpoolWhitelistedPools {
    #[prost(uint64, repeated, tag = "1")]
    pub ids: ::prost::alloc::vec::Vec<u64>,
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
#[proto_message(type_url = "/osmosis.superfluid.ConcentratedPoolUserPositionRecord")]
pub struct ConcentratedPoolUserPositionRecord {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(message, optional, tag = "4")]
    pub synthetic_lock: ::core::option::Option<super::lockup::SyntheticLock>,
    #[prost(message, optional, tag = "5")]
    pub delegation_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "6")]
    pub equivalent_staked_amount: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
}
/// SuperfluidAssetType indicates whether the superfluid asset is
/// a native token, lp share of a pool, or concentrated share of a pool
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
pub enum SuperfluidAssetType {
    Native = 0,
    LpShare = 1,
    /// SuperfluidAssetTypeLendingShare = 3; // for now not exist
    ConcentratedShare = 2,
}
impl SuperfluidAssetType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SuperfluidAssetType::Native => "SuperfluidAssetTypeNative",
            SuperfluidAssetType::LpShare => "SuperfluidAssetTypeLPShare",
            SuperfluidAssetType::ConcentratedShare => "SuperfluidAssetTypeConcentratedShare",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SuperfluidAssetTypeNative" => Some(Self::Native),
            "SuperfluidAssetTypeLPShare" => Some(Self::LpShare),
            "SuperfluidAssetTypeConcentratedShare" => Some(Self::ConcentratedShare),
            _ => None,
        }
    }
}
/// Params holds parameters for the superfluid module
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
#[proto_message(type_url = "/osmosis.superfluid.Params")]
pub struct Params {
    /// minimum_risk_factor is to be cut on OSMO equivalent value of lp tokens for
    /// superfluid staking, default: 5%. The minimum risk factor works
    /// to counter-balance the staked amount on chain's exposure to various asset
    /// volatilities, and have base staking be 'resistant' to volatility.
    #[prost(string, tag = "1")]
    pub minimum_risk_factor: ::prost::alloc::string::String,
}
/// GenesisState defines the module's genesis state.
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
#[proto_message(type_url = "/osmosis.superfluid.GenesisState")]
pub struct GenesisState {
    #[prost(message, optional, tag = "1")]
    pub params: ::core::option::Option<Params>,
    /// superfluid_assets defines the registered superfluid assets that have been
    /// registered via governance.
    #[prost(message, repeated, tag = "2")]
    pub superfluid_assets: ::prost::alloc::vec::Vec<SuperfluidAsset>,
    /// osmo_equivalent_multipliers is the records of osmo equivalent amount of
    /// each superfluid registered pool, updated every epoch.
    #[prost(message, repeated, tag = "3")]
    pub osmo_equivalent_multipliers: ::prost::alloc::vec::Vec<OsmoEquivalentMultiplierRecord>,
    /// intermediary_accounts is a secondary account for superfluid staking that
    /// plays an intermediary role between validators and the delegators.
    #[prost(message, repeated, tag = "4")]
    pub intermediary_accounts: ::prost::alloc::vec::Vec<SuperfluidIntermediaryAccount>,
    #[prost(message, repeated, tag = "5")]
    pub intemediary_account_connections:
        ::prost::alloc::vec::Vec<LockIdIntermediaryAccountConnection>,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryParamsRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/Params",
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
#[proto_message(type_url = "/osmosis.superfluid.QueryParamsResponse")]
pub struct QueryParamsResponse {
    /// params defines the parameters of the module.
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
#[proto_message(type_url = "/osmosis.superfluid.AssetTypeRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/AssetType",
    response_type = AssetTypeResponse
)]
pub struct AssetTypeRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.AssetTypeResponse")]
pub struct AssetTypeResponse {
    #[prost(enumeration = "SuperfluidAssetType", tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub asset_type: i32,
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
#[proto_message(type_url = "/osmosis.superfluid.AllAssetsRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/AllAssets",
    response_type = AllAssetsResponse
)]
pub struct AllAssetsRequest {}
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
#[proto_message(type_url = "/osmosis.superfluid.AllAssetsResponse")]
pub struct AllAssetsResponse {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<SuperfluidAsset>,
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
#[proto_message(type_url = "/osmosis.superfluid.AssetMultiplierRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/AssetMultiplier",
    response_type = AssetMultiplierResponse
)]
pub struct AssetMultiplierRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.AssetMultiplierResponse")]
pub struct AssetMultiplierResponse {
    #[prost(message, optional, tag = "1")]
    pub osmo_equivalent_multiplier: ::core::option::Option<OsmoEquivalentMultiplierRecord>,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidIntermediaryAccountInfo")]
pub struct SuperfluidIntermediaryAccountInfo {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(uint64, tag = "3")]
    #[serde(alias = "gaugeID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub gauge_id: u64,
    #[prost(string, tag = "4")]
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
#[proto_message(type_url = "/osmosis.superfluid.AllIntermediaryAccountsRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/AllIntermediaryAccounts",
    response_type = AllIntermediaryAccountsResponse
)]
pub struct AllIntermediaryAccountsRequest {
    #[prost(message, optional, tag = "1")]
    pub pagination: ::core::option::Option<super::super::cosmos::base::query::v1beta1::PageRequest>,
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
#[proto_message(type_url = "/osmosis.superfluid.AllIntermediaryAccountsResponse")]
pub struct AllIntermediaryAccountsResponse {
    #[prost(message, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<SuperfluidIntermediaryAccountInfo>,
    #[prost(message, optional, tag = "2")]
    pub pagination:
        ::core::option::Option<super::super::cosmos::base::query::v1beta1::PageResponse>,
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
#[proto_message(type_url = "/osmosis.superfluid.ConnectedIntermediaryAccountRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/ConnectedIntermediaryAccount",
    response_type = ConnectedIntermediaryAccountResponse
)]
pub struct ConnectedIntermediaryAccountRequest {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
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
#[proto_message(type_url = "/osmosis.superfluid.ConnectedIntermediaryAccountResponse")]
pub struct ConnectedIntermediaryAccountResponse {
    #[prost(message, optional, tag = "1")]
    pub account: ::core::option::Option<SuperfluidIntermediaryAccountInfo>,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryTotalDelegationByValidatorForDenomRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/TotalDelegationByValidatorForDenom",
    response_type = QueryTotalDelegationByValidatorForDenomResponse
)]
pub struct QueryTotalDelegationByValidatorForDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryTotalDelegationByValidatorForDenomResponse")]
pub struct QueryTotalDelegationByValidatorForDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub assets: ::prost::alloc::vec::Vec<Delegations>,
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
#[proto_message(type_url = "/osmosis.superfluid.Delegations")]
pub struct Delegations {
    #[prost(string, tag = "1")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount_sfsd: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub osmo_equivalent: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.TotalSuperfluidDelegationsRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/TotalSuperfluidDelegations",
    response_type = TotalSuperfluidDelegationsResponse
)]
pub struct TotalSuperfluidDelegationsRequest {}
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
#[proto_message(type_url = "/osmosis.superfluid.TotalSuperfluidDelegationsResponse")]
pub struct TotalSuperfluidDelegationsResponse {
    #[prost(string, tag = "1")]
    pub total_delegations: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationAmountRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/SuperfluidDelegationAmount",
    response_type = SuperfluidDelegationAmountResponse
)]
pub struct SuperfluidDelegationAmountRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationAmountResponse")]
pub struct SuperfluidDelegationAmountResponse {
    #[prost(message, repeated, tag = "1")]
    pub amount: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationsByDelegatorRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/SuperfluidDelegationsByDelegator",
    response_type = SuperfluidDelegationsByDelegatorResponse
)]
pub struct SuperfluidDelegationsByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationsByDelegatorResponse")]
pub struct SuperfluidDelegationsByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "3")]
    pub total_equivalent_staked_amount:
        ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidUndelegationsByDelegatorRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/SuperfluidUndelegationsByDelegator",
    response_type = SuperfluidUndelegationsByDelegatorResponse
)]
pub struct SuperfluidUndelegationsByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidUndelegationsByDelegatorResponse")]
pub struct SuperfluidUndelegationsByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub total_undelegated_coins:
        ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, repeated, tag = "3")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<super::lockup::SyntheticLock>,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationsByValidatorDenomRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/SuperfluidDelegationsByValidatorDenom",
    response_type = SuperfluidDelegationsByValidatorDenomResponse
)]
pub struct SuperfluidDelegationsByValidatorDenomRequest {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.SuperfluidDelegationsByValidatorDenomResponse")]
pub struct SuperfluidDelegationsByValidatorDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
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
    type_url = "/osmosis.superfluid.EstimateSuperfluidDelegatedAmountByValidatorDenomRequest"
)]
#[proto_query(
    path = "/osmosis.superfluid.Query/EstimateSuperfluidDelegatedAmountByValidatorDenom",
    response_type = EstimateSuperfluidDelegatedAmountByValidatorDenomResponse
)]
pub struct EstimateSuperfluidDelegatedAmountByValidatorDenomRequest {
    #[prost(string, tag = "1")]
    pub validator_address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
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
    type_url = "/osmosis.superfluid.EstimateSuperfluidDelegatedAmountByValidatorDenomResponse"
)]
pub struct EstimateSuperfluidDelegatedAmountByValidatorDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryTotalDelegationByDelegatorRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/TotalDelegationByDelegator",
    response_type = QueryTotalDelegationByDelegatorResponse
)]
pub struct QueryTotalDelegationByDelegatorRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryTotalDelegationByDelegatorResponse")]
pub struct QueryTotalDelegationByDelegatorResponse {
    #[prost(message, repeated, tag = "1")]
    pub superfluid_delegation_records: ::prost::alloc::vec::Vec<SuperfluidDelegationRecord>,
    #[prost(message, repeated, tag = "2")]
    pub delegation_response:
        ::prost::alloc::vec::Vec<super::super::cosmos::staking::v1beta1::DelegationResponse>,
    #[prost(message, repeated, tag = "3")]
    pub total_delegated_coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub total_equivalent_staked_amount:
        ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.superfluid.QueryUnpoolWhitelistRequest")]
#[proto_query(
    path = "/osmosis.superfluid.Query/UnpoolWhitelist",
    response_type = QueryUnpoolWhitelistResponse
)]
pub struct QueryUnpoolWhitelistRequest {}
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
#[proto_message(type_url = "/osmosis.superfluid.QueryUnpoolWhitelistResponse")]
pub struct QueryUnpoolWhitelistResponse {
    #[prost(uint64, repeated, tag = "1")]
    #[serde(alias = "poolIDs")]
    pub pool_ids: ::prost::alloc::vec::Vec<u64>,
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
    type_url = "/osmosis.superfluid.UserConcentratedSuperfluidPositionsDelegatedRequest"
)]
#[proto_query(
    path = "/osmosis.superfluid.Query/UserConcentratedSuperfluidPositionsDelegated",
    response_type = UserConcentratedSuperfluidPositionsDelegatedResponse
)]
pub struct UserConcentratedSuperfluidPositionsDelegatedRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
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
    type_url = "/osmosis.superfluid.UserConcentratedSuperfluidPositionsDelegatedResponse"
)]
pub struct UserConcentratedSuperfluidPositionsDelegatedResponse {
    #[prost(message, repeated, tag = "1")]
    pub cl_pool_user_position_records: ::prost::alloc::vec::Vec<ConcentratedPoolUserPositionRecord>,
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
    type_url = "/osmosis.superfluid.UserConcentratedSuperfluidPositionsUndelegatingRequest"
)]
#[proto_query(
    path = "/osmosis.superfluid.Query/UserConcentratedSuperfluidPositionsUndelegating",
    response_type = UserConcentratedSuperfluidPositionsUndelegatingResponse
)]
pub struct UserConcentratedSuperfluidPositionsUndelegatingRequest {
    #[prost(string, tag = "1")]
    pub delegator_address: ::prost::alloc::string::String,
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
    type_url = "/osmosis.superfluid.UserConcentratedSuperfluidPositionsUndelegatingResponse"
)]
pub struct UserConcentratedSuperfluidPositionsUndelegatingResponse {
    #[prost(message, repeated, tag = "1")]
    pub cl_pool_user_position_records: ::prost::alloc::vec::Vec<ConcentratedPoolUserPositionRecord>,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidDelegate")]
pub struct MsgSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidDelegateResponse")]
pub struct MsgSuperfluidDelegateResponse {}
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUndelegate")]
pub struct MsgSuperfluidUndelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUndelegateResponse")]
pub struct MsgSuperfluidUndelegateResponse {}
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUnbondLock")]
pub struct MsgSuperfluidUnbondLock {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUnbondLockResponse")]
pub struct MsgSuperfluidUnbondLockResponse {}
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUndelegateAndUnbondLock")]
pub struct MsgSuperfluidUndelegateAndUnbondLock {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    /// Amount of unlocking coin.
    #[prost(message, optional, tag = "3")]
    pub coin: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgSuperfluidUndelegateAndUnbondLockResponse")]
pub struct MsgSuperfluidUndelegateAndUnbondLockResponse {
    /// lock id of the new lock created for the remaining amount.
    /// returns the original lockid if the unlocked amount is equal to the
    /// original lock's amount.
    #[prost(uint64, tag = "1")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
}
/// MsgLockAndSuperfluidDelegate locks coins with the unbonding period duration,
/// and then does a superfluid lock from the newly created lockup, to the
/// specified validator addr.
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
#[proto_message(type_url = "/osmosis.superfluid.MsgLockAndSuperfluidDelegate")]
pub struct MsgLockAndSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.superfluid.MsgLockAndSuperfluidDelegateResponse")]
pub struct MsgLockAndSuperfluidDelegateResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
}
/// MsgCreateFullRangePositionAndSuperfluidDelegate creates a full range position
/// in a concentrated liquidity pool, then superfluid delegates.
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
#[proto_message(type_url = "/osmosis.superfluid.MsgCreateFullRangePositionAndSuperfluidDelegate")]
pub struct MsgCreateFullRangePositionAndSuperfluidDelegate {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub val_addr: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
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
#[proto_message(
    type_url = "/osmosis.superfluid.MsgCreateFullRangePositionAndSuperfluidDelegateResponse"
)]
pub struct MsgCreateFullRangePositionAndSuperfluidDelegateResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
}
/// MsgUnPoolWhitelistedPool Unpools every lock the sender has, that is
/// associated with pool pool_id. If pool_id is not approved for unpooling by
/// governance, this is a no-op. Unpooling takes the locked gamm shares, and runs
/// "ExitPool" on it, to get the constituent tokens. e.g. z gamm/pool/1 tokens
/// ExitPools into constituent tokens x uatom, y uosmo. Then it creates a new
/// lock for every constituent token, with the duration associated with the lock.
/// If the lock was unbonding, the new lockup durations should be the time left
/// until unbond completion.
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
#[proto_message(type_url = "/osmosis.superfluid.MsgUnPoolWhitelistedPool")]
pub struct MsgUnPoolWhitelistedPool {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
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
#[proto_message(type_url = "/osmosis.superfluid.MsgUnPoolWhitelistedPoolResponse")]
pub struct MsgUnPoolWhitelistedPoolResponse {
    #[prost(uint64, repeated, tag = "1")]
    #[serde(alias = "exited_lockIDs")]
    pub exited_lock_ids: ::prost::alloc::vec::Vec<u64>,
}
/// =====================
/// MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition
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
    type_url = "/osmosis.superfluid.MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition"
)]
pub struct MsgUnlockAndMigrateSharesToFullRangeConcentratedPosition {
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: i64,
    #[prost(message, optional, tag = "3")]
    pub shares_to_migrate: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    /// token_out_mins indicates minimum token to exit Balancer pool with.
    #[prost(message, repeated, tag = "4")]
    pub token_out_mins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
    type_url = "/osmosis.superfluid.MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse"
)]
pub struct MsgUnlockAndMigrateSharesToFullRangeConcentratedPositionResponse {
    #[prost(string, tag = "1")]
    pub amount0: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount1: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub liquidity_created: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub join_time: ::core::option::Option<crate::shim::Timestamp>,
}
/// ===================== MsgAddToConcentratedLiquiditySuperfluidPosition
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
#[proto_message(type_url = "/osmosis.superfluid.MsgAddToConcentratedLiquiditySuperfluidPosition")]
pub struct MsgAddToConcentratedLiquiditySuperfluidPosition {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "positionID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub position_id: u64,
    #[prost(string, tag = "2")]
    pub sender: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub token_desired0: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag = "4")]
    pub token_desired1: ::core::option::Option<super::super::cosmos::base::v1beta1::Coin>,
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
    type_url = "/osmosis.superfluid.MsgAddToConcentratedLiquiditySuperfluidPositionResponse"
)]
pub struct MsgAddToConcentratedLiquiditySuperfluidPositionResponse {
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
    /// new_liquidity is the final liquidity after the add.
    /// It includes the liquidity that existed before in the position
    /// and the new liquidity that was added to the position.
    #[prost(string, tag = "5")]
    pub new_liquidity: ::prost::alloc::string::String,
    #[prost(uint64, tag = "4")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
}
pub struct SuperfluidQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> SuperfluidQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn params(&self) -> Result<QueryParamsResponse, cosmwasm_std::StdError> {
        QueryParamsRequest {}.query(self.querier)
    }
    pub fn asset_type(
        &self,
        denom: ::prost::alloc::string::String,
    ) -> Result<AssetTypeResponse, cosmwasm_std::StdError> {
        AssetTypeRequest { denom }.query(self.querier)
    }
    pub fn all_assets(&self) -> Result<AllAssetsResponse, cosmwasm_std::StdError> {
        AllAssetsRequest {}.query(self.querier)
    }
    pub fn asset_multiplier(
        &self,
        denom: ::prost::alloc::string::String,
    ) -> Result<AssetMultiplierResponse, cosmwasm_std::StdError> {
        AssetMultiplierRequest { denom }.query(self.querier)
    }
    pub fn all_intermediary_accounts(
        &self,
        pagination: ::core::option::Option<super::super::cosmos::base::query::v1beta1::PageRequest>,
    ) -> Result<AllIntermediaryAccountsResponse, cosmwasm_std::StdError> {
        AllIntermediaryAccountsRequest { pagination }.query(self.querier)
    }
    pub fn connected_intermediary_account(
        &self,
        lock_id: u64,
    ) -> Result<ConnectedIntermediaryAccountResponse, cosmwasm_std::StdError> {
        ConnectedIntermediaryAccountRequest { lock_id }.query(self.querier)
    }
    pub fn total_delegation_by_validator_for_denom(
        &self,
        denom: ::prost::alloc::string::String,
    ) -> Result<QueryTotalDelegationByValidatorForDenomResponse, cosmwasm_std::StdError> {
        QueryTotalDelegationByValidatorForDenomRequest { denom }.query(self.querier)
    }
    pub fn total_superfluid_delegations(
        &self,
    ) -> Result<TotalSuperfluidDelegationsResponse, cosmwasm_std::StdError> {
        TotalSuperfluidDelegationsRequest {}.query(self.querier)
    }
    pub fn superfluid_delegation_amount(
        &self,
        delegator_address: ::prost::alloc::string::String,
        validator_address: ::prost::alloc::string::String,
        denom: ::prost::alloc::string::String,
    ) -> Result<SuperfluidDelegationAmountResponse, cosmwasm_std::StdError> {
        SuperfluidDelegationAmountRequest {
            delegator_address,
            validator_address,
            denom,
        }
        .query(self.querier)
    }
    pub fn superfluid_delegations_by_delegator(
        &self,
        delegator_address: ::prost::alloc::string::String,
    ) -> Result<SuperfluidDelegationsByDelegatorResponse, cosmwasm_std::StdError> {
        SuperfluidDelegationsByDelegatorRequest { delegator_address }.query(self.querier)
    }
    pub fn superfluid_undelegations_by_delegator(
        &self,
        delegator_address: ::prost::alloc::string::String,
        denom: ::prost::alloc::string::String,
    ) -> Result<SuperfluidUndelegationsByDelegatorResponse, cosmwasm_std::StdError> {
        SuperfluidUndelegationsByDelegatorRequest {
            delegator_address,
            denom,
        }
        .query(self.querier)
    }
    pub fn superfluid_delegations_by_validator_denom(
        &self,
        validator_address: ::prost::alloc::string::String,
        denom: ::prost::alloc::string::String,
    ) -> Result<SuperfluidDelegationsByValidatorDenomResponse, cosmwasm_std::StdError> {
        SuperfluidDelegationsByValidatorDenomRequest {
            validator_address,
            denom,
        }
        .query(self.querier)
    }
    pub fn estimate_superfluid_delegated_amount_by_validator_denom(
        &self,
        validator_address: ::prost::alloc::string::String,
        denom: ::prost::alloc::string::String,
    ) -> Result<EstimateSuperfluidDelegatedAmountByValidatorDenomResponse, cosmwasm_std::StdError>
    {
        EstimateSuperfluidDelegatedAmountByValidatorDenomRequest {
            validator_address,
            denom,
        }
        .query(self.querier)
    }
    pub fn total_delegation_by_delegator(
        &self,
        delegator_address: ::prost::alloc::string::String,
    ) -> Result<QueryTotalDelegationByDelegatorResponse, cosmwasm_std::StdError> {
        QueryTotalDelegationByDelegatorRequest { delegator_address }.query(self.querier)
    }
    pub fn unpool_whitelist(&self) -> Result<QueryUnpoolWhitelistResponse, cosmwasm_std::StdError> {
        QueryUnpoolWhitelistRequest {}.query(self.querier)
    }
    pub fn user_concentrated_superfluid_positions_delegated(
        &self,
        delegator_address: ::prost::alloc::string::String,
    ) -> Result<UserConcentratedSuperfluidPositionsDelegatedResponse, cosmwasm_std::StdError> {
        UserConcentratedSuperfluidPositionsDelegatedRequest { delegator_address }
            .query(self.querier)
    }
    pub fn user_concentrated_superfluid_positions_undelegating(
        &self,
        delegator_address: ::prost::alloc::string::String,
    ) -> Result<UserConcentratedSuperfluidPositionsUndelegatingResponse, cosmwasm_std::StdError>
    {
        UserConcentratedSuperfluidPositionsUndelegatingRequest { delegator_address }
            .query(self.querier)
    }
}
