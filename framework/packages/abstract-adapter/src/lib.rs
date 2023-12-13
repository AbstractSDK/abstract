//! # Abstract Adapter
//!
//! Basis for an interfacing contract to an external service.
use cosmwasm_std::{Empty, Response};

pub type AdapterResult<C = Empty> = Result<Response<C>, AdapterError>;
// Default to Empty

pub use crate::state::AdapterContract;
pub use error::AdapterError;

pub mod endpoints;
pub mod error;
/// Abstract SDK trait implementations
pub mod features;
mod handler;
pub mod msgs;
#[cfg(feature = "schema")]
pub mod schema;
pub mod state;
mod traits;

#[cfg(feature = "test-utils")]
pub mod mock {
    use crate::{AdapterContract, AdapterError};
    use abstract_core::{
        adapter::{self, *},
        objects::dependency::StaticDependency,
    };
    use abstract_sdk::{
        base::InstantiateEndpoint,
        features::{CustomData, DepsType},
        AbstractSdkError,
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        to_json_binary, DepsMut, Empty, Env, MessageInfo, Response, StdError,
    };
    use cw_orch::prelude::*;
    use thiserror::Error;

    use abstract_interface::AdapterDeployer;

    crate::adapter_msg_types!(MockAdapterContract, MockExecMsg, MockQueryMsg);

    pub const TEST_METADATA: &str = "test_metadata";
    pub const TEST_AUTHORIZED_ADDRESS: &str = "test_authorized_address";

    #[derive(Error, Debug, PartialEq)]
    pub enum MockError {
        #[error("{0}")]
        Std(#[from] StdError),

        #[error(transparent)]
        Adapter(#[from] AdapterError),

        #[error("{0}")]
        Abstract(#[from] abstract_core::AbstractError),

        #[error("{0}")]
        AbstractSdk(#[from] AbstractSdkError),
    }

    #[cosmwasm_schema::cw_serde]
    pub struct MockInitMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockExecMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockQueryMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockReceiveMsg;

    #[cosmwasm_schema::cw_serde]
    pub struct MockSudoMsg;

    /// Mock Adapter type
    pub type MockAdapterContract<'a> = AdapterContract<
        'a,
        MockError,
        MockInitMsg,
        MockExecMsg,
        MockQueryMsg,
        MockReceiveMsg,
        MockSudoMsg,
    >;

    pub const MOCK_DEP: StaticDependency = StaticDependency::new("module_id", &[">0.0.0"]);

    /// use for testing
    pub fn mock_adapter(deps: DepsType) -> MockAdapterContract {
        MockAdapterContract::new(deps, TEST_MODULE_ID, TEST_VERSION, Some(TEST_METADATA))
            .with_instantiate(|app, _| {
                app.set_data("mock_init".as_bytes());
                Ok(())
            })
            .with_execute(|app, _| {
                app.set_data("mock_exec".as_bytes());
                Ok(())
            })
            .with_query(|app, _| to_json_binary("mock_query").map_err(Into::into))
            .with_sudo(|app, _| {
                app.set_data("mock_sudo".as_bytes());
                Ok(())
            })
            .with_receive(|app, _| {
                app.set_data("mock_receive".as_bytes());
                Ok(())
            })
            .with_ibc_callbacks(&[("c_id", |app, _, _, _| {
                app.set_data("mock_callback".as_bytes());
                Ok(())
            })])
            .with_replies(&[(1u64, |app, msg| {
                app.set_data(msg.result.unwrap().data.unwrap());
                Ok(())
            })])
    }

    pub type AdapterMockResult = Result<(), MockError>;
    // export these for upload usage
    crate::export_endpoints!(mock_adapter, MockAdapterContract);

    pub fn mock_init(deps: DepsMut) -> Result<Response, MockError> {
        let info = mock_info(OWNER, &[]);
        let adapter = mock_adapter((deps, mock_env(), info).into());
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            module: MockInitMsg,
        };
        adapter.instantiate(init_msg)
    }

    pub fn mock_init_custom<'a>(
        deps: DepsMut<'a>,
        adapter: fn(deps: DepsType<'a>) -> MockAdapterContract<'a>,
    ) -> Result<Response, MockError> {
        let info = mock_info(OWNER, &[]);
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            module: MockInitMsg,
        };
        let adapter = adapter((deps, mock_env(), info).into());
        adapter.instantiate(init_msg)
    }

    impl Uploadable for BootMockAdapter<Mock> {
        fn wrapper(&self) -> <Mock as cw_orch::environment::TxHandler>::ContractSource {
            Box::new(ContractWrapper::new_with_empty(
                self::execute,
                self::instantiate,
                self::query,
            ))
        }
    }

    type Exec = adapter::ExecuteMsg<MockExecMsg>;
    type Query = adapter::QueryMsg<MockQueryMsg>;
    type Init = adapter::InstantiateMsg<MockInitMsg>;

    #[cw_orch::interface(Init, Exec, Query, Empty)]
    pub struct BootMockAdapter<Chain>;

    impl AdapterDeployer<Mock, MockInitMsg> for BootMockAdapter<Mock> {}

    /// Generate a BOOT instance for a mock adapter
    /// - $name: name of the contract (&str)
    /// - $id: id of the contract (&str)
    /// - $version: version of the contract (&str)
    /// - $deps: dependencies of the contract (&[StaticDependency])
    #[macro_export]
    macro_rules! gen_adapter_mock {
    ($name:ident, $id:expr, $version:expr, $deps:expr) => {
        use ::abstract_core::adapter::*;
        use ::cosmwasm_std::Empty;
        use ::abstract_adapter::mock::{MockExecMsg, MockQueryMsg, MockReceiveMsg, MockInitMsg, MockAdapterContract, MockError};
        use ::cw_orch::environment::CwEnv;
        use ::abstract_sdk::features::CustomData;



    pub fn mock_app(deps: ::abstract_sdk::features::DepsType) -> ::abstract_adapter::mock::MockAdapterContract<'_> {
        return ::abstract_adapter::mock::MockAdapterContract::new(deps, $id, $version, None)
            .with_dependencies($deps)
            .with_execute(|app, _| {
                app.set_data("mock_exec".as_bytes());
                Ok(())
            })
    }


        fn instantiate<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::InstantiateEndpoint;
            mock_app((deps, env, info).into()).instantiate(msg)
        }

        /// Execute entrypoint
        fn execute<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            mock_app((deps, env, info).into()).execute(msg)
        }

        /// Query entrypoint
        fn query<'a>(
            deps: ::cosmwasm_std::Deps<'a>,
            env: ::cosmwasm_std::Env,
            msg: <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <::abstract_adapter::mock::MockAdapterContract<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            mock_app((deps, env).into()).query(msg)
        }

        type Exec = ::abstract_core::adapter::ExecuteMsg<MockExecMsg, MockReceiveMsg>;
        type Query = ::abstract_core::adapter::QueryMsg<MockQueryMsg>;
        type Init = ::abstract_core::adapter::InstantiateMsg<MockInitMsg>;
        #[cw_orch::interface(Init, Exec, Query, Empty)]
        pub struct $name ;

        impl ::abstract_interface::AdapterDeployer<::cw_orch::prelude::Mock, MockInitMsg> for $name <::cw_orch::prelude::Mock> {}

        impl Uploadable for $name<::cw_orch::prelude::Mock> {
            fn wrapper(&self) -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
                Box::new(ContractWrapper::<
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
                ))
            }
        }

        impl<Chain: ::cw_orch::environment::CwEnv> $name <Chain> {
            pub fn new_test(chain: Chain) -> Self {
                Self(
                    ::cw_orch::contract::Contract::new($id, chain),
                )
            }
        }
    };
}
}
