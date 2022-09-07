use std::collections::HashSet;
use std::marker::PhantomData;

use abstract_os::version_control::Core;
use abstract_sdk::common_namespace::BASE_STATE_KEY;
use abstract_sdk::memory::Memory;

use cosmwasm_std::{Addr, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::ApiError;

pub const TRADER_NAMESPACE: &str = "traders";

/// The state variables for our ApiContract.
pub struct ApiContract<'a, T: Serialize + DeserializeOwned> {
    // Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'a, Addr, HashSet<Addr>>,
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, ApiState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,

    pub dependencies: &'static [&'static str],

    pub target_os: Option<Core>,
    _phantom_data: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned> Default for ApiContract<'a, T> {
    fn default() -> Self {
        Self::new(&[])
    }
}

/// Constructor
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub const fn new(dependencies: &'static [&'static str]) -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(BASE_STATE_KEY),
            traders: Map::new(TRADER_NAMESPACE),
            target_os: None,
            dependencies,
            _phantom_data: PhantomData,
        }
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }

    pub fn target(&self) -> Result<&Addr, ApiError> {
        Ok(&self
            .target_os
            .as_ref()
            .ok_or(ApiError::NoTargetOS {})?
            .proxy)
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
