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
    use abstract_os::api::{self, BaseInstantiateMsg, InstantiateMsg};
    use abstract_sdk::{base::InstantiateEndpoint, AbstractSdkError};
    use abstract_testing::prelude::{
        TEST_ADMIN, TEST_ANS_HOST, TEST_MODULE_ID, TEST_VERSION, TEST_VERSION_CONTROL,
    };
    use boot_core::{BootEnvironment, ContractWrapper};
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        DepsMut, Empty, Env, MessageInfo, Response, StdError,
    };
    use thiserror::Error;

    pub const TEST_METADATA: &str = "test_metadata";
    pub const TEST_TRADER: &str = "test_trader";

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Api(#[from] ApiError),

        #[error("{0}")]
        Abstract(#[from] abstract_os::AbstractOsError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockApiExecMsg;

    impl api::ApiExecuteMsg for MockApiExecMsg {}

    /// Mock API type
    pub type MockApi = ApiContract<MockError, Empty, MockApiExecMsg, Empty>;
    type ExecuteMsg = api::ExecuteMsg<MockApiExecMsg>;

    /// use for testing
    pub const MOCK_API: MockApi = MockApi::new(TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
        .with_execute(|_, _, _, _, _| Ok(Response::new().set_data("mock_response".as_bytes())))
        .with_instantiate(mock_init_handler);

    pub type ApiMockResult = Result<(), MockError>;
    // export these for upload usage
    crate::export_endpoints!(MOCK_API, MockApi);

    pub fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
        let api = MOCK_API;
        let info = mock_info(TEST_ADMIN, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            app: Empty {},
        };
        api.instantiate(deps, mock_env(), info, init_msg)
    }

    fn mock_init_handler(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _api: MockApi,
        _msg: Empty,
    ) -> Result<Response, MockError> {
        Ok(Response::new().set_data("mock_response".as_bytes()))
    }

    #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, api::QueryMsg, Empty)]
    pub struct BootMockApi;

    impl<Chain: BootEnvironment> ApiDeployer<Chain, Empty> for BootMockApi<Chain> {}

    impl<Chain: boot_core::BootEnvironment> BootMockApi<Chain> {
        pub fn new(name: &str, chain: Chain) -> Self {
            Self(boot_core::Contract::new(name, chain).with_mock(Box::new(
                ContractWrapper::new_with_empty(self::execute, self::instantiate, self::query),
            )))
        }
    }
}
