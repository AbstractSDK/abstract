use abstract_sdk::std::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    proxy::state::{ADMIN, STATE},
    IBC_CLIENT,
};
use abstract_std::ICA_CLIENT;
use cosmwasm_std::{
    wasm_execute, Binary, CosmosMsg, DepsMut, Empty, MessageInfo, StdError, SubMsg, WasmQuery,
};

use crate::{
    contract::{ProxyResponse, ProxyResult, RESPONSE_REPLY_ID},
    error::ProxyError,
};

const LIST_SIZE_LIMIT: usize = 15;

/// Executes `Vec<CosmosMsg>` on the proxy.
/// Permission: Module
pub fn execute_module_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    Ok(ProxyResponse::action("execute_module_action").add_messages(msgs))
}

/// Executes `CosmosMsg` on the proxy and forwards its response.
/// Permission: Module
pub fn execute_module_action_response(
    deps: DepsMut,
    msg_info: MessageInfo,
    msg: CosmosMsg<Empty>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    let submsg = SubMsg::reply_on_success(msg, RESPONSE_REPLY_ID);

    Ok(ProxyResponse::action("execute_module_action_response").add_submessage(submsg))
}

/// Executes IBC actions on the IBC client.
/// Permission: Module
pub fn execute_ibc_action(deps: DepsMut, msg_info: MessageInfo, msg: IbcClientMsg) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }
    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ibc_client_address = abstract_sdk::std::manager::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address, IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on manager. Add it under the {IBC_CLIENT} name."
            ))
        })?;

    let funds_to_send = if let IbcClientMsg::SendFunds { funds, .. } = &msg {
        funds.clone()
    } else {
        vec![]
    };
    let client_msg = wasm_execute(ibc_client_address, &msg, funds_to_send)?;

    Ok(ProxyResponse::action("execute_ibc_action").add_message(client_msg))
}

/// Execute an action on an ICA.
/// Permission: Module
///
/// This function queries the `abstract:ica-client` contract from the account's manager.
/// It then fires a smart-query on that address of type [`QueryMsg::IcaAction`](abstract_ica::msg::QueryMsg).
///
/// The resulting `Vec<CosmosMsg>` are then executed on the proxy contract.
pub fn ica_action(deps: DepsMut, msg_info: MessageInfo, action_query: Binary) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ica_client_address = abstract_sdk::std::manager::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address, ICA_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ica_client not found on manager. Add it under the {ICA_CLIENT} name."
            ))
        })?;

    let res: abstract_ica::msg::IcaActionResult = deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: ica_client_address.into(),
            msg: action_query,
        }
        .into(),
    )?;

    Ok(ProxyResponse::action("ica_action").add_messages(res.msgs))
}

/// Add a contract to the whitelist
pub fn add_modules(deps: DepsMut, msg_info: MessageInfo, modules: Vec<String>) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if state.modules.len() >= LIST_SIZE_LIMIT {
        return Err(ProxyError::ModuleLimitReached {});
    }

    for module in modules.iter() {
        let module_addr = deps.api.addr_validate(module)?;

        if state.modules.contains(&module_addr) {
            return Err(ProxyError::AlreadyWhitelisted(module.clone()));
        }

        // Add contract to whitelist.
        state.modules.push(module_addr);
    }

    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(ProxyResponse::new(
        "add_module",
        vec![("modules", modules.join(","))],
    ))
}

/// Remove a contract from the whitelist
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    STATE.update(deps.storage, |mut state| {
        let module_address = deps.api.addr_validate(&module)?;

        if !state.modules.contains(&module_address) {
            return Err(ProxyError::NotWhitelisted(module.clone()));
        }
        // Remove contract from whitelist.
        state.modules.retain(|addr| *addr != module_address);
        Ok(state)
    })?;

    // Respond and note the change
    Ok(ProxyResponse::new(
        "remove_module",
        vec![("module", module)],
    ))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: &String) -> ProxyResult {
    let admin_addr = deps.api.addr_validate(admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(ProxyResponse::new(
        "set_admin",
        vec![
            ("previous_admin", previous_admin.to_string()),
            ("admin", admin.to_string()),
        ],
    ))
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::{contract::execute, test_common::*};
    use abstract_std::proxy::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MOCK_CONTRACT_ADDR},
        Addr, OwnedDeps, Storage,
    };
    use speculoos::prelude::*;

    const TEST_MODULE: &str = "module";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let base = test_account_base(deps.api);
        let info = message_info(&base.manager, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    fn load_modules(storage: &dyn Storage) -> Vec<Addr> {
        STATE.load(storage).unwrap().modules
    }

    mod add_module {
        use super::*;

        use cw_controllers::AdminError;

        #[test]
        fn only_admin_can_add_module() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            let msg = ExecuteMsg::AddModules {
                modules: vec![test_module_addr.to_string()],
            };
            let info = message_info(&deps.api.addr_make("not_admin"), &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::Admin(AdminError::NotAdmin {}))
        }

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            let msg = ExecuteMsg::AddModules {
                modules: vec![test_module_addr.to_string()],
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            // Plus manager
            assert_that(&actual_modules).has_length(2);
            assert_that(&actual_modules).contains(&test_module_addr);
        }

        #[test]
        fn fails_adding_previously_added_module() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            let msg = ExecuteMsg::AddModules {
                modules: vec![test_module_addr.to_string()],
            };

            let res = execute_as_admin(&mut deps, msg.clone());
            assert_that(&res).is_ok();

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::AlreadyWhitelisted(test_module_addr.to_string()));
        }

        #[test]
        fn fails_adding_module_when_list_is_full() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            let mut msg = ExecuteMsg::AddModules {
                modules: vec![test_module_addr.to_string()],
            };

            // -1 because manager counts as module as well
            for i in 0..LIST_SIZE_LIMIT - 1 {
                let test_module = format!("module_{i}");
                let test_module_addr = deps.api.addr_make(&test_module);
                msg = ExecuteMsg::AddModules {
                    modules: vec![test_module_addr.to_string()],
                };
                let res = execute_as_admin(&mut deps, msg.clone());
                assert_that(&res).is_ok();
            }

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::ModuleLimitReached {});
        }
    }

    type ProxyTestResult = Result<(), ProxyError>;

    mod remove_module {
        use abstract_std::proxy::state::State;
        use cw_controllers::AdminError;

        use super::*;

        #[test]
        fn only_admin() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };
            let info = message_info(&deps.api.addr_make("not_admin"), &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::Admin(AdminError::NotAdmin {}))
        }

        #[test]
        fn remove_module() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![test_module_addr.clone()],
                },
            )?;

            let msg = ExecuteMsg::RemoveModule {
                module: test_module_addr.to_string(),
            };
            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            assert_that(&actual_modules).is_empty();

            Ok(())
        }

        #[test]
        fn fails_removing_non_existing_module() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let test_module_addr = deps.api.addr_make(TEST_MODULE);
            let msg = ExecuteMsg::RemoveModule {
                module: test_module_addr.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::NotWhitelisted(test_module_addr.to_string()));
        }
    }

    mod execute_action {
        use abstract_std::proxy::state::State;

        use super::*;

        #[test]
        fn only_whitelisted_can_execute() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);

            let msg = ExecuteMsg::ModuleAction { msgs: vec![] };

            let info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::SenderNotWhitelisted {});
        }

        #[test]
        fn forwards_action() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);
            let base = test_account_base(deps.api);

            // stub a module
            let module_addr = deps.api.addr_make(TEST_MODULE);
            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![module_addr.clone()],
                },
            )?;

            let action: CosmosMsg = wasm_execute(
                MOCK_CONTRACT_ADDR.to_string(),
                // example garbage
                &ExecuteMsg::SetAdmin {
                    admin: base.manager.to_string(),
                },
                vec![],
            )?
            .into();

            let msg = ExecuteMsg::ModuleAction {
                msgs: vec![action.clone()],
            };

            // execute it AS the module
            let res = execute(
                deps.as_mut(),
                mock_env(),
                message_info(&module_addr, &[]),
                msg,
            );
            assert_that(&res).is_ok();

            let msgs = res.unwrap().messages;
            assert_that(&msgs).has_length(1);

            let msg = &msgs[0];
            assert_that(&msg.msg).is_equal_to(&action);

            Ok(())
        }
    }

    mod execute_ibc {
        use super::*;

        use abstract_std::{manager, proxy::state::State};
        use cosmwasm_std::coins;

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            STATE
                .save(
                    &mut deps.storage,
                    &State {
                        modules: vec![abstr.account.manager.clone()],
                    },
                )
                .unwrap();

            let msg = ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::Register {
                    host_chain: "juno".parse().unwrap(),
                    namespace: None,
                    install_modules: vec![],
                },
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = message_info(&abstr.account.manager, &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
            // mock enabling ibc
            let ibc_client_addr = deps.api.addr_make("ibc_client_addr");
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &abstr.account.manager,
                    manager::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, ibc_client_addr.clone()),
                )
                .build();

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: ibc_client_addr.to_string(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::Register {
                        host_chain: "juno".parse().unwrap(),
                        namespace: None,
                        install_modules: vec![],
                    })
                    .unwrap(),
                    funds: vec![],
                },
            )));
        }

        #[test]
        fn send_funds() {
            let mut deps = mock_dependencies();
            mock_init(&mut deps);
            let abstr = AbstractMockAddrs::new(deps.api);
            // whitelist creator
            STATE
                .save(
                    &mut deps.storage,
                    &State {
                        modules: vec![abstr.account.manager.clone()],
                    },
                )
                .unwrap();

            let funds = coins(10, "denom");
            let msg = ExecuteMsg::IbcAction {
                msg: abstract_std::ibc_client::ExecuteMsg::SendFunds {
                    host_chain: "juno".parse().unwrap(),
                    funds: funds.clone(),
                    memo: None,
                },
            };

            let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = message_info(&abstr.account.manager, &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
            // mock enabling ibc
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &abstr.account.manager,
                    manager::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, Addr::unchecked("ibc_client_addr")),
                )
                .build();

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "ibc_client_addr".into(),
                    msg: to_json_binary(&abstract_std::ibc_client::ExecuteMsg::SendFunds {
                        host_chain: "juno".parse().unwrap(),
                        funds: funds.clone(),
                        memo: None,
                    })
                    .unwrap(),
                    funds,
                },
            )));
        }
    }

    // TODO: uncomment
    // mod ica_action {
    //     use abstract_ica::msg::IcaActionResult;
    //     use abstract_std::{manager, proxy::state::State};

    //     use super::*;

    //     #[test]
    //     fn ica_action() {
    //         let mut deps = mock_dependencies();
    //         let abstr = AbstractMockAddrs::new(deps.api);
    //         let ica_client_addr = deps.api.addr_make("ica_client_addr");
    //         mock_init(&mut deps);
    //         // whitelist creator
    //         STATE
    //             .save(
    //                 &mut deps.storage,
    //                 &State {
    //                     modules: vec![abstr.account.manager.clone()],
    //                 },
    //             )
    //             .unwrap();

    //         let action = Binary::from(b"some_action");
    //         let msg = ExecuteMsg::IcaAction {
    //             action_query_msg: action.clone(),
    //         };

    //         let not_whitelisted_info = message_info(&deps.api.addr_make("not_whitelisted"), &[]);
    //         execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

    //         let manager_info = message_info(&abstr.account.manager, &[]);
    //         // ibc not enabled
    //         execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
    //         // mock enabling ibc
    //         deps.querier = MockQuerierBuilder::default()
    //             .with_contract_map_entry(
    //                 &abstr.account.manager,
    //                 manager::state::ACCOUNT_MODULES,
    //                 (ICA_CLIENT, ica_client_addr.clone()),
    //             )
    //             .with_smart_handler(&ica_client_addr, move |bin| {
    //                 if bin.eq(&action) {
    //                     Ok(to_json_binary(&IcaActionResult {
    //                         msgs: vec![CosmosMsg::Custom(Empty {})],
    //                     })
    //                     .unwrap())
    //                 } else {
    //                     Err("Unexpected action query".to_owned())
    //                 }
    //             })
    //             .build();

    //         let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
    //         assert_that(&res.messages).has_length(1);
    //         assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Custom(Empty {})));
    //     }
    // }
}
