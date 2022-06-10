use std::collections::HashMap;

use abstract_os::native::version_control::state::Core;
use cw_multi_test::App;

use super::common_integration::NativeContracts;
use abstract_os::native::*;

pub fn os_store_as_expected(
    app: &App,
    native_contracts: &NativeContracts,
    os_store: &HashMap<u32, Core>,
) -> bool {
    let resp: os_factory::msg::ConfigResponse = app
        .wrap()
        .query_wasm_smart(
            &native_contracts.os_factory,
            &os_factory::msg::QueryMsg::Config {},
        )
        .unwrap();
    let max_os_id = resp.next_os_id - 1;

    for os_id in 0..max_os_id {
        // Check OS
        let core: Core = app
            .wrap()
            .query_wasm_smart(
                &native_contracts.version_control,
                &version_control::msg::QueryMsg::QueryOsAddress { os_id },
            )
            .unwrap();
        if core.ne(os_store.get(&os_id).unwrap()) {
            return false;
        }
    }
    true
}
