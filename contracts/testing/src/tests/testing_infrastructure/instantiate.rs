use std::collections::HashMap;

use crate::tests::common::{CHAIN_ID, DEFAULT_VERSION, TEST_CREATOR};

use cosmwasm_std::{Addr, Timestamp};

use abstract_os::{
    memory as MemoryMsg, module_factory as ModuleFactoryMsg, os_factory as OSFactoryMsg,
    version_control as VCMsg,
};
use abstract_os::{MEMORY, MODULE_FACTORY, OS_FACTORY, VERSION_CONTROL};

use cw_multi_test::{App, Executor};

use super::common_integration::NativeContracts;

/// Creates the basic contract instances needed to test the os.
///

pub fn init_native_contracts(app: &mut App, code_ids: &HashMap<String, u64>) -> NativeContracts {
    let owner = Addr::unchecked(TEST_CREATOR);
    // Instantiate Token Contract
    let msg = cw20_base::msg::InstantiateMsg {
        name: String::from("token"),
        symbol: String::from("TOKE"),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(cw20::MinterResponse {
            minter: owner.to_string(),
            cap: None,
        }),
        marketing: None,
    };

    let token_instance = app
        .instantiate_contract(
            code_ids.get("cw20").unwrap().clone(),
            owner.clone(),
            &msg,
            &[],
            String::from("TOKE"),
            None,
        )
        .unwrap();

    let memory_instantiate_msg = MemoryMsg::InstantiateMsg {};

    // Memory contract
    let memory_instance = app
        .instantiate_contract(
            code_ids.get(MEMORY).unwrap().clone(),
            owner.clone(),
            &memory_instantiate_msg,
            &[],
            "Memory",
            None,
        )
        .unwrap();

    let version_control_msg = VCMsg::InstantiateMsg {};
    // Instantiate VC Contract
    let version_control_instance = app
        .instantiate_contract(
            code_ids.get(VERSION_CONTROL).unwrap().clone(),
            owner.clone(),
            &version_control_msg,
            &[],
            "version_control",
            None,
        )
        .unwrap();

    let module_factory_msg = ModuleFactoryMsg::InstantiateMsg {
        memory_address: memory_instance.to_string(),
        version_control_address: version_control_instance.to_string(),
    };
    // Instantiate module factory Contract
    let module_factory_instance = app
        .instantiate_contract(
            code_ids.get(MODULE_FACTORY).unwrap().clone(),
            owner.clone(),
            &module_factory_msg,
            &[],
            "module_factory",
            None,
        )
        .unwrap();

    let os_factory_msg = OSFactoryMsg::InstantiateMsg {
        chain_id: CHAIN_ID.to_string(),
        memory_address: memory_instance.to_string(),
        module_factory_address: module_factory_instance.to_string(),
        version_control_address: version_control_instance.to_string(),
    };
    // Instantiate os factory Contract
    let os_factory_instance = app
        .instantiate_contract(
            code_ids.get(OS_FACTORY).unwrap().clone(),
            owner.clone(),
            &os_factory_msg,
            &[],
            "os_factory",
            None,
        )
        .unwrap();

    app.update_block(|b| {
        b.height += 17;
        b.time = Timestamp::from_seconds(1571797419);
    });

    add_contracts_to_version_control_and_set_factory(
        app,
        &owner,
        code_ids,
        &version_control_instance,
        &os_factory_instance,
    );

    app.update_block(|b| {
        b.height += 1;
        b.time = Timestamp::from_seconds(1571797425);
    });

    NativeContracts {
        token: token_instance,
        memory: memory_instance,
        version_control: version_control_instance,
        os_factory: os_factory_instance,
        module_factory: module_factory_instance,
    }
}

fn add_contracts_to_version_control_and_set_factory(
    app: &mut App,
    owner: &Addr,
    code_ids: &HashMap<String, u64>,
    version_control: &Addr,
    os_factory: &Addr,
) {
    for contract in code_ids {
        let msg = VCMsg::ExecuteMsg::AddCodeId {
            module: contract.0.clone(),
            version: DEFAULT_VERSION.to_string(),
            code_id: contract.1.clone(),
        };
        app.execute_contract(owner.clone(), version_control.clone(), &msg, &[])
            .unwrap();
    }
    let msg = VCMsg::ExecuteMsg::SetFactory {
        new_factory: os_factory.to_string(),
    };
    app.execute_contract(owner.clone(), version_control.clone(), &msg, &[])
        .unwrap();
}
