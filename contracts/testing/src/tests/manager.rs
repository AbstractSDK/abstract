use super::{
    common::{DEFAULT_VERSION, TEST_CREATOR},
    testing_infrastructure::env::{get_account_state, mock_app, register_api, AbstractEnv},
};
use abstract_sdk::core::objects::module::ModuleVersion;
use abstract_sdk::core::{adapter::BaseInstantiateMsg, api, manager as ManagerMsgs};
use abstract_sdk::core::{objects::module::ModuleInfo, EXCHANGE};
use anyhow::Result as AnyResult;
use cosmwasm_std::{to_binary, Addr, Empty};
use cw_multi_test::{App, ContractWrapper, Executor};

pub fn register_and_create_dex_api(
    app: &mut App,
    sender: &Addr,
    version_control: &Addr,
    ans_host: &Addr,
    version: Option<String>,
) -> AnyResult<()> {
    let module = ModuleInfo::from_id(
        EXCHANGE,
        abstract_sdk::core::objects::module::ModuleVersion::Version(
            version.unwrap_or(DEFAULT_VERSION.to_string()),
        ),
    )?;
    let contract = Box::new(ContractWrapper::new_with_empty(
        dex::contract::execute,
        dex::contract::instantiate,
        dex::contract::query,
    ));
    let code_id = app.store_code(contract);
    let msg = adapter::InstantiateMsg {
        base: BaseInstantiateMsg {
            ans_host_address: ans_host.to_string(),
            version_control_address: version_control.to_string(),
        },
        app: Empty {},
    };
    let adapter_addr = app
        .instantiate_contract(code_id, sender.clone(), &msg, &[], "api".to_owned(), None)
        .unwrap();
    register_api(app, sender, version_control, module, adapter_addr).unwrap();
    Ok(())
}

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let env = AbstractEnv::new(&mut app, &sender);

    let account_state = get_account_state(&app, &env.account_store, &0u32).unwrap();

    // Account 0 has proxy and subscriber module
    assert_eq!(account_state.len(), 2);
    let manager = env.account_store.get(&0u32).unwrap().manager.clone();

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
            module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest).unwrap(),
            init_msg: Some(
                to_binary(&adapter::InstantiateMsg {
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

    let _account_state = get_account_state(&app, &env.account_store, &0u32).unwrap();

    let _resp: abstract_sdk::core::version_control::ModuleResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_sdk::core::version_control::QueryMsg::Module {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest).unwrap(),
            },
        )
        .unwrap();

    app.execute_contract(
        sender.clone(),
        manager,
        &ManagerMsgs::ExecuteMsg::Upgrade {
            modules: vec![(
                ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest).unwrap(),
                None,
            )],
        },
        &[],
    )
    .unwrap();

    let _account_state = get_account_state(&app, &env.account_store, &0u32).unwrap();

    register_and_create_dex_api(
        &mut app,
        &sender,
        &env.native_contracts.version_control,
        &env.native_contracts.ans_host,
        Some("0.0.1".into()),
    )
    .unwrap();
    let _resp: abstract_sdk::core::version_control::ModuleResponse = app
        .wrap()
        .query_wasm_smart(
            env.native_contracts.version_control.clone(),
            &abstract_sdk::core::version_control::QueryMsg::Module {
                module: ModuleInfo::from_id(EXCHANGE, ModuleVersion::Latest).unwrap(),
            },
        )
        .unwrap();
}
