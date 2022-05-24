use std::collections::HashMap;

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::Addr;

use abstract_os::native::version_control::state::Core;
use terra_multi_test::{AppBuilder, BankKeeper, TerraApp, TerraMock};

use super::os_creation::{init_os, init_primary_os};
use super::upload::upload_base_contracts;

pub struct NativeContracts {
    pub token: Addr,
    pub memory: Addr,
    pub version_control: Addr,
    pub os_factory: Addr,
    pub module_factory: Addr,
}

pub fn mock_app() -> TerraApp {
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
