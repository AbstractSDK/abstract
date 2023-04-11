use crate::ApiError;
use abstract_core::objects::dependency::StaticDependency;
use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, Handler, IbcCallbackHandlerFn, InstantiateHandlerFn,
        QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn, SudoHandlerFn,
    },
    core::version_control::AccountBase,
    feature_objects::AnsHost,
    namespaces::BASE_STATE,
    AbstractSdkError,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub const AUTHORIZED_ADDRESSES_NAMESPACE: &str = "authorized_addresses";
pub const MAXIMUM_AUTHORIZED_ADDRESSES: u32 = 15;

pub trait ContractError:
    From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError> + 'static
{
}
impl<T> ContractError for T where
    T: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError> + 'static
{
}

/// The BaseState contains the main addresses needed for sending and verifying messages
/// Every DApp should use the provided **ans_host** contract for token/contract address resolution.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApiState {
    /// Used to verify requests
    pub version_control: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
}

/// The state variables for our ApiContract.
pub struct ApiContract<
    Error: ContractError,
    CustomInitMsg: 'static,
    CustomExecMsg: 'static,
    CustomQueryMsg: 'static,
    Receive: 'static = Empty,
    SudoMsg: 'static = Empty,
> where
    Self: Handler,
{
    pub(crate) contract: AbstractContract<Self, Error>,
    pub(crate) base_state: Item<'static, ApiState>,
    /// Map ProxyAddr -> AuthorizedAddrs
    pub authorized_addresses: Map<'static, Addr, Vec<Addr>>,
    /// The Account on which commands are executed. Set each time in the [`abstract_core::api::ExecuteMsg::Base`] handler.
    pub target_account: Option<AccountBase>,
}

/// Constructor
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    pub const fn new(
        name: &'static str,
        version: &'static str,
        metadata: Option<&'static str>,
    ) -> Self {
        Self {
            contract: AbstractContract::new(name, version, metadata),
            base_state: Item::new(BASE_STATE),
            authorized_addresses: Map::new(AUTHORIZED_ADDRESSES_NAMESPACE),
            target_account: None,
        }
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<ApiState> {
        self.base_state.load(store)
    }

    /// Return the address of the proxy for the Account associated with this API.
    /// Set each time in the [`abstract_core::api::ExecuteMsg::Base`] handler.
    pub fn target(&self) -> Result<&Addr, ApiError> {
        Ok(&self
            .target_account
            .as_ref()
            .ok_or_else(|| StdError::generic_err("No target Account specified to execute on."))?
            .proxy)
    }
    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.contract = self.contract.with_dependencies(dependencies);
        self
    }

    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Self, CustomInitMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_instantiate(instantiate_handler);
        self
    }

    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Self, CustomExecMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_execute(execute_handler);
        self
    }

    pub const fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Self, CustomQueryMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }

    pub const fn with_replies(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_replies([&[], reply_handlers]);
        self
    }

    pub const fn with_sudo(mut self, sudo_handler: SudoHandlerFn<Self, SudoMsg, Error>) -> Self {
        self.contract = self.contract.with_sudo(sudo_handler);
        self
    }

    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Self, ReceiveMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_receive(receive_handler);
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::mock::{ApiMockResult, MOCK_API};

    #[test]
    fn set_and_get_target() -> ApiMockResult {
        let mut mock = MOCK_API;
        let target = Addr::unchecked("target");
        mock.target_account = Some(AccountBase {
            proxy: target.clone(),
            manager: Addr::unchecked("manager"),
        });
        assert_eq!(mock.target()?, &target);
        Ok(())
    }
}
