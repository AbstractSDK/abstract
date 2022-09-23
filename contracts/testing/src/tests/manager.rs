use abstract_os::{api::BaseInstantiateMsg, manager as ManagerMsgs, objects::module::Module};

use abstract_os::{objects::module::ModuleInfo, EXCHANGE};

use abstract_sdk::abstract_os::objects::module::ModuleVersion;
use anyhow::Result as AnyResult;
use cosmwasm_std::Addr;

use cw_multi_test::{App, ContractWrapper, Executor};

use super::common::DEFAULT_VERSION;
use super::{
    common::TEST_CREATOR,
    testing_infrastructure::env::{get_os_state, mock_app, register_api, AbstractEnv},
};

pub fn register_and_create_dex_api(
    app: &mut App,
    sender: &Addr,
    version_control: &Addr,
    memory: &Addr,
    version: Option<String>,
) -> AnyResult<()> {
    let module = ModuleInfo::from_id(
        EXCHANGE,
        abstract_os::objects::module::ModuleVersion::Version(
            version.unwrap_or(DEFAULT_VERSION.to_string()),
        ),
    )?;
    let contract = Box::new(ContractWrapper::new_with_empty(
        dex::contract::execute,
        dex::contract::instantiate,
        dex::contract::query,
    ));
    let code_id = app.store_code(contract);
    let msg = BaseInstantiateMsg {
        memory_address: memory.to_string(),
        version_control_address: version_control.to_string(),
    };
    let api_addr = app
        .instantiate_contract(code_id, sender.clone(), &msg, &[], "api".to_owned(), None)
        .unwrap();
    register_api(app, &sender, &version_control, module, api_addr).unwrap();
    Ok(())
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    // OS 0 has proxy and subscriber module
    assert_eq!(os_state.len(), 2);
    let manager = env.os_store.get(&0u32).unwrap().manager.clone();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.memory,
        None,
    )
    .unwrap();
    app.execute_contract(
        sender.clone(),
        manager.clone(),
        &ManagerMsgs::ExecuteMsg::CreateModule {
            module: Module {
                info: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
                kind: abstract_os::objects::module::ModuleKind::Extension,
            },
            init_msg: None,
        },
        &[],
    )
    .unwrap();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.memory,
        Some("0.1.1".into()),
    )
    .unwrap();

    let _os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    let _resp: abstract_os::version_control::ApiAddressResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_os::version_control::QueryMsg::ApiAddress {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
            },
        )
        .unwrap();

    app.execute_contract(
        sender.clone(),
        manager,
        &ManagerMsgs::ExecuteMsg::Upgrade {
            module: Module {
                info: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
                kind: abstract_os::objects::module::ModuleKind::Extension,
            },
            migrate_msg: None,
        },
        &[],
    )
    .unwrap();

    let _os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.memory,
        Some("0.0.1".into()),
    )
    .unwrap();
    let _resp: abstract_os::version_control::ApiAddressResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_os::version_control::QueryMsg::ApiAddress {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
            },
        )
        .unwrap();
}
