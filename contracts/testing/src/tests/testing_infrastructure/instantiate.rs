use super::common_integration::NativeContracts;
use crate::tests::common::{DEFAULT_VERSION, TEST_CREATOR};
use abstract_sdk::core::{
    account_factory as AccountFactoryMsg, ans_host as AnsHostMsg,
    module_factory as ModuleFactoryMsg,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    version_control::{self as VCMsg, ModulesResponse},
    ACCOUNT_FACTORY, ANS_HOST, MODULE_FACTORY, VERSION_CONTROL,
};
use cosmwasm_std::{Addr, Timestamp};
use cw_multi_test::{App, Executor};
use std::collections::HashMap;

/// Creates the basic contract instances needed to test the account.
///

pub fn init_native_contracts(
    app: &mut App,
    code_ids: &HashMap<String, u64>,
    modules: &HashMap<String, ModuleReference>,
) -> NativeContracts {
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
            *code_ids.get("cw_plus:cw20").unwrap(),
            owner.clone(),
            &msg,
            &[],
            String::from("TOKE"),
            None,
        )
        .unwrap();

    let ans_host_instantiate_msg = AnsHostMsg::InstantiateMsg {};

    // AnsHost contract
    let ans_host_instance = app
        .instantiate_contract(
            *code_ids.get(ANS_HOST).unwrap(),
            owner.clone(),
            &ans_host_instantiate_msg,
            &[],
            "AnsHost",
            None,
        )
        .unwrap();

    let version_control_msg = VCMsg::InstantiateMsg {};
    // Instantiate VC Contract
    let version_control_instance = app
        .instantiate_contract(
            *code_ids.get(VERSION_CONTROL).unwrap(),
            owner.clone(),
            &version_control_msg,
            &[],
            "version_control",
            None,
        )
        .unwrap();

    let module_factory_msg = ModuleFactoryMsg::InstantiateMsg {
        ans_host_address: ans_host_instance.to_string(),
        version_control_address: version_control_instance.to_string(),
    };
    // Instantiate module factory Contract
    let module_factory_instance = app
        .instantiate_contract(
            *code_ids.get(MODULE_FACTORY).unwrap(),
            owner.clone(),
            &module_factory_msg,
            &[],
            "module_factory",
            None,
        )
        .unwrap();

    let account_factory_msg = AccountFactoryMsg::InstantiateMsg {
        ans_host_address: ans_host_instance.to_string(),
        module_factory_address: module_factory_instance.to_string(),
        version_control_address: version_control_instance.to_string(),
    };
    // Instantiate account factory Contract
    let account_factory_instance = app
        .instantiate_contract(
            *code_ids.get(ACCOUNT_FACTORY).unwrap(),
            owner.clone(),
            &account_factory_msg,
            &[],
            "account_factory",
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
        modules,
        &DEFAULT_VERSION.to_string(),
        &version_control_instance,
        &account_factory_instance,
    );

    app.update_block(|b| {
        b.height += 1;
        b.time = Timestamp::from_seconds(1571797425);
    });

    NativeContracts {
        token: token_instance,
        ans_host: ans_host_instance,
        version_control: version_control_instance,
        account_factory: account_factory_instance,
        module_factory: module_factory_instance,
    }
}

fn add_contracts_to_version_control_and_set_factory(
    app: &mut App,
    owner: &Addr,
    code_ids: &HashMap<String, ModuleReference>,
    version: &String,
    version_control: &Addr,
    account_factory: &Addr,
) {
    let modules = code_ids
        .iter()
        .map(|(k, v)| {
            (
                ModuleInfo::from_id(k, ModuleVersion::Version(version.to_string())).unwrap(),
                v.clone(),
            )
        })
        .collect();

    let msg = VCMsg::ExecuteMsg::ProposeModules { modules };
    app.execute_contract(owner.clone(), version_control.clone(), &msg, &[])
        .unwrap();

    let resp: ModulesResponse = app
        .wrap()
        .query_wasm_smart(
            version_control,
            &VCMsg::QueryMsg::Modules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
    println!("{:?}", resp);
    let msg = VCMsg::ExecuteMsg::SetFactory {
        new_factory: account_factory.to_string(),
    };
    app.execute_contract(owner.clone(), version_control.clone(), &msg, &[])
        .unwrap();
}
