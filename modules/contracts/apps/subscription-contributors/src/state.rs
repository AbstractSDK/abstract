use abstract_core::objects::AccountId;
use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Timestamp, Uint128};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};

pub const CONTRIBUTION_CONFIG: Item<ContributorsConfig> = Item::new("config");

use std::ops::Sub;

// List contributors
pub const CONTRIBUTORS: Map<&Addr, Compensation> = Map::new("contributors");
pub const CACHED_CONTRIBUTION_STATE: Item<ContributionState> = Item::new("cache_state");
pub const CONTRIBUTION_STATE: Item<ContributionState> = Item::new("state");

// Temporary AccountId
pub const COMPENSATION_CLAIMER: Item<AccountId> = Item::new("claimer");

/// Compensation details for contributors
#[cosmwasm_schema::cw_serde]
#[derive(Default)]
pub struct Compensation {
    pub base_per_week: Decimal,
    pub weight: u32,
    pub last_claim_timestamp: Timestamp,
    pub expiration_timestamp: Timestamp,
}

impl Compensation {
    pub fn overwrite(
        mut self,
        base_per_week: Option<Decimal>,
        weight: Option<u32>,
        expiration_timestamp: Option<Timestamp>,
    ) -> Self {
        if let Some(base_per_week) = base_per_week {
            self.base_per_week = base_per_week;
        }

        if let Some(weight) = weight {
            self.weight = weight;
        }

        if let Some(expiration_block) = expiration_timestamp {
            self.expiration_timestamp = expiration_block;
        }
        self
    }
}

impl Sub for Compensation {
    type Output = (Decimal, i32);

    fn sub(self, other: Self) -> (Decimal, i32) {
        (
            self.base_per_week - other.base_per_week,
            self.weight as i32 - other.weight as i32,
        )
    }
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

#[cosmwasm_schema::cw_serde]
pub struct ContributorsConfig {
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

impl ContributorsConfig {
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
