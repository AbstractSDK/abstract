use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint64};
use cw_asset::AssetInfo;
use cw_storage_plus::{Item};
use crate::util::{deposit_manager::Deposit, paged_map::PagedMap};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub version_control_address: Addr,
    pub payment_asset: AssetInfo,
    pub subscription_cost: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// Total income for the last month
    pub income: Uint64,
    /// The time when contributor claims can be performed
    pub next_pay_day: Uint64,
    /// all os_ids of clients that didn't pay
    pub debtors: Vec<u32>,
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
