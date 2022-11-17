use std::collections::HashMap;

use abstract_sdk::os::version_control::{Core, OsCoreResponse};
use cw_multi_test::App;

use super::common_integration::NativeContracts;
use abstract_sdk::os::*;

pub fn os_store_as_expected(
    app: &App,
    native_contracts: &NativeContracts,
    os_store: &HashMap<u32, Core>,
) -> bool {
    let resp: os_factory::ConfigResponse = app
        .wrap()
        .query_wasm_smart(
            &native_contracts.os_factory,
            &os_factory::QueryMsg::Config {},
        )
        .unwrap();
    let max_os_id = resp.next_os_id - 1;

    for os_id in 0..max_os_id {
        // Check OS
        let core: OsCoreResponse = app
            .wrap()
            .query_wasm_smart(
                &native_contracts.version_control,
                &version_control::QueryMsg::OsCore { os_id },
            )
            .unwrap();
        if core.os_core.ne(os_store.get(&os_id).unwrap()) {
            return false;
        }
    }
    true
}
