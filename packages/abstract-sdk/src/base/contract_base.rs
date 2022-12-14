use abstract_os::abstract_ica::StdAck;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage,
};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::Item;

use os::objects::dependency::StaticDependency;

use super::endpoints::migrate::{Metadata, Name, VersionString};

use super::handler::Handler;

pub type IbcCallbackHandlerFn<Module, Error> =
    fn(DepsMut, Env, MessageInfo, Module, String, StdAck) -> Result<Response, Error>;

pub type ExecuteHandlerFn<Module, RequestMsg, Error> =
    fn(DepsMut, Env, MessageInfo, Module, RequestMsg) -> Result<Response, Error>;

pub type MigrateHandlerFn<Module, MigrateMsg, Error> =
    fn(DepsMut, Env, Module, MigrateMsg) -> Result<Response, Error>;

pub type InstantiateHandlerFn<Module, InitMsg, Error> =
    fn(DepsMut, Env, MessageInfo, Module, InitMsg) -> Result<Response, Error>;

pub type QueryHandlerFn<Module, QueryMsg> =
    fn(Deps, Env, &Module, QueryMsg) -> Result<Binary, StdError>;

pub type ReceiveHandlerFn<App, Msg, Error> =
    fn(DepsMut, Env, MessageInfo, App, Msg) -> Result<Response, Error>;

pub type ReplyHandlerFn<Module, Error> = fn(DepsMut, Env, Module, Reply) -> Result<Response, Error>;

/// State variables for a generic contract
pub struct AbstractContract<
    Module: Handler + 'static,
    Error: From<cosmwasm_std::StdError> + 'static,
    CustomExecMsg = Empty,
    CustomInitMsg = Empty,
    CustomQueryMsg = Empty,
    CustomMigrateMsg = Empty,
    ReceiveMsg = Empty,
> {
    /// static info about the contract, used for migration
    pub(crate) info: (Name, VersionString, Metadata),
    /// On-chain storage of the same info
    pub(crate) version: Item<'static, ContractVersion>,
    /// ID's that this contract depends on
    pub(crate) dependencies: &'static [StaticDependency],
    /// Expected callbacks following an IBC action
    pub(crate) ibc_callback_handlers:
        &'static [(&'static str, IbcCallbackHandlerFn<Module, Error>)],
    /// Expected replies
    pub reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; 2],
    /// Handler of execute messages
    pub(crate) execute_handler: Option<ExecuteHandlerFn<Module, CustomExecMsg, Error>>,
    /// Handler of instantiate messages
    pub(crate) instantiate_handler: Option<InstantiateHandlerFn<Module, CustomInitMsg, Error>>,
    /// Handler of query messages
    pub(crate) query_handler: Option<QueryHandlerFn<Module, CustomQueryMsg>>,
    /// Handler for migrations
    pub(crate) migrate_handler: Option<MigrateHandlerFn<Module, CustomMigrateMsg, Error>>,
    /// Handler of `Receive variant Execute messages
    pub(crate) receive_handler: Option<ReceiveHandlerFn<Module, ReceiveMsg, Error>>,
}

impl<
        Module,
        Error: From<cosmwasm_std::StdError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
    AbstractContract<
        Module,
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
where
    Module: Handler,
{
    pub const fn new(name: Name, version: VersionString, metadata: Metadata) -> Self {
        Self {
            info: (name, version, metadata),
            version: CONTRACT,
            ibc_callback_handlers: &[],
            reply_handlers: [&[], &[]],
            dependencies: &[],
            execute_handler: None,
            receive_handler: None,
            migrate_handler: None,
            instantiate_handler: None,
            query_handler: None,
        }
    }

    pub fn version(&self, store: &dyn Storage) -> StdResult<ContractVersion> {
        self.version.load(store)
    }
    pub fn info(&self) -> (Name, VersionString, Metadata) {
        self.info
    }
    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.dependencies = dependencies;
        self
    }

    pub const fn with_replies(
        mut self,
        reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; 2],
    ) -> Self {
        self.reply_handlers = reply_handlers;
        self
    }

    /// add IBC callback handler to contract
    pub const fn with_ibc_callbacks(
        mut self,
        callbacks: &'static [(&'static str, IbcCallbackHandlerFn<Module, Error>)],
    ) -> Self {
        self.ibc_callback_handlers = callbacks;
        self
    }

    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<Module, CustomInitMsg, Error>,
    ) -> Self {
        self.instantiate_handler = Some(instantiate_handler);
        self
    }

    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Module, ReceiveMsg, Error>,
    ) -> Self {
        self.receive_handler = Some(receive_handler);
        self
    }

    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Module, CustomExecMsg, Error>,
    ) -> Self {
        self.execute_handler = Some(execute_handler);
        self
    }

    pub const fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Module, CustomQueryMsg>,
    ) -> Self {
        self.query_handler = Some(query_handler);
        self
    }

    pub const fn with_migrate(
        mut self,
        migrate_handler: MigrateHandlerFn<Module, CustomMigrateMsg, Error>,
    ) -> Self {
        self.migrate_handler = Some(migrate_handler);
        self
    }
}
