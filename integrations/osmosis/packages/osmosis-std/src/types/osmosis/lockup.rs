use osmosis_std_derive::CosmwasmExt;
/// PeriodLock is a single lock unit by period defined by the x/lockup module.
/// It's a record of a locked coin at a specific time. It stores owner, duration,
/// unlock time and the number of coins locked. A state of a period lock is
/// created upon lock creation, and deleted once the lock has been matured after
/// the `duration` has passed since unbonding started.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.PeriodLock")]
pub struct PeriodLock {
    /// ID is the unique id of the lock.
    /// The ID of the lock is decided upon lock creation, incrementing by 1 for
    /// every lock.
    #[prost(uint64, tag = "1")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
    /// Owner is the account address of the lock owner.
    /// Only the owner can modify the state of the lock.
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    /// Duration is the time needed for a lock to mature after unlocking has
    /// started.
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
    /// EndTime refers to the time at which the lock would mature and get deleted.
    /// This value is first initialized when an unlock has started for the lock,
    /// end time being block time + duration.
    #[prost(message, optional, tag = "4")]
    pub end_time: ::core::option::Option<crate::shim::Timestamp>,
    /// Coins are the tokens locked within the lock, kept in the module account.
    #[prost(message, repeated, tag = "5")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
    /// Reward Receiver Address is the address that would be receiving rewards for
    /// the incentives for the lock. This is set to owner by default and can be
    /// changed via separate msg.
    #[prost(string, tag = "6")]
    pub reward_receiver_address: ::prost::alloc::string::String,
}
/// QueryCondition is a struct used for querying locks upon different conditions.
/// Duration field and timestamp fields could be optional, depending on the
/// LockQueryType.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.QueryCondition")]
pub struct QueryCondition {
    /// LockQueryType is a type of lock query, ByLockDuration | ByLockTime
    #[prost(enumeration = "LockQueryType", tag = "1")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_query_type: i32,
    /// Denom represents the token denomination we are looking to lock up
    #[prost(string, tag = "2")]
    pub denom: ::prost::alloc::string::String,
    /// Duration is used to query locks with longer duration than the specified
    /// duration. Duration field must not be nil when the lock query type is
    /// `ByLockDuration`.
    #[prost(message, optional, tag = "3")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
    /// Timestamp is used by locks started before the specified duration.
    /// Timestamp field must not be nil when the lock query type is `ByLockTime`.
    /// Querying locks with timestamp is currently not implemented.
    #[prost(message, optional, tag = "4")]
    pub timestamp: ::core::option::Option<crate::shim::Timestamp>,
}
/// SyntheticLock is creating virtual lockup where new denom is combination of
/// original denom and synthetic suffix. At the time of synthetic lockup creation
/// and deletion, accumulation store is also being updated and on querier side,
/// they can query as freely as native lockup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.SyntheticLock")]
pub struct SyntheticLock {
    /// Underlying Lock ID is the underlying native lock's id for this synthetic
    /// lockup. A synthetic lock MUST have an underlying lock.
    #[prost(uint64, tag = "1")]
    #[serde(alias = "underlying_lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub underlying_lock_id: u64,
    /// SynthDenom is the synthetic denom that is a combination of
    /// gamm share + bonding status + validator address.
    #[prost(string, tag = "2")]
    pub synth_denom: ::prost::alloc::string::String,
    /// used for unbonding synthetic lockups, for active synthetic lockups, this
    /// value is set to uninitialized value
    #[prost(message, optional, tag = "3")]
    pub end_time: ::core::option::Option<crate::shim::Timestamp>,
    /// Duration is the duration for a synthetic lock to mature
    /// at the point of unbonding has started.
    #[prost(message, optional, tag = "4")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
}
/// LockQueryType defines the type of the lock query that can
/// either be by duration or start time of the lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
#[derive(::serde::Serialize, ::serde::Deserialize, ::schemars::JsonSchema)]
pub enum LockQueryType {
    ByDuration = 0,
    ByTime = 1,
    NoLock = 2,
}
impl LockQueryType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            LockQueryType::ByDuration => "ByDuration",
            LockQueryType::ByTime => "ByTime",
            LockQueryType::NoLock => "NoLock",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ByDuration" => Some(Self::ByDuration),
            "ByTime" => Some(Self::ByTime),
            "NoLock" => Some(Self::NoLock),
            _ => None,
        }
    }
}
/// GenesisState defines the lockup module's genesis state.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.GenesisState")]
pub struct GenesisState {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "last_lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub last_lock_id: u64,
    #[prost(message, repeated, tag = "2")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
    #[prost(message, repeated, tag = "3")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<SyntheticLock>,
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
#[proto_message(type_url = "/osmosis.lockup.Params")]
pub struct Params {
    #[prost(string, repeated, tag = "1")]
    pub force_unlock_allowed_addresses: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
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
#[proto_message(type_url = "/osmosis.lockup.ModuleBalanceRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/ModuleBalance",
    response_type = ModuleBalanceResponse
)]
pub struct ModuleBalanceRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.ModuleBalanceResponse")]
pub struct ModuleBalanceResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.ModuleLockedAmountRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/ModuleLockedAmount",
    response_type = ModuleLockedAmountResponse
)]
pub struct ModuleLockedAmountRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.ModuleLockedAmountResponse")]
pub struct ModuleLockedAmountResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockableCoinsRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountUnlockableCoins",
    response_type = AccountUnlockableCoinsResponse
)]
pub struct AccountUnlockableCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockableCoinsResponse")]
pub struct AccountUnlockableCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockingCoinsRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountUnlockingCoins",
    response_type = AccountUnlockingCoinsResponse
)]
pub struct AccountUnlockingCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockingCoinsResponse")]
pub struct AccountUnlockingCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedCoinsRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedCoins",
    response_type = AccountLockedCoinsResponse
)]
pub struct AccountLockedCoinsRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedCoinsResponse")]
pub struct AccountLockedCoinsResponse {
    #[prost(message, repeated, tag = "1")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedPastTime",
    response_type = AccountLockedPastTimeResponse
)]
pub struct AccountLockedPastTimeRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<crate::shim::Timestamp>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeResponse")]
pub struct AccountLockedPastTimeResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeNotUnlockingOnlyRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedPastTimeNotUnlockingOnly",
    response_type = AccountLockedPastTimeNotUnlockingOnlyResponse
)]
pub struct AccountLockedPastTimeNotUnlockingOnlyRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<crate::shim::Timestamp>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeNotUnlockingOnlyResponse")]
pub struct AccountLockedPastTimeNotUnlockingOnlyResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockedBeforeTimeRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountUnlockedBeforeTime",
    response_type = AccountUnlockedBeforeTimeResponse
)]
pub struct AccountUnlockedBeforeTimeRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<crate::shim::Timestamp>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountUnlockedBeforeTimeResponse")]
pub struct AccountUnlockedBeforeTimeResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeDenomRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedPastTimeDenom",
    response_type = AccountLockedPastTimeDenomResponse
)]
pub struct AccountLockedPastTimeDenomRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub timestamp: ::core::option::Option<crate::shim::Timestamp>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedPastTimeDenomResponse")]
pub struct AccountLockedPastTimeDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.LockedDenomRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/LockedDenom",
    response_type = LockedDenomResponse
)]
pub struct LockedDenomRequest {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
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
#[proto_message(type_url = "/osmosis.lockup.LockedDenomResponse")]
pub struct LockedDenomResponse {
    #[prost(string, tag = "1")]
    pub amount: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.LockedRequest")]
#[proto_query(path = "/osmosis.lockup.Query/LockedByID", response_type = LockedResponse)]
pub struct LockedRequest {
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
#[proto_message(type_url = "/osmosis.lockup.LockedResponse")]
pub struct LockedResponse {
    #[prost(message, optional, tag = "1")]
    pub lock: ::core::option::Option<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.LockRewardReceiverRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/LockRewardReceiver",
    response_type = LockRewardReceiverResponse
)]
pub struct LockRewardReceiverRequest {
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
#[proto_message(type_url = "/osmosis.lockup.LockRewardReceiverResponse")]
pub struct LockRewardReceiverResponse {
    #[prost(string, tag = "1")]
    pub reward_receiver: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.NextLockIDRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/NextLockID",
    response_type = NextLockIdResponse
)]
pub struct NextLockIdRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.NextLockIDResponse")]
pub struct NextLockIdResponse {
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
#[proto_message(type_url = "/osmosis.lockup.SyntheticLockupsByLockupIDRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/SyntheticLockupsByLockupID",
    response_type = SyntheticLockupsByLockupIdResponse
)]
#[deprecated]
pub struct SyntheticLockupsByLockupIdRequest {
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
#[proto_message(type_url = "/osmosis.lockup.SyntheticLockupsByLockupIDResponse")]
#[deprecated]
pub struct SyntheticLockupsByLockupIdResponse {
    #[prost(message, repeated, tag = "1")]
    pub synthetic_locks: ::prost::alloc::vec::Vec<SyntheticLock>,
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
#[proto_message(type_url = "/osmosis.lockup.SyntheticLockupByLockupIDRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/SyntheticLockupByLockupID",
    response_type = SyntheticLockupByLockupIdResponse
)]
pub struct SyntheticLockupByLockupIdRequest {
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
#[proto_message(type_url = "/osmosis.lockup.SyntheticLockupByLockupIDResponse")]
pub struct SyntheticLockupByLockupIdResponse {
    #[prost(message, optional, tag = "1")]
    pub synthetic_lock: ::core::option::Option<SyntheticLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedLongerDuration",
    response_type = AccountLockedLongerDurationResponse
)]
pub struct AccountLockedLongerDurationRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationResponse")]
pub struct AccountLockedLongerDurationResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedDurationRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedDuration",
    response_type = AccountLockedDurationResponse
)]
pub struct AccountLockedDurationRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedDurationResponse")]
pub struct AccountLockedDurationResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationNotUnlockingOnlyRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedLongerDurationNotUnlockingOnly",
    response_type = AccountLockedLongerDurationNotUnlockingOnlyResponse
)]
pub struct AccountLockedLongerDurationNotUnlockingOnlyRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationNotUnlockingOnlyResponse")]
pub struct AccountLockedLongerDurationNotUnlockingOnlyResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationDenomRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/AccountLockedLongerDurationDenom",
    response_type = AccountLockedLongerDurationDenomResponse
)]
pub struct AccountLockedLongerDurationDenomRequest {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
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
#[proto_message(type_url = "/osmosis.lockup.AccountLockedLongerDurationDenomResponse")]
pub struct AccountLockedLongerDurationDenomResponse {
    #[prost(message, repeated, tag = "1")]
    pub locks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.QueryParamsRequest")]
#[proto_query(
    path = "/osmosis.lockup.Query/Params",
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
#[proto_message(type_url = "/osmosis.lockup.QueryParamsResponse")]
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
#[proto_message(type_url = "/osmosis.lockup.MsgLockTokens")]
pub struct MsgLockTokens {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub duration: ::core::option::Option<crate::shim::Duration>,
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.MsgLockTokensResponse")]
pub struct MsgLockTokensResponse {
    #[prost(uint64, tag = "1")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
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
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlockingAll")]
pub struct MsgBeginUnlockingAll {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlockingAllResponse")]
pub struct MsgBeginUnlockingAllResponse {
    #[prost(message, repeated, tag = "1")]
    pub unlocks: ::prost::alloc::vec::Vec<PeriodLock>,
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
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlocking")]
pub struct MsgBeginUnlocking {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
    /// Amount of unlocking coins. Unlock all if not set.
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.MsgBeginUnlockingResponse")]
pub struct MsgBeginUnlockingResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "unlocking_lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub unlocking_lock_id: u64,
}
/// MsgExtendLockup extends the existing lockup's duration.
/// The new duration is longer than the original.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.MsgExtendLockup")]
pub struct MsgExtendLockup {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
    /// duration to be set. fails if lower than the current duration, or is
    /// unlocking
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
#[proto_message(type_url = "/osmosis.lockup.MsgExtendLockupResponse")]
pub struct MsgExtendLockupResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
/// MsgForceUnlock unlocks locks immediately for
/// addresses registered via governance.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(
    Clone,
    PartialEq,
    Eq,
    ::prost::Message,
    ::serde::Serialize,
    ::serde::Deserialize,
    ::schemars::JsonSchema,
    CosmwasmExt,
)]
#[proto_message(type_url = "/osmosis.lockup.MsgForceUnlock")]
pub struct MsgForceUnlock {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "ID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub id: u64,
    /// Amount of unlocking coins. Unlock all if not set.
    #[prost(message, repeated, tag = "3")]
    pub coins: ::prost::alloc::vec::Vec<super::super::cosmos::base::v1beta1::Coin>,
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
#[proto_message(type_url = "/osmosis.lockup.MsgForceUnlockResponse")]
pub struct MsgForceUnlockResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
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
#[proto_message(type_url = "/osmosis.lockup.MsgSetRewardReceiverAddress")]
pub struct MsgSetRewardReceiverAddress {
    #[prost(string, tag = "1")]
    pub owner: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    #[serde(alias = "lockID")]
    #[serde(
        serialize_with = "crate::serde::as_str::serialize",
        deserialize_with = "crate::serde::as_str::deserialize"
    )]
    pub lock_id: u64,
    #[prost(string, tag = "3")]
    pub reward_receiver: ::prost::alloc::string::String,
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
#[proto_message(type_url = "/osmosis.lockup.MsgSetRewardReceiverAddressResponse")]
pub struct MsgSetRewardReceiverAddressResponse {
    #[prost(bool, tag = "1")]
    pub success: bool,
}
pub struct LockupQuerier<'a, Q: cosmwasm_std::CustomQuery> {
    querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>,
}
impl<'a, Q: cosmwasm_std::CustomQuery> LockupQuerier<'a, Q> {
    pub fn new(querier: &'a cosmwasm_std::QuerierWrapper<'a, Q>) -> Self {
        Self { querier }
    }
    pub fn module_balance(&self) -> Result<ModuleBalanceResponse, cosmwasm_std::StdError> {
        ModuleBalanceRequest {}.query(self.querier)
    }
    pub fn module_locked_amount(
        &self,
    ) -> Result<ModuleLockedAmountResponse, cosmwasm_std::StdError> {
        ModuleLockedAmountRequest {}.query(self.querier)
    }
    pub fn account_unlockable_coins(
        &self,
        owner: ::prost::alloc::string::String,
    ) -> Result<AccountUnlockableCoinsResponse, cosmwasm_std::StdError> {
        AccountUnlockableCoinsRequest { owner }.query(self.querier)
    }
    pub fn account_unlocking_coins(
        &self,
        owner: ::prost::alloc::string::String,
    ) -> Result<AccountUnlockingCoinsResponse, cosmwasm_std::StdError> {
        AccountUnlockingCoinsRequest { owner }.query(self.querier)
    }
    pub fn account_locked_coins(
        &self,
        owner: ::prost::alloc::string::String,
    ) -> Result<AccountLockedCoinsResponse, cosmwasm_std::StdError> {
        AccountLockedCoinsRequest { owner }.query(self.querier)
    }
    pub fn account_locked_past_time(
        &self,
        owner: ::prost::alloc::string::String,
        timestamp: ::core::option::Option<crate::shim::Timestamp>,
    ) -> Result<AccountLockedPastTimeResponse, cosmwasm_std::StdError> {
        AccountLockedPastTimeRequest { owner, timestamp }.query(self.querier)
    }
    pub fn account_locked_past_time_not_unlocking_only(
        &self,
        owner: ::prost::alloc::string::String,
        timestamp: ::core::option::Option<crate::shim::Timestamp>,
    ) -> Result<AccountLockedPastTimeNotUnlockingOnlyResponse, cosmwasm_std::StdError> {
        AccountLockedPastTimeNotUnlockingOnlyRequest { owner, timestamp }.query(self.querier)
    }
    pub fn account_unlocked_before_time(
        &self,
        owner: ::prost::alloc::string::String,
        timestamp: ::core::option::Option<crate::shim::Timestamp>,
    ) -> Result<AccountUnlockedBeforeTimeResponse, cosmwasm_std::StdError> {
        AccountUnlockedBeforeTimeRequest { owner, timestamp }.query(self.querier)
    }
    pub fn account_locked_past_time_denom(
        &self,
        owner: ::prost::alloc::string::String,
        timestamp: ::core::option::Option<crate::shim::Timestamp>,
        denom: ::prost::alloc::string::String,
    ) -> Result<AccountLockedPastTimeDenomResponse, cosmwasm_std::StdError> {
        AccountLockedPastTimeDenomRequest {
            owner,
            timestamp,
            denom,
        }
        .query(self.querier)
    }
    pub fn locked_denom(
        &self,
        denom: ::prost::alloc::string::String,
        duration: ::core::option::Option<crate::shim::Duration>,
    ) -> Result<LockedDenomResponse, cosmwasm_std::StdError> {
        LockedDenomRequest { denom, duration }.query(self.querier)
    }
    pub fn locked_by_id(&self, lock_id: u64) -> Result<LockedResponse, cosmwasm_std::StdError> {
        LockedRequest { lock_id }.query(self.querier)
    }
    pub fn lock_reward_receiver(
        &self,
        lock_id: u64,
    ) -> Result<LockRewardReceiverResponse, cosmwasm_std::StdError> {
        LockRewardReceiverRequest { lock_id }.query(self.querier)
    }
    pub fn next_lock_id(&self) -> Result<NextLockIdResponse, cosmwasm_std::StdError> {
        NextLockIdRequest {}.query(self.querier)
    }
    #[deprecated]
    pub fn synthetic_lockups_by_lockup_id(
        &self,
        lock_id: u64,
    ) -> Result<SyntheticLockupsByLockupIdResponse, cosmwasm_std::StdError> {
        SyntheticLockupsByLockupIdRequest { lock_id }.query(self.querier)
    }
    pub fn synthetic_lockup_by_lockup_id(
        &self,
        lock_id: u64,
    ) -> Result<SyntheticLockupByLockupIdResponse, cosmwasm_std::StdError> {
        SyntheticLockupByLockupIdRequest { lock_id }.query(self.querier)
    }
    pub fn account_locked_longer_duration(
        &self,
        owner: ::prost::alloc::string::String,
        duration: ::core::option::Option<crate::shim::Duration>,
    ) -> Result<AccountLockedLongerDurationResponse, cosmwasm_std::StdError> {
        AccountLockedLongerDurationRequest { owner, duration }.query(self.querier)
    }
    pub fn account_locked_duration(
        &self,
        owner: ::prost::alloc::string::String,
        duration: ::core::option::Option<crate::shim::Duration>,
    ) -> Result<AccountLockedDurationResponse, cosmwasm_std::StdError> {
        AccountLockedDurationRequest { owner, duration }.query(self.querier)
    }
    pub fn account_locked_longer_duration_not_unlocking_only(
        &self,
        owner: ::prost::alloc::string::String,
        duration: ::core::option::Option<crate::shim::Duration>,
    ) -> Result<AccountLockedLongerDurationNotUnlockingOnlyResponse, cosmwasm_std::StdError> {
        AccountLockedLongerDurationNotUnlockingOnlyRequest { owner, duration }.query(self.querier)
    }
    pub fn account_locked_longer_duration_denom(
        &self,
        owner: ::prost::alloc::string::String,
        duration: ::core::option::Option<crate::shim::Duration>,
        denom: ::prost::alloc::string::String,
    ) -> Result<AccountLockedLongerDurationDenomResponse, cosmwasm_std::StdError> {
        AccountLockedLongerDurationDenomRequest {
            owner,
            duration,
            denom,
        }
        .query(self.querier)
    }
    pub fn params(&self) -> Result<QueryParamsResponse, cosmwasm_std::StdError> {
        QueryParamsRequest {}.query(self.querier)
    }
}
