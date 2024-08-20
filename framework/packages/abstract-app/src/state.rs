use abstract_sdk::{
    base::{ModuleIbcHandlerFn, SudoHandlerFn},
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
    AbstractSdkError,
};
use abstract_std::{
    app::AppState,
    objects::{
        dependency::StaticDependency, module::ModuleInfo, ownership::nested_admin::NestedAdmin,
    },
    AbstractError, AbstractResult,
};
use cosmwasm_std::{Empty, StdResult, Storage};
use cw_storage_plus::Item;

use crate::{
    AbstractContract, AppError, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
};

pub trait ContractError:
    From<cosmwasm_std::StdError>
    + From<AppError>
    + From<AbstractSdkError>
    + From<AbstractError>
    + 'static
{
}

impl<T> ContractError for T where
    T: From<cosmwasm_std::StdError>
        + From<AppError>
        + From<AbstractSdkError>
        + From<AbstractError>
        + 'static
{
}

/// The state variables for our AppContract.
pub struct AppContract<
    Error: ContractError,
    CustomInitMsg: 'static,
    CustomExecMsg: 'static,
    CustomQueryMsg: 'static,
    CustomMigrateMsg: 'static,
    Receive: 'static = Empty,
    SudoMsg: 'static = Empty,
> {
    // Custom state for every App
    pub admin: NestedAdmin,
    pub(crate) base_state: Item<AppState>,

    // Scaffolding contract that handles type safety and provides helper methods
    pub(crate) contract: AbstractContract<Self, Error>,
}

/// Constructor
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
    AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    pub const fn new(
        name: &'static str,
        version: &'static str,
        metadata: Option<&'static str>,
    ) -> Self {
        Self {
            base_state: Item::new(BASE_STATE),
            admin: NestedAdmin::new(ADMIN_NAMESPACE),
            contract: AbstractContract::new(name, version, metadata),
        }
    }

    pub fn module_id(&self) -> &'static str {
        self.contract.info().0
    }

    pub fn version(&self) -> &'static str {
        self.contract.info().1
    }

    pub fn module_info(&self) -> AbstractResult<ModuleInfo> {
        ModuleInfo::from_id(self.module_id(), self.version().into())
    }

    pub fn load_state(&self, store: &dyn Storage) -> StdResult<AppState> {
        self.base_state.load(store)
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

    pub const fn with_migrate(
        mut self,
        migrate_handler: MigrateHandlerFn<Self, CustomMigrateMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_migrate(migrate_handler);
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

    use crate::mock::MockAppContract;

    #[test]
    fn builder() {
        let app = MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None)
            .with_instantiate(|_, _, _, _, _| Ok(Response::new().set_data("mock_init".as_bytes())))
            .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
            .with_query(|_, _, _, _| cosmwasm_std::to_json_binary("mock_query").map_err(Into::into))
            .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
            .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_receive".as_bytes())))
            .with_ibc_callback(|_, _, _, _, _| {
                Ok(Response::new().set_data("mock_callback".as_bytes()))
            })
            .with_replies(&[(1u64, |_, _, _, msg| {
                Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
            })])
            .with_migrate(|_, _, _, _| Ok(Response::new().set_data("mock_migrate".as_bytes())));

        assert_eq!(app.module_id(), TEST_MODULE_ID);
        assert_eq!(app.version(), TEST_VERSION);
    }
}
