use std::str::FromStr;
use abstract_sdk::os::modules::Module;
use abstract_sdk::os::vault as vault_msg;
use abstract_sdk::os::ETF;
use abstract_sdk::os::{modules::ModuleInfo, registry::SUBSCRIPTION};
use anyhow::Result as AnyResult;
use cosmwasm_std::{Addr, BlockInfo, Decimal, Uint128, Uint64};
use cw_controllers::AdminError;
use cw_multi_test::{App, ContractWrapper, Executor};
use crate::tests::common::RANDOM_USER;
use crate::tests::testing_infrastructure::env::{exec_msg_on_manager, mint_tokens, token_balance};
use super::common::CW20;
use super::testing_infrastructure::env::{init_os, CoreActions};
use super::{
    common::TEST_CREATOR,
    testing_infrastructure::env::{get_os_state, mock_app, register_module, AbstractEnv},
};

#[test]
fn proper_initialization() {
    let mut app = mock_app();
    let sender = Addr::unchecked(TEST_CREATOR);
    let mut env = AbstractEnv::new(&mut app, &sender);

    let os_state = get_os_state(&app, &env.os_store, &0u32).unwrap();
    // upload vault contract
    let vault_contract = Box::new(
        ContractWrapper::new_with_empty(
            vault::contract::execute,
            proxy::contract::instantiate,
            proxy::contract::query,
        )
        .with_migrate_empty(proxy::contract::migrate),
    );
    let vault_info = ModuleInfo {
        name: ETF.into(),
        version: None,
    };
    register_module(
        &mut app,
        &sender,
        &mut env,
        vault_info.clone(),
        vault_contract,
    )
    .unwrap();
    // create second os
    init_os(&mut app, &sender, &mut env).unwrap();
    // add vault module, no defaults.
    let os_core = env.os_store.get(&1).unwrap();
    os_core
        .add_module(
            &mut app,
            &sender,
            Module {
                info: ModuleInfo {
                    name: ETF.into(),
                    version: None,
                },
                kind: abstract_sdk::os::modules::ModuleKind::App,
            },
            Some(vault_msg::InstantiateMsg {
                base: abstract_sdk::os::app::BaseInstantiateMsg {
                    ans_host_address: env.native_contracts.ans_host.to_string(),
                },
                deposit_asset: "test".into(),
                fee: Decimal::from_str("0.91").unwrap(),
                provider_addr: sender.to_string(),
                token_code_id: env.code_ids.get(CW20).unwrap().clone(),
                vault_lp_token_name: None,
                vault_lp_token_symbol: None,
            }),
        )
        .unwrap();
}
