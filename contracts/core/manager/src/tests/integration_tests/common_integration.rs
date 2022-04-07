use std::collections::HashMap;

use crate::tests::common::TEST_CREATOR;
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Addr, Empty, Timestamp, Uint128};
use pandora_os::core::modules::ModuleInfo;
use pandora_os::core::proxy::msg as TreasuryMsg;
use pandora_os::native::memory::msg as MemoryMsg;
use terra_mocks::TerraMockQuerier;
use terra_multi_test::{App, BankKeeper, ContractWrapper, Executor};
use terraswap::asset::{AssetInfo, PairInfo};

use super::upload::upload_contracts;

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

pub fn mock_app() -> App<Empty> {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let custom_querier: TerraMockQuerier =
        TerraMockQuerier::new(MockQuerier::new(&[(MOCK_CONTRACT_ADDR, &[])]));

    App::new(api, env.block, bank, MockStorage::new(), custom_querier)
    // let custom_handler = CachingCustomHandler::<CustomMsg, Empty>::new();
    // AppBuilder::new().with_custom(custom_handler).build()
}
