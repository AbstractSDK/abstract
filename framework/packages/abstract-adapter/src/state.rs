use abstract_sdk::features::ModuleIdentification;
use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, Handler, IbcCallbackHandlerFn, InstantiateHandlerFn,
        ModuleIbcHandlerFn, QueryHandlerFn, ReplyHandlerFn, SudoHandlerFn,
    },
    namespaces::BASE_STATE,
    std::version_control::AccountBase,
    AbstractSdkError,
};
use abstract_std::{
    adapter::AdapterState,
    objects::{dependency::StaticDependency, module::ModuleInfo},
    AbstractError, AbstractResult,
};
use cosmwasm_std::{Addr, Empty, StdError, StdResult, Storage};
use cw_storage_plus::{Item, Map};

use crate::AdapterError;

pub const AUTHORIZED_ADDRESSES_NAMESPACE: &str = "authorized_addresses";
pub const MAXIMUM_AUTHORIZED_ADDRESSES: u32 = 15;

pub trait ContractError:
    From<cosmwasm_std::StdError>
    + From<AdapterError>
    + From<AbstractSdkError>
    + From<AbstractError>
    + 'static
{
}
impl<T> ContractError for T where
    T: From<cosmwasm_std::StdError>
        + From<AdapterError>
        + From<AbstractSdkError>
        + From<AbstractError>
        + 'static
{
}

/// The state variables for our AdapterContract.
pub struct AdapterContract<
    Error: ContractError,
    CustomInitMsg: 'static,
    CustomExecMsg: 'static,
    CustomQueryMsg: 'static,
    SudoMsg: 'static = Empty,
> where
    Self: Handler,
{
    pub(crate) contract: AbstractContract<Self, Error>,
    pub(crate) base_state: Item<'static, AdapterState>,
    /// Map ProxyAddr -> AuthorizedAddrs
    pub authorized_addresses: Map<'static, Addr, Vec<Addr>>,
    /// The Account on which commands are executed. Set each time in the [`abstract_std::adapter::ExecuteMsg::Base`] handler.
    pub target_account: Option<AccountBase>,
}

/// Constructor
impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
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

    pub fn version(&self) -> &'static str {
        self.contract.info().1
    }

    pub fn module_info(&self) -> AbstractResult<ModuleInfo> {
        ModuleInfo::from_id(self.module_id(), self.version().into())
    }

    pub fn state(&self, store: &dyn Storage) -> StdResult<AdapterState> {
        self.base_state.load(store)
    }

    /// Return the address of the proxy for the Account associated with this Adapter.
    /// Set each time in the [`abstract_std::adapter::ExecuteMsg::Base`] handler.
    pub fn target(&self) -> Result<&Addr, AdapterError> {
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

    /// add IBC callback handler to contract
    pub const fn with_ibc_callback(mut self, callback: IbcCallbackHandlerFn<Self, Error>) -> Self {
        self.contract = self.contract.with_ibc_callback(callback);
        self
    }

    /// add Module IBC to contract
    pub const fn with_module_ibc(
        mut self,
        module_handler: ModuleIbcHandlerFn<Self, Error>,
    ) -> Self {
        self.contract = self.contract.with_module_ibc(module_handler);
        self
    }
}

#[cfg(test)]
mod tests {

    use abstract_testing::prelude::*;
    use cosmwasm_std::Response;

    use super::*;
    use crate::mock::{AdapterMockResult, MOCK_ADAPTER, TEST_METADATA};

    #[test]
    fn set_and_get_target() -> AdapterMockResult {
        let mut mock = MOCK_ADAPTER;
        let target = Addr::unchecked("target");
        mock.target_account = Some(AccountBase {
            proxy: target.clone(),
            manager: Addr::unchecked("manager"),
        });
        assert_eq!(mock.target()?, &target);
        Ok(())
    }

    #[test]
    fn builder_functions() {
        crate::mock::MockAdapterContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_instantiate(|_, _, _, _, _| Ok(Response::new().set_data("mock_init".as_bytes())))
            .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
            .with_query(|_, _, _, _| cosmwasm_std::to_json_binary("mock_query").map_err(Into::into))
            .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
            .with_ibc_callback(|_, _, _, _, _| {
                Ok(Response::new().set_data("mock_callback".as_bytes()))
            })
            .with_replies(&[(1u64, |_, _, _, msg| {
                Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
            })]);
    }
}
