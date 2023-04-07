//! # Abstract api
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub type ApiResult<C = Empty> = Result<Response<C>, ApiError>;
// Default to Empty

pub use crate::state::ApiContract;
pub use error::ApiError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub mod schema;
pub mod state;

#[cfg(feature = "test-utils")]
pub mod mock {
    use crate::{ApiContract, ApiError};
    use abstract_boot::ApiDeployer;
    use abstract_core::{
        api::{self, *},
        objects::dependency::StaticDependency,
    };
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::prelude::{
        TEST_ADMIN, TEST_ANS_HOST, TEST_MODULE_ID, TEST_VERSION, TEST_VERSION_CONTROL,
    };
    use boot_core::{ContractWrapper, CwEnv};
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        to_binary, DepsMut, Empty, Response, StdError,
    };
    use thiserror::Error;

    pub const TEST_METADATA: &str = "test_metadata";
    pub const TEST_AUTHORIZED_ADDRESS: &str = "test_authorized_address";

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),

        #[error("{0}")]
        Abstract(#[from] abstract_core::AbstractError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockExecMsg;

    impl abstract_core::api::ApiExecuteMsg for MockExecMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryMsg;

    impl abstract_core::api::ApiQueryMsg for MockQueryMsg {}

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockSudoMsg;

    /// Mock API type
    pub type MockApiContract =
        ApiContract<MockError, MockInitMsg, MockExecMsg, MockQueryMsg, MockSudoMsg, MockReceiveMsg>;

    pub const MOCK_DEP: StaticDependency = StaticDependency::new("module_id", &[">0.0.0"]);

    /// use for testing
    pub const MOCK_API: MockApiContract =
        MockApiContract::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_instantiate(|_, _, _, _, _| Ok(Response::new().set_data("mock_init".as_bytes())))
            .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_exec".as_bytes())))
            .with_query(|_, _, _, _| to_binary("mock_query").map_err(Into::into))
            .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
            .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_receive".as_bytes())))
            .with_ibc_callbacks(&[("c_id", |_, _, _, _, _, _| {
                Ok(Response::new().set_data("mock_callback".as_bytes()))
            })])
            .with_replies(&[(1u64, |_, _, _, msg| {
                Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
            })]);

    pub type ApiMockResult = Result<(), MockError>;
    // export these for upload usage
    crate::export_endpoints!(MOCK_API, MockApiContract);

    pub fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
        let api = MOCK_API;
        let info = mock_info(TEST_ADMIN, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            module: MockInitMsg,
        };
        api.instantiate(deps, mock_env(), info, init_msg)
    }

    pub fn mock_init_custom(deps: DepsMut, api: MockApiContract) -> Result<Response, MockError> {
        let info = mock_info(TEST_ADMIN, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            module: MockInitMsg,
        };
        api.instantiate(deps, mock_env(), info, init_msg)
    }

    type Exec = api::ExecuteMsg<MockExecMsg>;
    type Query = api::QueryMsg<MockQueryMsg>;
    type Init = api::InstantiateMsg<MockInitMsg>;
    #[boot_core::contract(Init, Exec, Query, Empty)]
    pub struct BootMockApi;

    impl<Chain: CwEnv> ApiDeployer<Chain, MockInitMsg> for BootMockApi<Chain> {}

    impl<Chain: boot_core::CwEnv> BootMockApi<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(boot_core::Contract::new(name, chain).with_mock(Box::new(
                ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query),
            )))
        }
    }

    /// Generate a BOOT instance for a mock api
    /// - $name: name of the contract (&str)
    /// - $id: id of the contract (&str)
    /// - $version: version of the contract (&str)
    /// - $deps: dependencies of the contract (&[StaticDependency])
    #[macro_export]
    macro_rules! gen_api_mock {
    ($name:ident, $id:expr, $version:expr, $deps:expr) => {
        use ::abstract_core::api::*;
        use ::cosmwasm_std::Empty;
        use ::abstract_api::mock::{MockExecMsg, MockQueryMsg, MockReceiveMsg, MockInitMsg, MockApiContract, MockError};

        const MOCK_API: ::abstract_api::mock::MockApiContract = ::abstract_api::mock::MockApiContract::new($id, $version, None)
        .with_dependencies($deps);

        fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::InstantiateEndpoint;
            MOCK_API.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            MOCK_API.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <::abstract_api::mock::MockApiContract as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            MOCK_API.query(deps, env, msg)
        }

        type Exec = ::abstract_core::api::ExecuteMsg<MockExecMsg, MockReceiveMsg>;
        type Query = ::abstract_core::api::QueryMsg<MockQueryMsg>;
        type Init = ::abstract_core::api::InstantiateMsg<MockInitMsg>;
        #[boot_core::contract(Init, Exec, Query, Empty)]
        pub struct $name ;

        impl<Chain: ::boot_core::CwEnv> ::abstract_boot::ApiDeployer<Chain, MockInitMsg> for $name <Chain> {}

        impl<Chain: ::boot_core::CwEnv> $name <Chain> {
            pub fn new(chain: Chain) -> Self {
                Self(
                    ::boot_core::Contract::new($id, chain).with_mock(Box::new(::boot_core::ContractWrapper::<
                        Exec,
                        _,
                        _,
                        _,
                        _,
                        _,
                    >::new_with_empty(
                        self::execute,
                        self::instantiate,
                        self::query,
                    ))),
                )
            }
        }
    };
}
}
