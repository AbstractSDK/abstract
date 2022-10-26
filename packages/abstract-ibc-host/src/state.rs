use std::marker::PhantomData;

use abstract_sdk::{memory::Memory, BASE_STATE};

use cosmwasm_std::{Addr, Binary, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const TRADER_NAMESPACE: &str = "traders";

pub const PENDING: Item<(String, u32)> = Item::new("pending");
/// (channel-id,os_id) -> remote_addr
pub const ACCOUNTS: Map<(&str, u32), Addr> = Map::new("accounts");
pub const CLOSED_CHANNELS: Item<Vec<String>> = Item::new("closed");
// this stores all results from current dispatch
pub const RESULTS: Item<Vec<Binary>> = Item::new("results");

/// The state variables for our host contract.
pub struct Host<'a, T: Serialize + DeserializeOwned> {
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, HostState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,
    /// Signal the expected execute message struct
    _phantom_data: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned> Default for Host<'a, T> {
    fn default() -> Self {
        Self::new(&[])
    }
}

/// Constructor
impl<'a, T: Serialize + DeserializeOwned> Host<'a, T> {
    pub const fn new(_dependencies: &'static [&'static str]) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(BASE_STATE),
            _phantom_data: PhantomData,
        }
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<HostState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HostState {
    /// Memory contract struct (address)
    pub memory: Memory,
    /// code id for Stargate proxy contract
    pub cw1_code_id: u64,
    /// Chain identifier
    pub chain: String,
}
