use abstract_std::{objects::time_weighted_average::TimeWeightedAverage, AbstractResult};
use cosmwasm_std::{Addr, Api, BlockInfo, Decimal, Timestamp};
use cw_address_like::AddressLike;
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_storage_plus::{Item, Map};

/// Setting for protocol token emissions
#[cosmwasm_schema::cw_serde]
pub enum EmissionType<T: AddressLike> {
    None,
    /// A fixed number of tokens are distributed to users on a per-second basis.
    /// emission = second_shared / total_subscribers
    SecondShared(Decimal, AssetInfoBase<T>),
    /// Each user receives a fixed number of tokens on a per-second basis.
    /// emission = second_per_user
    SecondPerUser(Decimal, AssetInfoBase<T>),
    // TODO: subscription-contribution
    // /// Requires contribution functionality to be active
    // /// Emissions will be based on protocol income and user/contributor split.
    // /// See [`ContributionConfig`]
    // IncomeBased(AssetInfoBase<T>),
}

impl EmissionType<String> {
    pub fn check(self, api: &dyn Api) -> AbstractResult<EmissionType<Addr>> {
        match self {
            EmissionType::None => Ok(EmissionType::None),
            EmissionType::SecondShared(d, a) => {
                Ok(EmissionType::SecondShared(d, a.check(api, None)?))
            }
            EmissionType::SecondPerUser(d, a) => {
                Ok(EmissionType::SecondPerUser(d, a.check(api, None)?))
            } // EmissionType::IncomeBased(a) => Ok(EmissionType::IncomeBased(a.check(api, None)?)),
        }
    }
}

/// Config for subscriber functionality
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionConfig {
    /// Asset that's accepted as payment
    pub payment_asset: AssetInfo,
    /// Cost of the subscription on a per-second basis.
    pub subscription_cost_per_second: Decimal,
    /// Subscription emissions per second
    pub subscription_per_second_emissions: EmissionType<Addr>,
    /// Unsubscription hook addr
    pub unsubscribe_hook_addr: Option<Addr>,
}

/// Keeps track of the active subscribers.
/// Is updated each time a sub joins/leaves
/// Used to calculate income.
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionState {
    /// amount of active subscribers
    pub active_subs: u32,
}

/// Stored info for each subscriber.
#[cosmwasm_schema::cw_serde]
pub struct Subscriber {
    /// When the subscription ends
    pub expiration_timestamp: Timestamp,
    /// last time emissions were claimed
    pub last_emission_claim_timestamp: Timestamp,
}

impl Subscriber {
    pub fn new(block: &BlockInfo, paid_for_seconds: u64) -> Self {
        Self {
            expiration_timestamp: block.time.plus_seconds(paid_for_seconds),
            last_emission_claim_timestamp: block.time,
        }
    }

    pub fn extend(&mut self, paid_for_seconds: u64) {
        self.expiration_timestamp = self.expiration_timestamp.plus_seconds(paid_for_seconds)
    }

    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        block.time >= self.expiration_timestamp
    }
}

/// Average number of subscribers
pub const SUBSCRIPTION_CONFIG: Item<SubscriptionConfig> = Item::new("config");
pub const SUBSCRIPTION_STATE: Item<SubscriptionState> = Item::new("state");
pub const SUBSCRIBERS: Map<&Addr, Subscriber> = Map::new("subs");
pub const EXPIRED_SUBSCRIBERS: Map<&Addr, Subscriber> = Map::new("unsubs");

pub const INCOME_TWA: TimeWeightedAverage = TimeWeightedAverage::new("twa");
