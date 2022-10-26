use std::{collections::HashSet, marker::PhantomData};

use abstract_os::version_control::Core;
use abstract_sdk::{memory::Memory, IbcCallbackHandlerFn, ReceiveHandlerFn, BASE_STATE};

use cosmwasm_std::{Addr, Empty, StdResult, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::ApiError;

pub const TRADER_NAMESPACE: &str = "traders";

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
}
/// The state variables for our ApiContract.
pub struct ApiContract<
    'a,
    Request: Serialize + DeserializeOwned,
    Error: From<cosmwasm_std::StdError> + From<ApiError>,
    Receive: Serialize + DeserializeOwned = Empty,
> {
    // Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'a, Addr, HashSet<Addr>>,
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, ApiState>,
    /// Stores the API version
    pub version: Item<'a, ContractVersion>,
    pub target_os: Option<Core>,
    pub dependencies: &'static [&'static str],

    pub(crate) ibc_callbacks: &'a [(&'static str, IbcCallbackHandlerFn<Self, Error>)],
    pub(crate) receive_handler: Option<ReceiveHandlerFn<Self, Receive, Error>>,

    _phantom_data_request: PhantomData<Request>,
    _phantom_data_error: PhantomData<Error>,
    _phantom_data_receive: PhantomData<Receive>,
}

/// Constructor
impl<
        'a,
        T: Serialize + DeserializeOwned,
        R: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<ApiError>,
    > ApiContract<'a, T, E, R>
{
    pub const fn new() -> Self {
        Self {
            version: CONTRACT,
            base_state: Item::new(BASE_STATE),
            traders: Map::new(TRADER_NAMESPACE),
            target_os: None,
            ibc_callbacks: &[],
            dependencies: &[],
            receive_handler: None,
            _phantom_data_request: PhantomData,
            _phantom_data_receive: PhantomData,
            _phantom_data_error: PhantomData,
        }
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'a [(&'static str, IbcCallbackHandlerFn<Self, E>)],
    ) -> Self {
        self.ibc_callbacks = callbacks;
        self
    }

    pub const fn with_receive(mut self, receive_handler: ReceiveHandlerFn<Self, R, E>) -> Self {
        self.receive_handler = Some(receive_handler);
        self
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [&'static str]) -> Self {
        self.dependencies = dependencies;
        self
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
