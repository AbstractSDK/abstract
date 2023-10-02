use abstract_core::{
    objects::{time_weighted_average::TimeWeightedAverage, AccountId},
    AbstractResult,
};
use cosmwasm_std::{Addr, Api, Decimal};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_storage_plus::{Item, Map};

// #### SUBSCRIPTION SECTION ####

/// Setting for protocol token emissions
#[cosmwasm_schema::cw_serde]
pub enum UncheckedEmissionType {
    None,
    /// A fixed number of tokens are distributed to users on a per-block basis.
    /// emission = block_shared / total_subscribers
    BlockShared(Decimal, AssetInfoUnchecked),
    /// Each user receives a fixed number of tokens on a per-block basis.
    /// emission = block_per_user
    BlockPerUser(Decimal, AssetInfoUnchecked),
    /// Requires contribution functionality to be active
    /// Emissions will be based on protocol income and user/contributor split.
    /// See [`ContributionConfig`]
    IncomeBased(AssetInfoUnchecked),
}

impl UncheckedEmissionType {
    pub fn check(self, api: &dyn Api) -> AbstractResult<EmissionType> {
        match self {
            UncheckedEmissionType::None => Ok(EmissionType::None),
            UncheckedEmissionType::BlockShared(d, a) => {
                Ok(EmissionType::BlockShared(d, a.check(api, None)?))
            }
            UncheckedEmissionType::BlockPerUser(d, a) => {
                Ok(EmissionType::BlockPerUser(d, a.check(api, None)?))
            }
            UncheckedEmissionType::IncomeBased(a) => {
                Ok(EmissionType::IncomeBased(a.check(api, None)?))
            }
        }
    }
}

/// Setting for protocol token emissions
#[cosmwasm_schema::cw_serde]
pub enum EmissionType {
    None,
    /// emission = block_shared / total_subs
    BlockShared(Decimal, AssetInfo),
    /// emission = block_per_user
    BlockPerUser(Decimal, AssetInfo),
    /// Requires contribution functionality to be active
    IncomeBased(AssetInfo),
}

/// Config for subscriber functionality
#[cosmwasm_schema::cw_serde]
pub struct SubscriptionConfig {
    /// Only addr that can register on OS
    pub factory_address: Addr,
    /// Asset that's accepted as payment
    pub payment_asset: AssetInfo,
    /// Cost of the subscription on a per-block basis.
    pub subscription_cost_per_block: Decimal,
    /// Subscription emissions per block
    pub subscription_per_block_emissions: EmissionType,
    /// If contributors contract enabled
    pub contributors_enabled: bool,
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
    pub expiration_block: u64,
    /// last time emissions were claimed
    pub last_emission_claim_block: u64,
    /// Address of the OS manager
    pub manager_addr: Addr,
}

/// Average number of subscribers
pub const SUBSCRIPTION_CONFIG: Item<SubscriptionConfig> = Item::new("\u{0}{10}sub_config");
pub const SUBSCRIPTION_STATE: Item<SubscriptionState> = Item::new("\u{0}{9}sub_state");
pub const SUBSCRIBERS: Map<&AccountId, Subscriber> = Map::new("subscribed");
pub const DORMANT_SUBSCRIBERS: Map<&AccountId, Subscriber> = Map::new("un-subscribed");

pub const INCOME_TWA: TimeWeightedAverage = TimeWeightedAverage::new("\u{0}{7}sub_twa");
