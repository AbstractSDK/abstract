use cosmwasm_std::{Addr, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use abstract_sdk::common_namespace::{ADMIN_KEY, BASE_STATE_KEY};
use abstract_sdk::memory::Memory;

/// The state variables for our AddOnContract.
pub struct AddOnContract<'a> {
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, AddOnState>,
    pub version: Item<'a, ContractVersion>,
    pub admin: Admin<'a>,
}

impl Default for AddOnContract<'static> {
    fn default() -> Self {
        Self::new(BASE_STATE_KEY, ADMIN_KEY)
    }
}

/// Constructor
impl<'a> AddOnContract<'a> {
    fn new(base_state_key: &'a str, admin_key: &'a str) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(base_state_key),
            admin: Admin::new(admin_key),
        }
    }
    pub fn state(&self, store: &dyn Storage) -> StdResult<AddOnState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddOnState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
