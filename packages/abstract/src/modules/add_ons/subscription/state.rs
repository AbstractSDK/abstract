use std::ops::Sub;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::util::{deposit_manager::Deposit, paged_map::PagedMap};
use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Uint128, Uint64};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map, U32Key};

pub const MONTH: u64 = 60 * 60 * 24 * 30;
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionConfig {
    pub version_control_address: Addr,
    pub factory_address: Addr,
    pub payment_asset: AssetInfo,
    pub subscription_cost: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionState {
    /// Total income for the last month
    pub income: Uint64,
    /// amount of active subscribers
    pub active_subs: u32,
    /// Is the income collected?
    pub collected: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Subscriber {
    pub balance: Deposit,
    pub claimed_emissions: bool,
    pub manager_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct IncomeAccumulator {
    pub income: u32,
    pub active_subs: u32,
    pub debtors: Vec<u32>,
}

pub const SUB_CONFIG: Item<SubscriptionConfig> = Item::new("\u{0}{10}sub_config");
pub const SUB_STATE: Item<SubscriptionState> = Item::new("\u{0}{9}sub_state");

pub const CLIENTS: PagedMap<Subscriber, IncomeAccumulator> =
    PagedMap::new("clients", "clients_status");
pub const DORMANT_CLIENTS: Map<U32Key, Subscriber> = Map::new("dormant_clients");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributionConfig {
    /// Percentage of income that is redirected to the protocol
    pub protocol_income_share: Decimal,
    /// Percentage of emissions allocated to users
    pub emission_user_share: Decimal,
    /// Max emissions (when income = 0) = max_emissions_multiple * floor_emissions
    pub max_emissions_multiple: Decimal,
    /// Token address of the emitted token
    pub project_token: Addr,
    /// Emissions amplification factor in inverse emissions <-> target equation
    pub emissions_amp_factor: Uint128,
    /// Emissions offset factor in inverse emissions <-> target equation
    pub emissions_offset: Uint128,
    /// Denom of base payment to contributors
    pub base_denom: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributionState {
    /// Target income to pay base salaries
    pub target: Uint64,
    /// expense the org is able to make based on the income, target and splitS
    pub expense: Uint64,
    /// total weights for token emission allocations
    pub total_weight: Uint128,
    /// total emissions for this month
    pub emissions: Uint128,
    /// time of next payout
    pub next_pay_day: Uint64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ContributorAccumulator {
    pub contributors_to_retire: Vec<String>,
}
// List contributors
pub const CONTRIBUTORS: PagedMap<Compensation, ContributorAccumulator> =
    PagedMap::new("contributors", "status");
pub const CON_CONFIG: Item<ContributionConfig> = Item::new("\u{0}{10}con_config");
pub const CON_STATE: Item<ContributionState> = Item::new("\u{0}{9}con_state");

pub struct ContributorContext {
    /// Total token emissions weight
    pub total_weight: u128,
    /// Total emissions going to contributors
    pub contributor_emissions: u64,
    /// Base salary payout % ( Income / Target ), max 100%
    pub payout_ratio: Decimal,
    /// Block time at execution
    pub block_time: u64,
    pub next_pay_day: u64,
    pub base_denom: String,
    pub token_address: String,
    pub proxy_address: String,
}

pub struct SubscriberContext {
    pub subscription_cost: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Compensation {
    pub base: u32,
    pub weight: u32,
    pub next_pay_day: Uint64,
    pub expiration: Uint64,
}

impl Sub for Compensation {
    type Output = (i32, i32);

    fn sub(self, other: Self) -> (i32, i32) {
        (
            self.base as i32 - other.base as i32,
            self.weight as i32 - other.weight as i32,
        )
    }
}
