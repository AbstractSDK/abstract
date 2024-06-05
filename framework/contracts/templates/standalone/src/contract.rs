use crate::{error::MyStandaloneError, MY_STANDALONE_ID, STANDALONE_VERSION};

use abstract_standalone::StandaloneContract;
use cosmwasm_std::Response;

/// The type of the result returned by your app's entry points.
pub type MyStandaloneResult<T = Response> = Result<T, MyStandaloneError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type MyStandalone = StandaloneContract;

pub const MY_STANDALONE: MyStandalone =
    MyStandalone::new(MY_STANDALONE_ID, STANDALONE_VERSION, None);

#[cfg(not(target_arch = "wasm32"))]
pub mod interface {
    use cw_orch::contract::{interface_traits::InstantiableContract, Contract};
    use cw_orch::prelude::*;

    use crate::{msg::*, MY_STANDALONE};

    #[cw_orch::interface(
        MyStandaloneInstantiateMsg,
        MyStandaloneExecuteMsg,
        MyStandaloneQueryMsg,
        MyStandaloneMigrateMsg
    )]
    pub struct MyStandaloneInterface;

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::DependencyCreation
        for MyStandaloneInterface<Chain>
    {
        type DependenciesConfig = cosmwasm_std::Empty;
    }

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::RegisteredModule
        for MyStandaloneInterface<Chain>
    {
        type InitMsg = <MyStandaloneInterface<Chain> as InstantiableContract>::InstantiateMsg;

        fn module_id<'a>() -> &'a str {
            MY_STANDALONE.module_id()
        }

        fn module_version<'a>() -> &'a str {
            MY_STANDALONE.version()
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> From<Contract<Chain>> for MyStandaloneInterface<Chain> {
        fn from(value: Contract<Chain>) -> Self {
            MyStandaloneInterface(value)
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> Uploadable for MyStandaloneInterface<Chain> {
        fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
            let wasm_name = env!("CARGO_CRATE_NAME").replace('-', "_");
            cw_orch::prelude::ArtifactsDir::auto(Some(env!("CARGO_MANIFEST_DIR").to_string()))
                .find_wasm_path(&wasm_name)
                .unwrap()
        }

        fn wrapper() -> Box<dyn MockContract<Empty, Empty>> {
            use crate::handlers;

            Box::new(
                ContractWrapper::new_with_empty(
                    handlers::execute,
                    handlers::instantiate,
                    handlers::query,
                )
                .with_migrate(handlers::migrate),
            )
        }
    }

    impl<Chain: cw_orch::environment::CwEnv> abstract_interface::StandaloneDeployer<Chain>
        for MyStandaloneInterface<Chain>
    {
    }
}
