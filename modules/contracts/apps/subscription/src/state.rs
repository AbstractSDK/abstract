use abstract_core::{
    objects::{time_weighted_average::TimeWeightedAverage, AccountId},
    AbstractResult,
};
use cosmwasm_std::{Addr, Api, Decimal, Timestamp};
use cw_address_like::AddressLike;
use cw_asset::{AssetInfo, AssetInfoBase};
use cw_storage_plus::{Item, Map};

// #### SUBSCRIPTION SECTION ####

/// Setting for protocol token emissions
#[cosmwasm_schema::cw_serde]
pub enum EmissionType<T: AddressLike> {
    None,
    /// A fixed number of tokens are distributed to users on a per-week basis.
    /// emission = week_shared / total_subscribers
    WeekShared(Decimal, AssetInfoBase<T>),
    /// Each user receives a fixed number of tokens on a per-week basis.
    /// emission = week_per_user
    WeekPerUser(Decimal, AssetInfoBase<T>),
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
    /// Only addr that can register on OS
    pub factory_address: Addr,
    /// Asset that's accepted as payment
    pub payment_asset: AssetInfo,
    /// Cost of the subscription on a per-week basis.
    pub subscription_cost_per_week: Decimal,
    /// Subscription emissions per week
    pub subscription_per_week_emissions: EmissionType<Addr>,
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
    /// Address of the OS manager
    pub manager_addr: Addr,
}

/// Average number of subscribers
pub const SUBSCRIPTION_CONFIG: Item<SubscriptionConfig> = Item::new("\u{0}{10}sub_config");
pub const SUBSCRIPTION_STATE: Item<SubscriptionState> = Item::new("\u{0}{9}sub_state");
pub const SUBSCRIBERS: Map<&AccountId, Subscriber> = Map::new("subscribed");
pub const DORMANT_SUBSCRIBERS: Map<&AccountId, Subscriber> = Map::new("un-subscribed");

pub const INCOME_TWA: TimeWeightedAverage = TimeWeightedAverage::new("\u{0}{7}sub_twa");
