use crate::{
    AbstractContract, AppError, ExecuteHandlerFn, IbcCallbackHandlerFn, InstantiateHandlerFn,
    MigrateHandlerFn, QueryHandlerFn, ReceiveHandlerFn, ReplyHandlerFn,
};
use abstract_core::objects::{dependency::StaticDependency, nested_admin::NestedAdmin};
use abstract_core::AbstractError;
use abstract_sdk::features::ModuleEndpointResponse;
use abstract_sdk::{
    base::SudoHandlerFn,
    feature_objects::{AnsHost, VersionControlContract},
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
    AbstractSdkError,
};
use cosmwasm_std::{Addr, Empty, StdResult, Storage};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AppState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Used to verify requests
    pub version_control: VersionControlContract,
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
    pub admin: NestedAdmin<'static>,
    pub(crate) base_state: Item<'static, AppState>,
    pub response: ModuleEndpointResponse,

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
    pub fn new(name: &'static str, version: &'static str, metadata: Option<&'static str>) -> Self {
        Self {
            base_state: Item::new(BASE_STATE),
            admin: NestedAdmin::new(ADMIN_NAMESPACE),
            contract: AbstractContract::new(name, version, metadata),
            response: ModuleEndpointResponse::default(),
        }
    }

    pub fn module_id(&self) -> &str {
        self.contract.info().0
    }

    pub fn version(&self) -> &str {
        self.contract.info().1
    }

    pub fn load_state(&self, store: &dyn Storage) -> StdResult<AppState> {
        self.base_state.load(store)
    }

    /// add dependencies to the contract
    pub fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.contract = self.contract.with_dependencies(dependencies);
        self
    }

    pub fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Self, CustomInitMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_instantiate(instantiate_handler);
        self
    }

    pub fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Self, CustomExecMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_execute(execute_handler);
        self
    }

    pub fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Self, CustomQueryMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_query(query_handler);
        self
    }

    pub fn with_migrate(
        mut self,
        migrate_handler: MigrateHandlerFn<Self, CustomMigrateMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_migrate(migrate_handler);
        self
    }

    pub fn with_replies(
        mut self,
        reply_handlers: &'static [(u64, ReplyHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_replies([&[], reply_handlers]);
        self
    }

    pub fn with_sudo(mut self, sudo_handler: SudoHandlerFn<Self, SudoMsg, Error>) -> Self {
        self.contract = self.contract.with_sudo(sudo_handler);
        self
    }

    pub fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Self, ReceiveMsg, Error>,
    ) -> Self {
        self.contract = self.contract.with_receive(receive_handler);
        self
    }

    /// add IBC callback handler to contract
    pub fn with_ibc_callbacks(
        mut self,
        callbacks: &'static [(&'static str, IbcCallbackHandlerFn<Self, Error>)],
    ) -> Self {
        self.contract = self.contract.with_ibc_callbacks(callbacks);
        self
    }
}

#[cfg(test)]
mod tests {
    use abstract_sdk::features::CustomData;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{DepsMut, Env, MessageInfo};

    use crate::mock::{MockAppContract, MockError, MockInitMsg};

    fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        app: &mut MockAppContract,
        _msg: MockInitMsg,
    ) -> Result<(), MockError> {
        app.set_data("mock_init".as_bytes());
        Ok(())
    }
    #[test]
    fn builder() {
        let app = MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None)
            .with_instantiate(instantiate)
            .with_execute(|_, _, _, app, _| {
                app.set_data("mock_exec".as_bytes());
                Ok(())
            })
            .with_query(|_, _, _, _| cosmwasm_std::to_json_binary("mock_query").map_err(Into::into))
            .with_sudo(|_, _, app, _| {
                app.set_data("mock_sudo".as_bytes());
                Ok(())
            })
            .with_receive(|_, _, _, app, _| {
                app.set_data("mock_receive".as_bytes());
                Ok(())
            })
            .with_ibc_callbacks(&[("c_id", |_, _, _, app, _, _, _| {
                app.set_data("mock_callback".as_bytes());
                Ok(())
            })])
            .with_replies(&[(1u64, |_, _, app, msg| {
                app.set_data(msg.result.unwrap().data.unwrap());
                Ok(())
            })])
            .with_migrate(|_, _, app, _| {
                app.set_data("mock_migrate".as_bytes());
                Ok(())
            });

        assert_eq!(app.module_id(), TEST_MODULE_ID);
        assert_eq!(app.version(), TEST_VERSION);
    }
}
