use abstract_core::{
    objects::{core::AccountId, time_weighted_average::TimeWeightedAverage},
    AbstractResult,
};
use cosmwasm_std::{Addr, Api, Decimal, StdError, StdResult, Uint128, Uint64};
use cw_asset::{AssetInfo, AssetInfoUnchecked};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::ops::Sub;

// #### SUBSCRIPTION SECTION ####

/// Setting for protocol token emissions
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionConfig {
    /// Used to verify OS and get the proxy
    pub version_control_address: Addr,
    /// Only addr that can register on OS
    pub factory_address: Addr,
    /// Asset that's accepted as payment
    pub payment_asset: AssetInfo,
    /// Cost of the subscription on a per-block basis.
    pub subscription_cost_per_block: Decimal,
    /// Subscription emissions per block
    pub subscription_per_block_emissions: EmissionType,
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
pub const INCOME_TWA: TimeWeightedAverage = TimeWeightedAverage::new("\u{0}{7}sub_twa");
pub const SUBSCRIPTION_CONFIG: Item<SubscriptionConfig> = Item::new("\u{0}{10}sub_config");
pub const SUBSCRIPTION_STATE: Item<SubscriptionState> = Item::new("\u{0}{9}sub_state");
pub const SUBSCRIBERS: Map<AccountId, Subscriber> = Map::new("subscribed");
pub const DORMANT_SUBSCRIBERS: Map<AccountId, Subscriber> = Map::new("un-subscribed");

// #### CONTRIBUTION SECTION ####

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributionConfig {
    /// Percentage of income that is redirected to the protocol
    pub protocol_income_share: Decimal,
    /// Percentage of emissions allocated to users
    pub emission_user_share: Decimal,
    /// Max emissions (when income = 0) = max_emissions_multiple * floor_emissions
    pub max_emissions_multiple: Decimal,
    /// Emissions amplification factor in inverse emissions <-> target equation
    pub emissions_amp_factor: Uint128,
    /// Emissions offset factor in inverse emissions <-> target equation
    pub emissions_offset: Uint128,
    /// token
    pub token_info: AssetInfo,
}

impl ContributionConfig {
    pub fn verify(self) -> StdResult<Self> {
        if !(decimal_is_percentage(&self.protocol_income_share)
            || decimal_is_percentage(&self.emission_user_share))
        {
            Err(StdError::generic_err(
                "Some config fields should not be >1.",
            ))
        } else {
            Ok(self)
        }
    }
}

fn decimal_is_percentage(decimal: &Decimal) -> bool {
    decimal <= &Decimal::one()
}

#[cosmwasm_schema::cw_serde]
pub struct ContributionState {
    /// Target income to pay base salaries
    pub income_target: Decimal,
    /// expense the org is able to make based on the income, target and split
    pub expense: Decimal,
    /// total weights for token emission allocations
    pub total_weight: Uint128,
    /// total emissions for this month
    pub emissions: Decimal,
}

// List contributors
pub const CONTRIBUTORS: Map<&Addr, Compensation> = Map::new("contributors");
pub const CONTRIBUTION_CONFIG: Item<ContributionConfig> = Item::new("\u{0}{10}con_config");
pub const CACHED_CONTRIBUTION_STATE: Item<ContributionState> =
    Item::new("\u{0}{15}cache_con_state");
pub const CONTRIBUTION_STATE: Item<ContributionState> = Item::new("\u{0}{9}con_state");

/// Compensation details for contributors
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema, Default)]
pub struct Compensation {
    pub base_per_block: Decimal,
    pub weight: u32,
    pub last_claim_block: Uint64,
    pub expiration_block: Uint64,
}

impl Compensation {
    pub fn overwrite(
        mut self,
        base_per_block: Option<Decimal>,
        weight: Option<u32>,
        expiration_block: Option<u64>,
    ) -> Self {
        if let Some(base_per_block) = base_per_block {
            self.base_per_block = base_per_block;
        }

        if let Some(weight) = weight {
            self.weight = weight;
        }

        if let Some(expiration_block) = expiration_block {
            self.expiration_block = expiration_block.into();
        }
        self
    }
}

impl Sub for Compensation {
    type Output = (Decimal, i32);

    fn sub(self, other: Self) -> (Decimal, i32) {
        (
            self.base_per_block - other.base_per_block,
            self.weight as i32 - other.weight as i32,
        )
    }
}
