use abstract_adapter::{
    gen_adapter_mock,
    mock::{MockError as AdapterMockError, MockInitMsg},
    AdapterContract,
};
use abstract_app::{gen_app_mock, mock::MockError as AppMockError, AppContract};
use abstract_core::objects::dependency::StaticDependency;
use abstract_interface::{AdapterDeployer, AppDeployer, DeployStrategy};
use cw_orch::prelude::*;

pub type MockAdapterContract = AdapterContract<AdapterMockError, Empty, Empty, Empty, Empty, Empty>;
pub type MockAppContract = AppContract<AppMockError, Empty, Empty, Empty, Empty, Empty, Empty>;

pub const V1: &str = "1.0.0";
pub const V2: &str = "2.0.0";

/// deploys different version adapters and app for migration testing
pub fn deploy_modules<T: CwEnv>(chain: &T) {
    adapter_1::MockAdapterI1V1::new_test(chain.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg {}, DeployStrategy::Error)
        .unwrap();

    // do same for version 2
    adapter_1::MockAdapterI1V2::new_test(chain.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg {}, DeployStrategy::Error)
        .unwrap();

    // and now for adapter 2
    adapter_2::MockAdapterI2V1::new_test(chain.clone())
        .deploy(V1.parse().unwrap(), MockInitMsg {}, DeployStrategy::Error)
        .unwrap();

    // do same for version 2
    adapter_2::MockAdapterI2V2::new_test(chain.clone())
        .deploy(V2.parse().unwrap(), MockInitMsg {}, DeployStrategy::Error)
        .unwrap();

    // and now for app 1
    app_1::MockAppI1V1::new_test(chain.clone())
        .deploy(V1.parse().unwrap(), DeployStrategy::Error)
        .unwrap();

    // do same for version 2
    app_1::MockAppI1V2::new_test(chain.clone())
        .deploy(V2.parse().unwrap(), DeployStrategy::Error)
        .unwrap();
}

pub mod adapter_1 {
    use super::*;

    pub const MOCK_ADAPTER_ID: &str = "tester:mock-adapter1";

    pub use self::{v1::*, v2::*};

    pub mod v1 {
        use super::*;
        gen_adapter_mock!(MockAdapterI1V1, MOCK_ADAPTER_ID, "1.0.0", &[]);
    }

    pub mod v2 {
        use super::*;
        gen_adapter_mock!(MockAdapterI1V2, MOCK_ADAPTER_ID, "2.0.0", &[]);
    }
}

pub mod adapter_2 {
    use super::*;

    pub const MOCK_ADAPTER_ID: &str = "tester:mock-adapter2";

    pub use self::{v0_1_0::*, v1::*, v2_0_0::*};

    pub mod v1 {
        use super::*;
        gen_adapter_mock!(MockAdapterI2V1, MOCK_ADAPTER_ID, "1.0.0", &[]);
    }

    pub mod v2_0_0 {
        use super::*;
        gen_adapter_mock!(MockAdapterI2V2, MOCK_ADAPTER_ID, "2.0.0", &[]);
    }

    pub mod v0_1_0 {
        use super::*;
        gen_adapter_mock!(MockAdapterI2V0_1_0, MOCK_ADAPTER_ID, "0.1.0", &[]);
    }
}

// app 1 depends on adapter 1 and adapter 2
pub mod app_1 {
    pub use v1::*;
    pub use v2::*;

    use super::*;
    pub const MOCK_APP_ID: &str = "tester:mock-app1";
    pub mod v1 {
        use super::*;
        gen_app_mock!(
            MockAppI1V1,
            MOCK_APP_ID,
            "1.0.0",
            &[
                StaticDependency::new(adapter_1::MOCK_ADAPTER_ID, &[V1]),
                StaticDependency::new(adapter_2::MOCK_ADAPTER_ID, &[V1]),
            ]
        );
    }

    pub mod v2 {
        use super::*;
        gen_app_mock!(
            MockAppI1V2,
            MOCK_APP_ID,
            "2.0.0",
            &[
                StaticDependency::new(adapter_1::MOCK_ADAPTER_ID, &[V2]),
                StaticDependency::new(adapter_2::MOCK_ADAPTER_ID, &[V2]),
            ]
        );
    }
}

pub mod gen_mock {

    #[macro_export]
    /// Macro to generate a mock contract's interface. The cw2 expression (bool) is used to indicate if the cw2 state should be set or not.
    macro_rules! gen_standalone_mock {
        ($name:ident, $id:expr, $version:expr, $cw2:expr) => {
            #[cosmwasm_schema::cw_serde]
            pub struct MockMsg {}

            pub fn mock_instantiate(
                deps: ::cosmwasm_std::DepsMut,
                _env: ::cosmwasm_std::Env,
                _info: ::cosmwasm_std::MessageInfo,
                _msg: MockMsg,
            ) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Response> {
                if $cw2 {
                    ::cw2::set_contract_version(deps.storage, $id, $version)?;
                }
                Ok(::cosmwasm_std::Response::new())
            }

            /// Execute entrypoint
            pub fn mock_execute(
                _deps: ::cosmwasm_std::DepsMut,
                _env: ::cosmwasm_std::Env,
                _info: ::cosmwasm_std::MessageInfo,
                _msg: MockMsg,
            ) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Response> {
                Ok(::cosmwasm_std::Response::new())
            }

            /// Query entrypoint
            pub fn mock_query(
                _deps: ::cosmwasm_std::Deps,
                _env: ::cosmwasm_std::Env,
                _msg: MockMsg,
            ) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Binary> {
                Ok(::cosmwasm_std::Binary::default())
            }

            #[cw_orch::interface(MockMsg, MockMsg, MockMsg, Empty)]
            pub struct $name;

            impl<T: ::cw_orch::prelude::CwEnv> ::cw_orch::prelude::Uploadable for $name<T> {
                fn wrapper() -> <Mock as ::cw_orch::environment::TxHandler>::ContractSource {
                    Box::new(ContractWrapper::<MockMsg, _, _, _, _, _>::new_with_empty(
                        self::mock_execute,
                        self::mock_instantiate,
                        self::mock_query,
                    ))
                }
            }

            impl<Chain: ::cw_orch::environment::CwEnv> $name<Chain> {
                pub fn new_test(chain: Chain) -> Self {
                    Self(::cw_orch::contract::Contract::new($id, chain))
                }
            }
        };
    }
}

// this standalone have cw2
pub mod standalone_cw2 {
    use super::*;
    use crate::gen_standalone_mock;
    pub const MOCK_STANDALONE_ID: &str = "crate.io:mock-standalone1";
    pub const MOCK_STANDALONE_VERSION: &str = "1.0.0";

    gen_standalone_mock!(
        StandaloneCw2,
        MOCK_STANDALONE_ID,
        MOCK_STANDALONE_VERSION,
        true
    );
}

// this standalone does not have cw2
pub mod standalone_no_cw2 {
    use super::*;
    use crate::gen_standalone_mock;
    pub const MOCK_STANDALONE_ID: &str = "crates.io:mock-standalone2";
    pub const MOCK_STANDALONE_VERSION: &str = "1.0.0";

    gen_standalone_mock!(
        StandaloneNoCw2,
        MOCK_STANDALONE_ID,
        MOCK_STANDALONE_VERSION,
        false
    );
}
