use std::collections::HashSet;

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub use crate::objects::account::ACCOUNT_ID;
use crate::objects::{common_namespace::ADMIN_NAMESPACE, module::ModuleId};

pub type SuspensionStatus = bool;

/// Manager configuration
#[cosmwasm_schema::cw_serde]
pub struct Config {
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}

/// Abstract Account details.
#[cosmwasm_schema::cw_serde]
pub struct AccountInfo {
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

pub mod namespace {
    pub const SUSPENSION_STATUS: &str = "a";
    pub const CONFIG: &str = "b";
    pub const INFO: &str = "c";
    pub const ACCOUNT_MODULES: &str = "d";
    pub const DEPENDENTS: &str = "e";
    pub const SUB_ACCOUNTS: &str = "f";
}

pub const STATE: Item<State> = Item::new("a");
pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);

/// Suspension status
pub const SUSPENSION_STATUS: Item<SuspensionStatus> = Item::new(namespace::SUSPENSION_STATUS);
/// Configuration
pub const CONFIG: Item<Config> = Item::new(namespace::CONFIG);
/// Info about the Account
pub const INFO: Item<AccountInfo> = Item::new(namespace::INFO);
/// Enabled Abstract modules
pub const ACCOUNT_MODULES: Map<ModuleId, Addr> = Map::new(namespace::ACCOUNT_MODULES);
/// Stores the dependency relationship between modules
/// map module -> modules that depend on module.
pub const DEPENDENTS: Map<ModuleId, HashSet<String>> = Map::new(namespace::DEPENDENTS);
/// List of sub-accounts
pub const SUB_ACCOUNTS: Map<u32, cosmwasm_std::Empty> = Map::new(namespace::SUB_ACCOUNTS);
// Additional states, not listed here: cw_gov_ownable::GovOwnership

#[cosmwasm_schema::cw_serde]
pub struct State {
    pub modules: Vec<Addr>,
}
