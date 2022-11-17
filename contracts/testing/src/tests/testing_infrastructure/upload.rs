use std::collections::HashMap;

use abstract_sdk::os::{
    objects::module_reference::ModuleReference, ANS_HOST, MANAGER, MODULE_FACTORY, OS_FACTORY,
    PROXY, VERSION_CONTROL,
};
use cw_multi_test::{App, ContractWrapper};

use super::{common_integration::NativeContracts, instantiate::init_native_contracts};

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
pub fn upload_base_contracts(app: &mut App) -> (HashMap<String, u64>, NativeContracts) {
    let mut code_ids: HashMap<String, u64> = HashMap::new();
    let mut modules: HashMap<String, ModuleReference> = HashMap::new();

    // Instantiate Token Contract
    let cw20_token_contract = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));
    let cw20_token_code_id = app.store_code(cw20_token_contract);
    code_ids.insert("cw_plus:cw20".into(), cw20_token_code_id);

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
    code_ids.insert(PROXY.into(), proxy_code_id);
    modules.insert(PROXY.into(), ModuleReference::App(proxy_code_id));

    // Upload AnsHost Contract
    let ans_host_contract = Box::new(ContractWrapper::new_with_empty(
        ans_host::contract::execute,
        ans_host::contract::instantiate,
        ans_host::contract::query,
    ));

    let ans_host_code_id = app.store_code(ans_host_contract);
    code_ids.insert(ANS_HOST.into(), ans_host_code_id);

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
    code_ids.insert(VERSION_CONTROL.into(), version_control_code_id);

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
    code_ids.insert(OS_FACTORY.into(), os_factory_code_id);

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
    code_ids.insert(MODULE_FACTORY.into(), module_factory_code_id);

    // Upload manager Contract
    let manager_contract = Box::new(
        ContractWrapper::new_with_empty(
            manager::contract::execute,
            manager::contract::instantiate,
            manager::contract::query,
        )
        .with_migrate_empty(manager::contract::migrate),
    );

    let manager_code_id = app.store_code(manager_contract);
    code_ids.insert(MANAGER.into(), manager_code_id);
    modules.insert(MANAGER.into(), ModuleReference::App(manager_code_id));

    let native_contracts = init_native_contracts(app, &code_ids, &modules);
    (code_ids, native_contracts)
}
