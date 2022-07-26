use std::collections::HashSet;
use std::marker::PhantomData;

use abstract_sdk::common_namespace::BASE_STATE_KEY;
use abstract_sdk::memory::Memory;

use cosmwasm_std::{Addr, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const TRADER_NAMESPACE: &str = "traders";

/// The state variables for our ApiContract.
pub struct ApiContract<'a, T: Serialize + DeserializeOwned> {
    // Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'a, Addr, HashSet<Addr>>,
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, ApiState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,

    pub request_destination: Addr,
    _phantom_data: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned> Default for ApiContract<'a, T> {
    fn default() -> Self {
        Self::new(BASE_STATE_KEY, TRADER_NAMESPACE, Addr::unchecked(""))
    }
}

/// Constructor
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub(crate) fn new(
        base_state_key: &'a str,
        traders_namespace: &'a str,
        proxy_address: Addr,
    ) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(base_state_key),
            traders: Map::new(traders_namespace),
            request_destination: proxy_address,
            _phantom_data: PhantomData,
        }
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
    pub api_dependencies: Vec<String>,
}
