use abstract_standalone::objects::dependency::StaticDependency;
use abstract_standalone::traits::Dependencies;
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

    fn dependencies<'a>() -> &'a [StaticDependency] {
        MY_STANDALONE.dependencies()
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
        Box::new(
            ContractWrapper::new_with_empty(
                crate::contract::execute,
                crate::contract::instantiate,
                crate::contract::query,
            )
            .with_migrate(crate::contract::migrate),
        )
    }
}

impl<Chain: cw_orch::environment::CwEnv> abstract_interface::StandaloneDeployer<Chain>
    for MyStandaloneInterface<Chain>
{
}
