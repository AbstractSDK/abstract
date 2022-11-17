use std::{collections::HashSet, fmt::Debug};

use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
        QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
    },
    feature_objects::AnsHost,
    namespaces::BASE_STATE,
    os::version_control::Core,
};

use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage};

use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ExtensionError;

pub const TRADER_NAMESPACE: &str = "traders";

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExtensionState {
    /// Used to verify requests
    pub version_control: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}
/// The state variables for our ExtensionContract.
pub struct ExtensionContract<
    Error: From<cosmwasm_std::StdError> + From<ExtensionError> + 'static,
    CustomExecMsg: 'static = Empty,
    CustomInitMsg: 'static = Empty,
    CustomQueryMsg: 'static = Empty,
    Receive: 'static = Empty,
> {
    pub(crate) contract:
        AbstractContract<Self, Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, Empty, Receive>,
    pub(crate) base_state: Item<'static, ExtensionState>,
    // Map ProxyAddr -> WhitelistedTraders
    pub traders: Map<'static, Addr, HashSet<Addr>>,
    // Every DApp should use the provided ans_host contract for token/contract address resolution
    /// Stores the extension version
    pub target_os: Option<Core>,
}

/// Constructor
impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self {
            contract: AbstractContract::new(name, version),
            base_state: Item::new(BASE_STATE),
            traders: Map::new(TRADER_NAMESPACE),
            target_os: None,
        }
    }

    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [&'static str]) -> Self {
        self.contract = self.contract.with_dependencies(dependencies);
        self
    }

    pub const fn with_replies(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_replies([&[], reply_handlers]);
        self
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'static [(&'static str, IbcCallbackHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_ibc_callbacks(callbacks);
        self
    }
    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Self, CustomInitMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_instantiate(instantiate_handler);
        self
    }

    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Self, ReceiveMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_receive(receive_handler);
        self
    }

    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Self, CustomExecMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_execute(execute_handler);
        self
    }

    pub const fn with_query(mut self, query_handler: QueryHandlerFn<Self, CustomQueryMsg>) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ExtensionState> {
        self.base_state.load(store)
    }

    pub fn target(&self) -> Result<&Addr, ExtensionError> {
        Ok(&self
            .target_os
            .as_ref()
            .ok_or_else(|| StdError::generic_err("No target OS specified to execute on."))?
            .proxy)
    }
}
