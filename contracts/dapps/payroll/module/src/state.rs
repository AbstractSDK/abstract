use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use pandora_os::{util::{deposit_manager::Deposit, paged_map::PagedMap}, dapps::payout::Compensation};
use terraswap::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub ratio: Decimal,
    pub payment_asset: AssetInfo,
    pub subscription_cost: Uint64,
    pub project_token: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// max allowed tokens to be distributed
    pub token_cap: Uint128,
    /// Total income for the last month
    pub income: Uint64,
    /// Target income to pay base salaries
    pub target: Uint64,
    /// expense the org is able to make based on the income and target
    pub expense: Uint64,
    /// total weights for token emission allocations
    pub total_weight: Uint128,
    /// The time when contributor claims can be performed
    pub next_pay_day: Uint64,
    /// all os_ids of clients that didn't pay
    pub debtors: Vec<u32>,
    /// ratio of income/target
    pub expense_ratio: Decimal,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct IncomeAccumulator {
    pub income: u32,
    pub debtors: Vec<u32>,
}

pub const MONTH: u64 = 60 * 60 * 24 * 30;
pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
pub const STATE: Item<State> = Item::new("\u{0}{5}state");

// List clients
pub const CLIENTS: PagedMap<Deposit, IncomeAccumulator> =
    PagedMap::new("clients", "clients_status");
// List contributors
pub const CONTRIBUTORS: Map<&str, Compensation> = Map::new("contributors");
