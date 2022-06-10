use std::collections::HashMap;

use abstract_os::core::modules::ModuleInfo;
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::Addr;

use cw_multi_test::{App, AppBuilder, BankKeeper, TerraMock};

pub struct NativeContracts {
    pub token: Addr,
    pub memory: Addr,
    pub version_control: Addr,
    pub os_factory: Addr,
    pub module_factory: Addr,
}

pub struct OsInstance {
    pub manager: Addr,
    pub proxy: Addr,
    pub modules: HashMap<String, ModuleInfo>,
}

pub fn mock_app() -> App {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let custom = TerraMock::luna_ust_case();

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .with_custom(custom)
        .build()
}
