use super::handler::Handler;
use crate::{AbstractSdkError, AbstractSdkResult};
use abstract_core::abstract_ica::StdAck;
use core::objects::dependency::StaticDependency;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, Storage};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::Item;

pub type ModuleId = &'static str;
/// Version of the contract in str format.
pub type VersionString = &'static str;
pub type ModuleMetadata = Option<&'static str>;

pub trait MessageTypes {
    type CustomInitMsg;
    type CustomExecMsg;
    type CustomQueryMsg;
    type CustomMigrateMsg;
    type ReceiveMsg;
    type SudoMsg;
}
// ANCHOR: init
/// Function signature for an instantiate handler.
pub type InstantiateHandlerFn<Module, InitMsg, Error> =
    fn(DepsMut, Env, MessageInfo, Module, InitMsg) -> Result<Response, Error>;
// ANCHOR_END: init

// ANCHOR: exec
/// Function signature for an execute handler.
pub type ExecuteHandlerFn<Module, ExecMsg, Error> =
    fn(DepsMut, Env, MessageInfo, Module, ExecMsg) -> Result<Response, Error>;
// ANCHOR_END: exec

// ANCHOR: query
/// Function signature for a query handler.
pub type QueryHandlerFn<Module, QueryMsg, Error> =
    fn(Deps, Env, &Module, QueryMsg) -> Result<Binary, Error>;
// ANCHOR_END: query

type CallbackId = String;
// ANCHOR: ibc
/// Function signature for an IBC callback handler.
pub type IbcCallbackHandlerFn<Module, Error> =
    fn(DepsMut, Env, MessageInfo, Module, CallbackId, StdAck) -> Result<Response, Error>;
// ANCHOR_END: ibc

// ANCHOR: mig
/// Function signature for a migrate handler.
pub type MigrateHandlerFn<Module, MigrateMsg, Error> =
    fn(DepsMut, Env, Module, MigrateMsg) -> Result<Response, Error>;
// ANCHOR_END: mig

// ANCHOR: rec
/// Function signature for a receive handler.
pub type ReceiveHandlerFn<Module, Msg, Error> =
    fn(DepsMut, Env, MessageInfo, Module, Msg) -> Result<Response, Error>;
// ANCHOR_END: rec

// ANCHOR: sudo
/// Function signature for a sudo handler.
pub type SudoHandlerFn<Module, SudoMsg, Error> =
    fn(DepsMut, Env, Module, SudoMsg) -> Result<Response, Error>;
// ANCHOR_END: sudo

// ANCHOR: reply
/// Function signature for a reply handler.
pub type ReplyHandlerFn<Module, Error> = fn(DepsMut, Env, Module, Reply) -> Result<Response, Error>;
// ANCHOR_END: reply

/// There can be two locations where reply handlers are added.
/// 1. Base implementation of the contract.
/// 2. Custom implementation of the contract.
const MAX_REPLY_COUNT: usize = 2;

/// Abstract generic contract
pub struct AbstractContract<Module: Handler + 'static, Error: From<AbstractSdkError> + 'static> {
    /// Static info about the contract, used for migration
    pub(crate) info: (ModuleId, VersionString, ModuleMetadata),
    /// On-chain storage of the same info.
    pub(crate) version: Item<'static, ContractVersion>,
    /// Modules that this contract depends on.
    pub(crate) dependencies: &'static [StaticDependency],
    /// Handler of instantiate messages.
    pub(crate) instantiate_handler:
        Option<InstantiateHandlerFn<Module, <Module as Handler>::CustomInitMsg, Error>>,
    /// Handler of execute messages.
    pub(crate) execute_handler:
        Option<ExecuteHandlerFn<Module, <Module as Handler>::CustomExecMsg, Error>>,
    /// Handler of query messages.
    pub(crate) query_handler:
        Option<QueryHandlerFn<Module, <Module as Handler>::CustomQueryMsg, Error>>,
    /// Handler for migrations.
    pub(crate) migrate_handler:
        Option<MigrateHandlerFn<Module, <Module as Handler>::CustomMigrateMsg, Error>>,
    /// Handler for sudo messages.
    pub(crate) sudo_handler: Option<SudoHandlerFn<Module, <Module as Handler>::SudoMsg, Error>>,
    /// List of reply handlers per reply ID.
    pub reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; MAX_REPLY_COUNT],
    /// Handler of `Receive variant Execute messages.
    pub(crate) receive_handler:
        Option<ReceiveHandlerFn<Module, <Module as Handler>::ReceiveMsg, Error>>,
    /// IBC callbacks handlers following an IBC action, per callback ID.
    pub(crate) ibc_callback_handlers:
        &'static [(&'static str, IbcCallbackHandlerFn<Module, Error>)],
}

impl<Module, Error: From<AbstractSdkError>> AbstractContract<Module, Error>
where
    Module: Handler,
{
    /// Creates a new customizable abstract contract.
    pub const fn new(name: ModuleId, version: VersionString, metadata: ModuleMetadata) -> Self {
        Self {
            info: (name, version, metadata),
            version: CONTRACT,
            ibc_callback_handlers: &[],
            reply_handlers: [&[], &[]],
            dependencies: &[],
            execute_handler: None,
            receive_handler: None,
            migrate_handler: None,
            sudo_handler: None,
            instantiate_handler: None,
            query_handler: None,
        }
    }
    /// Gets the cw2 version of the contract.
    pub fn version(&self, store: &dyn Storage) -> AbstractSdkResult<ContractVersion> {
        self.version.load(store).map_err(Into::into)
    }
    /// Gets the static info of the contract.
    pub fn info(&self) -> (ModuleId, VersionString, ModuleMetadata) {
        self.info
    }
    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.dependencies = dependencies;
        self
    }
    /// Add reply handlers to the contract.
    pub const fn with_replies(
        mut self,
        reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; MAX_REPLY_COUNT],
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

    /// Add instantiate handler to the contract.
    pub const fn with_instantiate(
        mut self,
        instantiate_handler: InstantiateHandlerFn<
            Module,
            <Module as Handler>::CustomInitMsg,
            Error,
        >,
    ) -> Self {
        self.instantiate_handler = Some(instantiate_handler);
        self
    }

    /// Add query handler to the contract.
    pub const fn with_migrate(
        mut self,
        migrate_handler: MigrateHandlerFn<Module, <Module as Handler>::CustomMigrateMsg, Error>,
    ) -> Self {
        self.migrate_handler = Some(migrate_handler);
        self
    }

    /// Add sudo handler to the contract.
    pub const fn with_sudo(
        mut self,
        sudo_handler: SudoHandlerFn<Module, <Module as Handler>::SudoMsg, Error>,
    ) -> Self {
        self.sudo_handler = Some(sudo_handler);
        self
    }

    /// Add receive handler to the contract.
    pub const fn with_receive(
        mut self,
        receive_handler: ReceiveHandlerFn<Module, <Module as Handler>::ReceiveMsg, Error>,
    ) -> Self {
        self.receive_handler = Some(receive_handler);
        self
    }

    /// Add execute handler to the contract.
    pub const fn with_execute(
        mut self,
        execute_handler: ExecuteHandlerFn<Module, <Module as Handler>::CustomExecMsg, Error>,
    ) -> Self {
        self.execute_handler = Some(execute_handler);
        self
    }

    /// Add query handler to the contract.
    pub const fn with_query(
        mut self,
        query_handler: QueryHandlerFn<Module, <Module as Handler>::CustomQueryMsg, Error>,
    ) -> Self {
        self.query_handler = Some(query_handler);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::Empty;
    use speculoos::assert_that;

    #[cosmwasm_schema::cw_serde]
    struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    struct MockExecMsg;

    #[cosmwasm_schema::cw_serde]
    struct MockQueryMsg;

    #[cosmwasm_schema::cw_serde]
    struct MockMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    struct MockReceiveMsg;

    #[cosmwasm_schema::cw_serde]
    struct MockSudoMsg;

    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Sdk(#[from] AbstractSdkError),
    }

    struct MockModule;

    type MockAppContract = AbstractContract<MockModule, MockError>;

    impl Handler for MockModule {
        type Error = MockError;
        type CustomInitMsg = MockInitMsg;
        type CustomExecMsg = MockExecMsg;
        type CustomQueryMsg = MockQueryMsg;
        type CustomMigrateMsg = MockMigrateMsg;
        type ReceiveMsg = MockReceiveMsg;
        type SudoMsg = MockSudoMsg;

        fn contract(&self) -> &AbstractContract<Self, Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn test_info() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default());
        let (name, version, metadata) = contract.info();
        assert_that!(&name).is_equal_to("test_contract");
        assert_that!(&version).is_equal_to("0.1.0");
        assert_that!(metadata).is_equal_to(ModuleMetadata::default());
    }

    #[test]
    fn test_with_empty() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_dependencies(&[]);

        assert!(contract.reply_handlers.iter().all(|x| x.is_empty()));

        assert!(contract.dependencies.is_empty());
        assert!(contract.ibc_callback_handlers.is_empty());
        assert!(contract.instantiate_handler.is_none());
        assert!(contract.receive_handler.is_none());
        assert!(contract.execute_handler.is_none());
        assert!(contract.query_handler.is_none());
        assert!(contract.migrate_handler.is_none());
    }

    #[test]
    fn test_with_dependencies() {
        const VERSION: &str = "0.1.0";
        const DEPENDENCY: StaticDependency = StaticDependency::new("test", &[VERSION]);
        const DEPENDENCIES: &[StaticDependency] = &[DEPENDENCY];

        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_dependencies(DEPENDENCIES);

        assert_that!(contract.dependencies[0].clone()).is_equal_to(DEPENDENCY);
    }

    #[test]
    fn test_with_instantiate() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_instantiate(|_, _, _, _, _| {
                Ok(Response::default().add_attribute("test", "instantiate"))
            });

        assert!(contract.instantiate_handler.is_some());
    }

    #[test]
    fn test_with_receive() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_receive(|_, _, _, _, _| Ok(Response::default().add_attribute("test", "receive")));

        assert!(contract.receive_handler.is_some());
    }

    #[test]
    fn test_with_sudo() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_sudo(|_, _, _, _| Ok(Response::default().add_attribute("test", "sudo")));

        assert!(contract.sudo_handler.is_some());
    }

    #[test]
    fn test_with_execute() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_execute(|_, _, _, _, _| Ok(Response::default().add_attribute("test", "execute")));

        assert!(contract.execute_handler.is_some());
    }

    #[test]
    fn test_with_query() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_query(|_, _, _, _| Ok(cosmwasm_std::to_binary(&Empty {}).unwrap()));

        assert!(contract.query_handler.is_some());
    }

    #[test]
    fn test_with_migrate() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_migrate(|_, _, _, _| Ok(Response::default().add_attribute("test", "migrate")));

        assert!(contract.migrate_handler.is_some());
    }

    #[test]
    fn test_with_reply_handlers() {
        const REPLY_ID: u64 = 50u64;
        const HANDLER: ReplyHandlerFn<MockModule, MockError> =
            |_, _, _, _| Ok(Response::default().add_attribute("test", "reply"));
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_replies([&[(REPLY_ID, HANDLER)], &[]]);

        assert_that!(contract.reply_handlers[0][0].0).is_equal_to(REPLY_ID);
        assert!(contract.reply_handlers[1].is_empty());
    }

    #[test]
    fn test_with_ibc_callback_handlers() {
        const IBC_ID: &str = "aoeu";
        const HANDLER: IbcCallbackHandlerFn<MockModule, MockError> =
            |_, _, _, _, _, _| Ok(Response::default().add_attribute("test", "ibc"));
        let contract = MockAppContract::new("test_contract", "0.1.0", ModuleMetadata::default())
            .with_ibc_callbacks(&[(IBC_ID, HANDLER)]);

        assert_that!(contract.ibc_callback_handlers[0].0).is_equal_to(IBC_ID);
    }
}
