use std::collections::HashMap;

use abstract_os::*;

use cw_multi_test::{App, ContractWrapper};

use super::common_integration::NativeContracts;
use super::instantiate::init_native_contracts;

/// Uploads:
/// - CW Token
///
/// -- Core --
/// - Treasury
/// - Manager
///
/// -- Native --
/// - AnsHost
/// - Module Factory
/// - Version Control
/// - Os Factory
pub fn upload_contracts(app: &mut App) -> (HashMap<&str, u64>, NativeContracts) {
    let mut code_ids: HashMap<&str, u64> = HashMap::new();

    // Instantiate Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    let cw20_token_code_id = app.store_code(cw20_token_contract);
    code_ids.insert("cw20", cw20_token_code_id);

    // Upload Treasury Contract
    let proxy_contract = Box::new(
        ContractWrapper::new_with_empty(
            proxy::contract::execute,
            proxy::contract::instantiate,
            proxy::contract::query,
        )
        .with_migrate_empty(proxy::contract::migrate),
    );
    let proxy_code_id = app.store_code(proxy_contract);
    code_ids.insert(PROXY, proxy_code_id);

    // Upload AnsHost Contract
    let ans_host_contract = Box::new(ContractWrapper::new_with_empty(
        ans_host::contract::execute,
        ans_host::contract::instantiate,
        ans_host::contract::query,
    ));

    let ans_host_code_id = app.store_code(ans_host_contract);
    code_ids.insert(ANS_HOST, ans_host_code_id);

    // Upload vc Contract
    let version_control_contract = Box::new(
        ContractWrapper::new_with_empty(
            version_control::contract::execute,
            version_control::contract::instantiate,
            version_control::contract::query,
        )
        .with_migrate_empty(version_control::contract::migrate),
    );

    let version_control_code_id = app.store_code(version_control_contract);
    code_ids.insert(VERSION_CONTROL, version_control_code_id);

    // Upload os_factory Contract
    let os_factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            os_factory::contract::execute,
            os_factory::contract::instantiate,
            os_factory::contract::query,
        )
        .with_reply_empty(os_factory::contract::reply),
    );

    let os_factory_code_id = app.store_code(os_factory_contract);
    code_ids.insert(OS_FACTORY, os_factory_code_id);

    // Upload module_factory Contract
    let module_factory_contract = Box::new(
        ContractWrapper::new_with_empty(
            module_factory::contract::execute,
            module_factory::contract::instantiate,
            module_factory::contract::query,
        )
        .with_reply_empty(module_factory::contract::reply),
    );

    let module_factory_code_id = app.store_code(module_factory_contract);
    code_ids.insert(MODULE_FACTORY, module_factory_code_id);

    // Upload manager Contract
    let manager_contract = Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_migrate_empty(crate::contract::migrate),
    );

    let manager_code_id = app.store_code(manager_contract);
    code_ids.insert(MANAGER, manager_code_id);
    let native_contracts = init_native_contracts(app, &code_ids);
    (code_ids, native_contracts)
}
