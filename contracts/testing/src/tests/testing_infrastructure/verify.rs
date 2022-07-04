use std::collections::HashMap;

use abstract_os::version_control::{Core, OsAddrResponse};
use cw_multi_test::App;

use super::common_integration::NativeContracts;
use abstract_os::*;

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
        let core: OsAddrResponse = app
            .wrap()
            .query_wasm_smart(
                &native_contracts.version_control,
                &version_control::QueryMsg::QueryOsAddress { os_id },
            )
            .unwrap();
        if core.os_address.ne(os_store.get(&os_id).unwrap()) {
            return false;
        }
    }
    true
}
