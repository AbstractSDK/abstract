mod endpoints;
pub mod features;
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

    pub type StandaloneTestResult = Result<(), MockError>;

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg {
        pub base: standalone::StandaloneInstantiateMsg,
        pub random_field: String,
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::ExecuteFns)]
    pub enum MockExecMsg {
        DoSomething {},
    }

    #[cosmwasm_schema::cw_serde]
    #[derive(cw_orch::QueryFns, QueryResponses)]
    pub enum MockQueryMsg {
        #[returns(MockQueryResponse)]
        GetSomething {},
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

    use abstract_sdk::{
        feature_objects::{AnsHost, VersionControlContract},
        AbstractResponse, AbstractSdkError,
    };
    use abstract_testing::{
        addresses::{TEST_ANS_HOST, TEST_PROXY, TEST_VERSION_CONTROL},
        prelude::*,
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
        info: MessageInfo,
        msg: MockInitMsg,
    ) -> Result<Response, MockError> {
        BASIC_MOCK_STANDALONE.instantiate(deps, info, msg.base, true)?;
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
    pub fn query(_deps: Deps, _env: Env, msg: MockQueryMsg) -> StdResult<Binary> {
        match msg {
            MockQueryMsg::GetSomething {} => to_json_binary(&MockQueryResponse {}),
        }
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

        BASIC_MOCK_STANDALONE
            .base_state
            .save(
                &mut deps.storage,
                &standalone::StandaloneState {
                    proxy_address: Addr::unchecked(TEST_PROXY),
                    ans_host: AnsHost::new(Addr::unchecked(TEST_ANS_HOST)),
                    version_control: VersionControlContract::new(Addr::unchecked(
                        TEST_VERSION_CONTROL,
                    )),
                    is_migratable: true,
                },
            )
            .unwrap();

        deps
    }

    #[cw_orch::interface(MockInitMsg, MockExecMsg, MockQueryMsg, MockMigrateMsg)]
    pub struct MockStandaloneI<Chain>;

    impl<T: cw_orch::prelude::CwEnv> abstract_interface::StandaloneDeployer<T> for MockStandaloneI<T> {}

    impl<T: cw_orch::prelude::CwEnv> Uploadable for MockStandaloneI<T> {
        fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
            Box::new(
                ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query)
                    .with_migrate(self::migrate),
            )
        }
    }

    #[macro_export]
    macro_rules! gen_standalone_mock {
        ($name:ident,$id:expr, $version:expr) => {
            use ::cw_orch::prelude::*;
            use $crate::mock::{
                MockExecMsg, MockInitMsg, MockMigrateMsg, MockQueryMsg, MockReceiveMsg,
            };
            use $crate::sdk::base::Handler;
            use $crate::sdk::features::AccountIdentification;
            use $crate::sdk::{Execution, TransferInterface};
            use $crate::std::standalone;
            use $crate::traits::AbstractResponse;

            const MOCK_APP_WITH_DEP: $crate::mock::MockStandaloneContract =
                $crate::mock::MockStandaloneContract::new($id, $version, None);

            fn mock_instantiate(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: $crate::mock::MockInitMsg,
            ) -> Result<::cosmwasm_std::Response, $crate::mock::MockError> {
                MOCK_APP_WITH_DEP.instantiate(deps, info, msg.base, true)?;
                Ok(MOCK_APP_WITH_DEP
                    .response("instantiate")
                    .set_data("mock_init".as_bytes()))
            }

            /// Execute entrypoint
            fn mock_execute(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: $crate::mock::MockExecMsg,
            ) -> Result<::cosmwasm_std::Response, $crate::mock::MockError> {
                match msg {
                    MockExecMsg::DoSomething {} => {}
                }
                Ok(MOCK_APP_WITH_DEP
                    .response("instantiate")
                    .set_data("mock_exec".as_bytes()))
            }

            /// Query entrypoint
            fn mock_query(
                deps: ::cosmwasm_std::Deps,
                env: ::cosmwasm_std::Env,
                msg: $crate::mock::MockQueryMsg,
            ) -> Result<::cosmwasm_std::Binary, ::cosmwasm_std::StdError> {
                match msg {
                    MockQueryMsg::GetSomething {} => {
                        ::cosmwasm_std::to_json_binary(&$crate::mock::MockQueryResponse {})
                    }
                }
            }

            fn mock_migrate(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                msg: $crate::mock::MockMigrateMsg,
            ) -> Result<::cosmwasm_std::Response, $crate::mock::MockError> {
                MOCK_APP_WITH_DEP.migrate(deps)?;
                Ok(MOCK_APP_WITH_DEP
                    .response("migrate")
                    .set_data("mock_migrate".as_bytes()))
            }

            #[cw_orch::interface(
                $crate::mock::MockInitMsg,
                $crate::mock::MockExecMsg,
                $crate::mock::MockQueryMsg,
                $crate::mock::MockMigrateMsg
            )]
            pub struct $name;

            impl<T: cw_orch::prelude::CwEnv> ::abstract_interface::StandaloneDeployer<T>
                for $name<T>
            {
            }

            impl<T: cw_orch::prelude::CwEnv> Uploadable for $name<T> {
                fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
                    Box::new(
                        ContractWrapper::<_, _, _, _, _, _>::new_with_empty(
                            self::mock_execute,
                            self::mock_instantiate,
                            self::mock_query,
                        )
                        .with_migrate(self::mock_migrate),
                    )
                }
            }

            impl<Chain: ::cw_orch::environment::CwEnv> $name<Chain> {
                pub fn new_test(chain: Chain) -> Self {
                    Self(cw_orch::contract::Contract::new($id, chain))
                }
            }
        };
    }
}
