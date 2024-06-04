mod endpoints;
pub mod features;
#[cfg(feature = "schema")]
pub mod state;

pub use crate::state::StandaloneContract;

// Useful re-exports
pub use abstract_std as std;
// re-export objects specifically
pub use abstract_sdk as sdk;

pub use crate::std::objects;
pub mod traits {
    pub use abstract_sdk::{features::*, prelude::*};
}

mod interface;
pub use abstract_interface;
#[cfg(feature = "test-utils")]
pub use abstract_testing;

#[cfg(feature = "test-utils")]
pub mod mock {
    use abstract_std::standalone;
    use cosmwasm_schema::QueryResponses;
    pub use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{
        to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    };
    use cw_controllers::AdminError;
    use cw_orch::prelude::*;
    use cw_storage_plus::Item;

    pub type AppTestResult = Result<(), MockError>;

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg {
        pub ans_host_address: String,
        pub version_control_address: String,
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::ExecuteFns)]
    pub enum MockExecMsg {
        DoSomething {},
        DoSomethingAdmin {},
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::QueryFns, QueryResponses)]
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

    #[cw_orch::interface(MockInitMsg, MockExecMsg, MockQueryMsg, MockMigrateMsg)]
    pub struct MockStandaloneWithDepI;

    use abstract_sdk::{AbstractResponse, AbstractSdkError};
    use abstract_testing::{
        addresses::{TEST_ANS_HOST, TEST_VERSION_CONTROL},
        prelude::{
            MockDeps, MockQuerierBuilder, TEST_MODULE_FACTORY, TEST_MODULE_ID, TEST_VERSION,
        },
    };
    use thiserror::Error;

    use crate::StandaloneContract;

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error("{0}")]
        Abstract(#[from] abstract_std::AbstractError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),

        #[error("{0}")]
        Admin(#[from] AdminError),
    }

    pub type MockStandaloneContract = StandaloneContract;

    pub const BASIC_MOCK_STANDALONE: MockStandaloneContract =
        MockStandaloneContract::new(TEST_MODULE_ID, TEST_VERSION, None);

    // Easy way to see if an ibc-callback was actually received.
    pub const IBC_CALLBACK_RECEIVED: Item<bool> = Item::new("ibc_callback_received");
    pub fn standalone_base_mock_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default()
            .with_smart_handler(TEST_MODULE_FACTORY, |_msg| panic!("unexpected messsage"))
    }

    #[cosmwasm_std::entry_point]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: MockInitMsg,
    ) -> Result<Response, MockError> {
        BASIC_MOCK_STANDALONE.instantiate(
            deps,
            standalone::BaseInstantiateMsg {
                ans_host_address: msg.ans_host_address,
                version_control_address: msg.version_control_address,
            },
        )?;
        Ok(BASIC_MOCK_STANDALONE.response("instantiate"))
    }

    #[cosmwasm_std::entry_point]
    pub fn execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: MockExecMsg,
    ) -> Result<Response, MockError> {
        Ok(BASIC_MOCK_STANDALONE
            .response("mock_exec")
            .set_data("mock_exec".as_bytes()))
    }

    #[cosmwasm_std::entry_point]
    pub fn query(_deps: Deps, _env: Env, _msg: MockQueryMsg) -> StdResult<Binary> {
        to_json_binary(&MockQueryResponse {})
    }

    #[cosmwasm_std::entry_point]
    pub fn migrate(deps: DepsMut, _env: Env, _msg: MockMigrateMsg) -> Result<Response, MockError> {
        BASIC_MOCK_STANDALONE.migrate(deps)?;
        Ok(BASIC_MOCK_STANDALONE.response("migrate"))
    }

    /// Instantiate the contract with the default [`TEST_MODULE_FACTORY`].
    /// This will set the [`abstract_testing::addresses::TEST_MANAGER`] as the admin.
    pub fn mock_init() -> MockDeps {
        let mut deps = mock_dependencies();
        let _info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = standalone_base_mock_querier().build();

        let msg = MockInitMsg {
            ans_host_address: TEST_ANS_HOST.to_string(),
            version_control_address: TEST_VERSION_CONTROL.to_string(),
        };

        BASIC_MOCK_STANDALONE
            .instantiate(
                deps.as_mut(),
                standalone::BaseInstantiateMsg {
                    ans_host_address: msg.ans_host_address,
                    version_control_address: msg.version_control_address,
                },
            )
            .unwrap();

        deps
    }

    #[cw_orch::interface(MockInitMsg, MockExecMsg, MockQueryMsg, MockMigrateMsg)]
    pub struct MockStandaloneI<Chain>;

    impl<T: cw_orch::prelude::CwEnv> abstract_interface::AppDeployer<T> for MockStandaloneI<T> {}

    impl<T: cw_orch::prelude::CwEnv> Uploadable for MockStandaloneI<T> {
        fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
            Box::new(
                ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query)
                    .with_migrate(self::migrate),
            )
        }
    }

    #[macro_export]
    macro_rules! gen_app_mock {
    ($name:ident,$id:expr, $version:expr, $deps:expr) => {
        use $crate::std::app;
        use ::abstract_app::mock::{MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg};
        use ::cw_orch::prelude::*;
        use $crate::sdk::base::Handler;
        use $crate::sdk::features::AccountIdentification;
        use $crate::sdk::{Execution, TransferInterface};


        type Exec = app::ExecuteMsg<MockExecMsg, MockReceiveMsg>;
        type Query = app::QueryMsg<MockQueryMsg>;
        type Init = app::InstantiateMsg<MockInitMsg>;
        type Migrate = app::MigrateMsg<MockMigrateMsg>;
        const MOCK_APP_WITH_DEP: ::abstract_app::mock::MockAppContract = ::abstract_app::mock::MockAppContract::new($id, $version, None)
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
                let adapter1_addr = $crate::std::manager::state::ACCOUNT_MODULES.query(&deps.querier,manager, "tester:mock-adapter1")?;
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
                            &adapter1_addr.unwrap(),
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
            msg: <::abstract_app::mock::MockAppContract as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::InstantiateEndpoint;
            MOCK_APP_WITH_DEP.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        fn mock_execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_app::mock::MockAppContract as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ExecuteEndpoint;
            MOCK_APP_WITH_DEP.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        fn mock_query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_app::mock::MockAppContract as $crate::sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <::abstract_app::mock::MockAppContract as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::QueryEndpoint;
            MOCK_APP_WITH_DEP.query(deps, env, msg)
        }

        fn mock_migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_app::mock::MockAppContract as $crate::sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_app::mock::MockAppContract as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::MigrateEndpoint;
            MOCK_APP_WITH_DEP.migrate(deps, env, msg)
        }

        #[cw_orch::interface(Init, Exec, Query, Migrate)]
        pub struct $name;

        impl<T: cw_orch::prelude::CwEnv> ::abstract_interface::AppDeployer<T> for $name <T> {}

        impl<T: cw_orch::prelude::CwEnv> Uploadable for $name<T> {
            fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
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
