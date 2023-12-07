mod endpoints;
pub mod error;
pub mod features;
pub(crate) mod handler;
pub mod msgs;
#[cfg(feature = "schema")]
pub mod schema;
pub mod state;
pub(crate) use abstract_sdk::base::*;

pub use crate::state::AppContract;
pub use error::AppError;
pub type AppResult<C = Empty> = Result<Response<C>, AppError>;
mod interface;

use cosmwasm_std::{Empty, Response};
#[cfg(feature = "test-utils")]
pub mod mock {
    pub use abstract_core::app;
    use abstract_core::{
        manager::ModuleInstallConfig,
        objects::{dependency::StaticDependency, module::ModuleInfo},
    };
    use abstract_interface::{AppDeployer, DependencyCreation};
    use cosmwasm_schema::QueryResponses;
    pub use cosmwasm_std::testing::*;
    use cosmwasm_std::{to_json_binary, Response, StdError};
    use cw_controllers::AdminError;
    use cw_orch::prelude::*;
    use cw_storage_plus::Item;

    pub type AppTestResult = Result<(), MockError>;

    crate::app_msg_types!(MockAppContract, MockExecMsg, MockQueryMsg);

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::ExecuteFns)]
    #[impl_into(ExecuteMsg)]
    pub enum MockExecMsg {
        DoSomething {},
        DoSomethingAdmin {},
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::QueryFns)]
    #[impl_into(QueryMsg)]
    #[derive(QueryResponses)]
    pub enum MockQueryMsg {
        #[returns(MockQueryResponse)]
        GetSomething {},

        #[returns(ReceivedIbcCallbackStatus)]
        GetReceivedIbcCallbackStatus {},
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryResponse {}

    #[cosmwasm_schema::cw_serde]
    pub struct ReceivedIbcCallbackStatus {
        pub received: bool,
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockSudoMsg;

    use crate::{AppContract, AppError};
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::{
        addresses::{test_account_base, TEST_ANS_HOST, TEST_VERSION_CONTROL},
        prelude::{
            MockDeps, MockQuerierBuilder, TEST_DEPENDENCY_MODULE_ID, TEST_MODULE_FACTORY,
            TEST_MODULE_ID, TEST_VERSION,
        },
    };
    use thiserror::Error;

    use self::interface::MockAppInterface;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        DappError(#[from] AppError),

        #[error("{0}")]
        Abstract(#[from] abstract_core::AbstractError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),

        #[error("{0}")]
        Admin(#[from] AdminError),
    }

    pub type MockAppContract = AppContract<
        // MockModule,
        MockError,
        MockInitMsg,
        MockExecMsg,
        MockQueryMsg,
        MockMigrateMsg,
        MockReceiveMsg,
        MockSudoMsg,
    >;

    pub const BASIC_MOCK_APP: MockAppContract =
        MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None);

    // Easy way to see if an ibc-callback was actually received.
    pub const IBC_CALLBACK_RECEIVED: Item<bool> = Item::new("ibc_callback_received");

    pub const MOCK_APP: MockAppContract = MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None)
        .with_instantiate(|deps, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &false)?;
            Ok(Response::new().set_data("mock_init".as_bytes()))
        })
        .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
        .with_query(|deps, _, _, msg| match msg {
            MockQueryMsg::GetSomething {} => {
                to_json_binary(&MockQueryResponse {}).map_err(Into::into)
            }
            MockQueryMsg::GetReceivedIbcCallbackStatus {} => {
                to_json_binary(&ReceivedIbcCallbackStatus {
                    received: IBC_CALLBACK_RECEIVED.load(deps.storage)?,
                })
                .map_err(Into::into)
            }
        })
        .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
        .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_receive".as_bytes())))
        .with_ibc_callbacks(&[("c_id", |deps, _, _, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &true).unwrap();
            Ok(Response::new().set_data("mock_callback".as_bytes()))
        })])
        .with_dependencies(&[StaticDependency::new(
            TEST_DEPENDENCY_MODULE_ID,
            &[TEST_VERSION],
        )])
        .with_replies(&[(1u64, |_, _, _, msg| {
            Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
        })])
        .with_migrate(|_, _, _, _| Ok(Response::new().set_data("mock_migrate".as_bytes())));

    crate::cw_orch_interface!(MOCK_APP, MockAppContract, MockAppInterface);

    // Needs to be in a separate module due to the `interface` module names colliding otherwise.
    pub mod mock_app_dependency {
        use abstract_testing::prelude::{TEST_DEPENDENCY_MODULE_ID, TEST_VERSION};
        use cosmwasm_std::{to_json_binary, Response};

        use super::{MockAppContract, MockQueryResponse};

        pub const MOCK_APP_DEPENDENCY: MockAppContract =
            MockAppContract::new(TEST_DEPENDENCY_MODULE_ID, TEST_VERSION, None)
                .with_instantiate(|_, _, _, _, _| {
                    Ok(Response::new().set_data("mock_init".as_bytes()))
                })
                .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
                .with_query(|_, _, _, _| to_json_binary(&MockQueryResponse {}).map_err(Into::into));

        crate::cw_orch_interface!(
            MOCK_APP_DEPENDENCY,
            MockAppContract,
            MockAppDependencyInterface
        );
    }

    impl<Chain: CwEnv> DependencyCreation for MockAppInterface<Chain> {
        type DependenciesConfig = Empty;
        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<Vec<ModuleInstallConfig>, abstract_interface::AbstractInterfaceError> {
            let install_config = ModuleInstallConfig::new(
                ModuleInfo::from_id(TEST_DEPENDENCY_MODULE_ID, TEST_VERSION.into())?,
                Some(to_json_binary(&MockInitMsg {})?),
            );
            Ok(vec![install_config])
        }
    }

    crate::export_endpoints!(MOCK_APP, MockAppContract);

    pub fn app_base_mock_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default()
            .with_smart_handler(TEST_MODULE_FACTORY, |_msg| panic!("unexpected messsage"))
    }

    /// Instantiate the contract with the default [`TEST_MODULE_FACTORY`].
    /// This will set the [`TEST_MANAGER`] as the admin.
    pub fn mock_init() -> MockDeps {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                account_base: test_account_base(),
            },
            module: MockInitMsg {},
        };

        BASIC_MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();

        deps
    }

    type Exec = app::ExecuteMsg<MockExecMsg>;
    type Query = app::QueryMsg<MockQueryMsg>;
    type Init = app::InstantiateMsg<MockInitMsg>;
    type Migrate = app::MigrateMsg<MockMigrateMsg>;

    #[cw_orch::interface(Init, Exec, Query, Migrate)]
    pub struct BootMockApp<Chain>;

    impl AppDeployer<Mock> for BootMockApp<Mock> {}

    impl Uploadable for BootMockApp<Mock> {
        fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
            Box::new(
                ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query)
                    .with_migrate(self::migrate),
            )
        }
    }

    #[macro_export]
    macro_rules! gen_app_mock {
    ($name:ident,$id:expr, $version:expr, $deps:expr) => {
        use ::abstract_core::app;
        use ::abstract_app::mock::{MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg};
        use ::cw_orch::prelude::*;
        use ::abstract_sdk::base::Handler;
        use ::abstract_sdk::features::AccountIdentification;
        use ::abstract_sdk::{Execution, TransferInterface};


        type Exec = app::ExecuteMsg<MockExecMsg, MockReceiveMsg>;
        type Query = app::QueryMsg<MockQueryMsg>;
        type Init = app::InstantiateMsg<MockInitMsg>;
        type Migrate = app::MigrateMsg<MockMigrateMsg>;
        const MOCK_APP: ::abstract_app::mock::MockAppContract = ::abstract_app::mock::MockAppContract::new($id, $version, None)
        .with_dependencies($deps)
        .with_execute(|deps, _env, info, module, msg| {
            match msg {
                MockExecMsg::DoSomethingAdmin{} => {
                    module.admin.assert_admin(deps.as_ref(), &info.sender)?;
                },
                _ => {},
            }
            Ok(::cosmwasm_std::Response::new().set_data("mock_exec".as_bytes()))
        })
        .with_instantiate(|deps, _env, info, module, msg| {
            let mut response = ::cosmwasm_std::Response::new().set_data("mock_init".as_bytes());
            // See test `create_sub_account_with_installed_module` where this will be triggered.
            if module.info().0 == "tester:mock-app1" {
                println!("checking address of adapter1");
                let manager = module.admin.get(deps.as_ref())?.unwrap();
                // Check if the adapter has access to its dependency during instantiation.
                let adapter1_addr = ::abstract_core::manager::state::ACCOUNT_MODULES.query(&deps.querier,manager, "tester:mock-adapter1")?;
                // We have address!
                ::cosmwasm_std::ensure!(
                    adapter1_addr.is_some(),
                    ::cosmwasm_std::StdError::generic_err("no address")
                );
                println!("adapter_addr: {adapter1_addr:?}");
                // See test `install_app_with_proxy_action` where this transfer will happen.
                let proxy_addr = module.proxy_address(deps.as_ref())?;
                let balance = deps.querier.query_balance(proxy_addr, "TEST")?;
                if !balance.amount.is_zero() {
                println!("sending amount from proxy: {balance:?}");
                    let action = module
                        .bank(deps.as_ref())
                        .transfer::<::cosmwasm_std::Coin>(
                            vec![balance.into()],
                            &::cosmwasm_std::Addr::unchecked("test_addr"),
                        )?;
                    let msg = module.executor(deps.as_ref()).execute(vec![action])?;
                    println!("message: {msg:?}");
                    response = response.add_message(msg);
                }
                Ok(response)}
            else {
                Ok(response)}
            });

        fn mock_instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::InstantiateEndpoint;
            MOCK_APP.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        fn mock_execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            MOCK_APP.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        fn mock_query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            MOCK_APP.query(deps, env, msg)
        }

        fn mock_migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::MigrateEndpoint;
            MOCK_APP.migrate(deps, env, msg)
        }

        #[cw_orch::interface(Init, Exec, Query, Migrate)]
        pub struct $name;

        impl ::abstract_interface::AppDeployer<Mock> for $name <Mock> {}

        impl Uploadable for $name<Mock> {
            fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
                Box::new(ContractWrapper::<
                    Exec,
                    _,
                    _,
                    _,
                    _,
                    _,
                >::new_with_empty(
                    self::mock_execute,
                    self::mock_instantiate,
                    self::mock_query,
                ).with_migrate(self::mock_migrate))
            }
        }

        impl<Chain: ::cw_orch::environment::CwEnv> $name <Chain> {
            pub fn new_test(chain: Chain) -> Self {
                Self(
                    cw_orch::contract::Contract::new($id,chain),
                )
            }
        }
    };
}
}
