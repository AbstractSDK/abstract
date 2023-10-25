use abstract_core::{objects::time_weighted_average::TimeWeightedAverage, AbstractResult};
use cosmwasm_std::{Addr, Api, BlockInfo, Decimal256, Timestamp};
use cw_address_like::AddressLike;
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_storage_plus::{Item, Map};

/// Setting for protocol token emissions
#[cosmwasm_schema::cw_serde]
pub enum EmissionType<T: AddressLike> {
    None,
    /// A fixed number of tokens are distributed to users on a per-week basis.
    /// emission = week_shared / total_subscribers
    WeekShared(Decimal256, AssetInfoBase<T>),
    /// Each user receives a fixed number of tokens on a per-week basis.
    /// emission = week_per_user
    WeekPerUser(Decimal256, AssetInfoBase<T>),
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
            EmissionType::WeekShared(d, a) => Ok(EmissionType::WeekShared(d, a.check(api, None)?)),
            EmissionType::WeekPerUser(d, a) => {
                Ok(EmissionType::WeekPerUser(d, a.check(api, None)?))
            } // EmissionType::IncomeBased(a) => Ok(EmissionType::IncomeBased(a.check(api, None)?)),
        }
    }
}

/// Config for subscriber functionality
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionConfig {
    /// Asset that's accepted as payment
    pub payment_asset: AssetInfo,
    /// Cost of the subscription on a per-week basis.
    pub subscription_cost_per_week: Decimal256,
    /// Subscription emissions per week
    pub subscription_per_week_emissions: EmissionType<Addr>,
    /// Unsubscription hook addr
    pub unsubscription_hook_addr: Option<Addr>,
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
    pub fn new(block: &BlockInfo, paid_for_weeks: u64) -> Self {
        Self {
            expiration_timestamp: block.time.plus_days(paid_for_weeks * 7),
            last_emission_claim_timestamp: block.time,
        }
    }

    pub fn extend(&mut self, paid_for_weeks: u64) {
        self.expiration_timestamp = self.expiration_timestamp.plus_days(paid_for_weeks * 7)
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
