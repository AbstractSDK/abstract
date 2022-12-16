use abstract_sdk::os::{api, api::BaseInstantiateMsg, manager as ManagerMsgs};

use abstract_sdk::os::{objects::module::ModuleInfo, EXCHANGE};

use abstract_sdk::os::objects::module::ModuleVersion;

use anyhow::Result as AnyResult;
use cosmwasm_std::{to_binary, Addr, Empty};
use cw_multi_test::{App, ContractWrapper, Executor};

use super::{
    common::{DEFAULT_VERSION, TEST_CREATOR},
    testing_infrastructure::env::{get_os_state, mock_app, register_api, AbstractEnv},
};

pub fn register_and_create_dex_api(
    app: &mut App,
    sender: &Addr,
    version_control: &Addr,
    ans_host: &Addr,
    version: Option<String>,
) -> AnyResult<()> {
    let module = ModuleInfo::from_id(
        EXCHANGE,
        abstract_sdk::os::objects::module::ModuleVersion::Version(
            version.unwrap_or(DEFAULT_VERSION.to_string()),
        ),
    )?;
    let contract = Box::new(ContractWrapper::new_with_empty(
        dex::contract::execute,
        dex::contract::instantiate,
        dex::contract::query,
    ));
    let code_id = app.store_code(contract);
    let msg = api::InstantiateMsg {
        base: BaseInstantiateMsg {
            ans_host_address: ans_host.to_string(),
            version_control_address: version_control.to_string(),
        },
        app: Empty {},
    };
    let api_addr = app
        .instantiate_contract(code_id, sender.clone(), &msg, &[], "api".to_owned(), None)
        .unwrap();
    register_api(app, sender, version_control, module, api_addr).unwrap();
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
        &env.native_contracts.ans_host,
        None,
    )
    .unwrap();
    app.execute_contract(
        sender.clone(),
        manager.clone(),
        &ManagerMsgs::ExecuteMsg::InstallModule {
            module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
            init_msg: Some(
                to_binary(&api::InstantiateMsg {
                    base: BaseInstantiateMsg {
                        ans_host_address: env.native_contracts.ans_host.to_string(),
                        version_control_address: env.native_contracts.version_control.to_string(),
                    },
                    app: Empty {},
                })
                .unwrap(),
            ),
        },
        &[],
    )
    .unwrap();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.ans_host,
        Some("0.1.1".into()),
    )
    .unwrap();

    let _os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    let _resp: abstract_sdk::os::version_control::ModuleResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_sdk::os::version_control::QueryMsg::Module {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
            },
        )
        .unwrap();

    app.execute_contract(
        sender.clone(),
        manager,
        &ManagerMsgs::ExecuteMsg::Upgrade {
            modules: vec![(
                ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
                None,
            )],
        },
        &[],
    )
    .unwrap();

    let _os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.ans_host,
        Some("0.0.1".into()),
    )
    .unwrap();
    let _resp: abstract_sdk::os::version_control::ModuleResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_sdk::os::version_control::QueryMsg::Module {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest {}).unwrap(),
            },
        )
        .unwrap();
}
