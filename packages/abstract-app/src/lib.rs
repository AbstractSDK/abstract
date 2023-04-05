mod endpoints;
pub mod error;
pub mod features;
pub(crate) mod handler;
pub mod schema;
pub mod state;
pub(crate) use abstract_sdk::base::*;

pub use crate::state::AppContract;
pub use error::AppError;
pub type AppResult<C = Empty> = Result<Response<C>, AppError>;

use cosmwasm_std::{Empty, Response};
#[cfg(feature = "test-utils")]
pub mod mock {
    use abstract_boot::AppDeployer;
    pub use abstract_core::app;
    use boot_core::{ContractWrapper, CwEnv};
    pub use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, to_binary, Addr, StdError};

    pub type AppTestResult = Result<(), MockError>;

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockExecMsg;

    impl app::AppExecuteMsg for MockExecMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryMsg;

    impl app::AppQueryMsg for MockQueryMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockMigrateMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    use crate::{AppContract, AppError};
    use abstract_core::{module_factory::ContextResponse, version_control::AccountBase};
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::prelude::{
        MockDeps, MockQuerierBuilder, TEST_ANS_HOST, TEST_MANAGER, TEST_MODULE_FACTORY,
        TEST_MODULE_ID, TEST_PROXY, TEST_VERSION,
    };
    use thiserror::Error;

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
    }

    pub type MockAppContract = AppContract<
        // MockModule,
        MockError,
        MockInitMsg,
        MockExecMsg,
        MockQueryMsg,
        MockMigrateMsg,
        MockReceiveMsg,
    >;

    pub const MOCK_APP: MockAppContract = MockAppContract::new(TEST_MODULE_ID, TEST_VERSION, None);

    crate::export_endpoints!(MOCK_APP, MockAppContract);

    pub fn app_base_mock_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default().with_smart_handler(TEST_MODULE_FACTORY, |msg| {
            match from_binary(msg).unwrap() {
                abstract_core::module_factory::QueryMsg::Context {} => {
                    let resp = ContextResponse {
                        account_base: Some(AccountBase {
                            manager: Addr::unchecked(TEST_MANAGER),
                            proxy: Addr::unchecked(TEST_PROXY),
                        }),
                        module: None,
                    };
                    Ok(to_binary(&resp).unwrap())
                }
                _ => panic!("unexpected message"),
            }
        })
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
            },
            module: MockInitMsg {},
        };

        MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();

        deps
    }

    type Exec = app::ExecuteMsg<MockExecMsg>;
    type Query = app::QueryMsg<MockQueryMsg>;
    type Init = app::InstantiateMsg<MockInitMsg>;
    type Migrate = app::MigrateMsg<MockMigrateMsg>;
    #[boot_core::contract(Init, Exec, Query, Migrate)]
    pub struct BootMockApp;

    impl<Chain: CwEnv> AppDeployer<Chain> for BootMockApp<Chain> {}

    impl<Chain: boot_core::CwEnv> BootMockApp<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(
                boot_core::Contract::new(name, chain).with_mock(Box::new(
                    ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query)
                        .with_migrate(self::migrate),
                )),
            )
        }
    }

    #[macro_export]
    macro_rules! gen_app_mock {
    ($name:ident,$id:expr, $version:expr, $deps:expr) => {
        use ::abstract_core::app;
        use ::abstract_app::mock::{MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg};

        type Exec = app::ExecuteMsg<MockExecMsg, MockReceiveMsg>;
        type Query = app::QueryMsg<MockQueryMsg>;
        type Init = app::InstantiateMsg<MockInitMsg>;
        type Migrate = app::MigrateMsg<MockMigrateMsg>;
        const MOCK_APP: ::abstract_app::mock::MockAppContract = ::abstract_app::mock::MockAppContract::new($id, $version, None)
        .with_dependencies($deps);

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

        #[boot_core::contract(Init, Exec, Query, Migrate)]
        pub struct $name;

        impl<Chain: ::boot_core::CwEnv> ::abstract_boot::AppDeployer<Chain> for $name <Chain> {}

        impl<Chain: ::boot_core::CwEnv> $name <Chain> {
            pub fn new(chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new($id,chain).with_mock(Box::new(::boot_core::ContractWrapper::<
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
                    ).with_migrate(self::mock_migrate))),
                )
            }
        }
    };
}
}
