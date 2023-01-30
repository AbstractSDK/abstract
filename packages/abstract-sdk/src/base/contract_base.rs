use super::handler::Handler;
use abstract_os::abstract_ica::StdAck;
use cosmwasm_std::{
    Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdError, StdResult, Storage,
};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::Item;
use os::objects::dependency::StaticDependency;

pub type ContractName = &'static str;
pub type VersionString = &'static str;
pub type ContractMetadata = Option<&'static str>;

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

const MAX_REPLY_COUNT: usize = 2;

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
    pub(crate) info: (ContractName, VersionString, ContractMetadata),
    /// On-chain storage of the same info
    pub(crate) version: Item<'static, ContractVersion>,
    /// ID's that this contract depends on
    pub(crate) dependencies: &'static [StaticDependency],
    /// Expected callbacks following an IBC action
    pub(crate) ibc_callback_handlers:
        &'static [(&'static str, IbcCallbackHandlerFn<Module, Error>)],
    /// Expected replies
    pub reply_handlers: [&'static [(u64, ReplyHandlerFn<Module, Error>)]; MAX_REPLY_COUNT],
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
    pub const fn new(
        name: ContractName,
        version: VersionString,
        metadata: ContractMetadata,
    ) -> Self {
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
    pub fn info(&self) -> (ContractName, VersionString, ContractMetadata) {
        self.info
    }
    /// add dependencies to the contract
    pub const fn with_dependencies(mut self, dependencies: &'static [StaticDependency]) -> Self {
        self.dependencies = dependencies;
        self
    }

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

#[cfg(test)]
mod test {
    use super::*;

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

    use thiserror::Error;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),
    }

    struct MockModule;

    type MockAppContract = AbstractContract<
        MockModule,
        MockError,
        MockExecMsg,
        MockInitMsg,
        MockQueryMsg,
        MockMigrateMsg,
        MockReceiveMsg,
    >;

    impl Handler for MockModule {
        type Error = MockError;
        type CustomExecMsg = MockExecMsg;
        type CustomInitMsg = MockInitMsg;
        type CustomQueryMsg = MockQueryMsg;
        type CustomMigrateMsg = MockMigrateMsg;
        type ReceiveMsg = MockReceiveMsg;

        fn contract(
            &self,
        ) -> &AbstractContract<
            Self,
            Self::Error,
            Self::CustomExecMsg,
            Self::CustomInitMsg,
            Self::CustomQueryMsg,
            Self::CustomMigrateMsg,
            Self::ReceiveMsg,
        > {
            unimplemented!()
        }
    }

    use speculoos::prelude::*;

    // #[test]
    // fn test_version() {
    //     let contract =
    //         MockAppContract::new("test_contract".into(), "0.1.0".into(), Metadata::default());
    //     let deps = mock_dependencies();
    //     let version = contract.version(&deps.storage).unwrap();
    //     let expected = ContractVersion {
    //         contract: "test_contract".into(),
    //         version: "0.1.0".into(),
    //     };
    //
    //     assert_that!(version).is_equal_to(expected);
    // }

    #[test]
    fn test_info() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default());
        let (name, version, metadata) = contract.info();
        assert_that!(&name).is_equal_to("test_contract");
        assert_that!(&version).is_equal_to("0.1.0");
        assert_that!(metadata).is_equal_to(ContractMetadata::default());
    }

    #[test]
    fn test_with_empty() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
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
        const verison: &str = "0.1.0";
        const dependency: StaticDependency = StaticDependency::new("test", &[verison]);
        const dependencies: &[StaticDependency] = &[dependency];

        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_dependencies(dependencies);

        assert_that!(contract.dependencies[0].clone()).is_equal_to(dependency);
    }

    #[test]
    fn test_with_instantiate() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_instantiate(|_, _, _, _, _| {
                Ok(Response::default().add_attribute("test", "instantiate"))
            });

        assert!(contract.instantiate_handler.is_some());
    }

    #[test]
    fn test_with_receive() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_receive(|_, _, _, _, _| Ok(Response::default().add_attribute("test", "receive")));

        assert!(contract.receive_handler.is_some());
    }

    #[test]
    fn test_with_execute() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_execute(|_, _, _, _, _| Ok(Response::default().add_attribute("test", "execute")));

        assert!(contract.execute_handler.is_some());
    }

    #[test]
    fn test_with_query() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_query(|_, _, _, _| Ok(cosmwasm_std::to_binary(&Empty {}).unwrap()));

        assert!(contract.query_handler.is_some());
    }

    #[test]
    fn test_with_migrate() {
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_migrate(|_, _, _, _| Ok(Response::default().add_attribute("test", "migrate")));

        assert!(contract.migrate_handler.is_some());
    }

    #[test]
    fn test_with_reply_handlers() {
        const reply_id: u64 = 50u64;
        const handler: ReplyHandlerFn<MockModule, MockError> =
            |_, _, _, _| Ok(Response::default().add_attribute("test", "reply"));
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_replies([&[(reply_id, handler)], &[]]);

        assert_that!(contract.reply_handlers[0][0].0).is_equal_to(reply_id);
        assert!(contract.reply_handlers[1].is_empty());
    }

    #[test]
    fn test_with_ibc_callback_handlers() {
        const ibc_id: &str = "aoeu";
        const handler: IbcCallbackHandlerFn<MockModule, MockError> =
            |_, _, _, _, _, _| Ok(Response::default().add_attribute("test", "ibc"));
        let contract = MockAppContract::new("test_contract", "0.1.0", ContractMetadata::default())
            .with_ibc_callbacks(&[(ibc_id, handler)]);

        assert_that!(contract.ibc_callback_handlers[0].0).is_equal_to(ibc_id);
    }
}
