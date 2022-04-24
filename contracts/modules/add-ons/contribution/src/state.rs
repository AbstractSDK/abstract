use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item, Map};
use pandora_os::{
    modules::add_ons::contribution::Compensation, util::paged_map::PagedMap,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub subscription_contract: Addr,
    pub ratio: Decimal,
    pub project_token: Addr,
    pub payment_asset: AssetInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// max allowed tokens to be distributed
    pub emissions_cap: Uint128,
    /// Target income to pay base salaries
    pub target: Uint64,
    /// expense the org is able to make based on the income and target
    pub expense: Uint64,
    /// total weights for token emission allocations
    pub total_weight: Uint128,
    /// ratio of income/target
    pub expense_ratio: Decimal,
    /// time of next payout
    pub next_pay_day: Uint64,
}
pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

#[derive(Clone, Debug, PartialEq,Serialize, Deserialize)]
pub struct Accumulator {
    pub contributors_to_retire: Vec<String>,
}

// List contributors
pub const CONTRIBUTORS: PagedMap<Compensation, Accumulator> = PagedMap::new("contributors","status");
