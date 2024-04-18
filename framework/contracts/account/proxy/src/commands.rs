use abstract_core::objects::{oracle::Oracle, price_source::UncheckedPriceSource, AssetEntry};
use abstract_sdk::core::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    proxy::state::{ADMIN, ANS_HOST, STATE},
    IBC_CLIENT,
};
use cosmwasm_std::{wasm_execute, CosmosMsg, DepsMut, Empty, MessageInfo, StdError, SubMsg};

use crate::{
    contract::{ProxyResponse, ProxyResult, RESPONSE_REPLY_ID},
    error::ProxyError,
};

const LIST_SIZE_LIMIT: usize = 15;

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
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

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
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

/// Executes IBC actions forwarded by whitelisted contracts
/// Calls the messages on the IBC client (ensuring permission)
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<IbcClientMsg>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state.modules.contains(&msg_info.sender) {
        return Err(ProxyError::SenderNotWhitelisted {});
    }
    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ibc_client_address = abstract_sdk::core::manager::state::ACCOUNT_MODULES
        .query(&deps.querier, manager_address, IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on manager. Add it under the {IBC_CLIENT} name."
            ))
        })?;
    let client_msgs: Result<Vec<_>, _> = msgs
        .into_iter()
        .map(|execute_msg| {
            let funds_to_send = if let IbcClientMsg::SendFunds { funds, .. } = &execute_msg {
                funds.to_vec()
            } else {
                vec![]
            };
            wasm_execute(&ibc_client_address, &execute_msg, funds_to_send)
        })
        .collect();

    Ok(ProxyResponse::action("execute_ibc_action").add_messages(client_msgs?))
}

/// Update the stored vault asset information
pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
    to_remove: Vec<AssetEntry>,
) -> ProxyResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let ans_host = &ANS_HOST.load(deps.storage)?;

    let oracle = Oracle::new();
    oracle.update_assets(deps, ans_host, to_add, to_remove)?;
    Ok(ProxyResponse::action("update_proxy_assets"))
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
    use super::*;

    use crate::{contract::execute, test_common::*};
    use abstract_core::proxy::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MOCK_CONTRACT_ADDR},
        Addr, OwnedDeps, Storage,
    };
    use speculoos::prelude::*;

    const TEST_MODULE: &str = "module";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let info = mock_info(TEST_MANAGER, &[]);
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
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModules {
                modules: vec![TEST_MODULE.to_string()],
            };
            let info = mock_info("not_admin", &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::Admin(AdminError::NotAdmin {}))
        }

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModules {
                modules: vec![TEST_MODULE.to_string()],
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            // Plus manager
            assert_that(&actual_modules).has_length(2);
            assert_that(&actual_modules).contains(&Addr::unchecked(TEST_MODULE));
        }

        #[test]
        fn fails_adding_previously_added_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModules {
                modules: vec![TEST_MODULE.to_string()],
            };

            let res = execute_as_admin(&mut deps, msg.clone());
            assert_that(&res).is_ok();

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::AlreadyWhitelisted(TEST_MODULE.to_string()));
        }

        #[test]
        fn fails_adding_module_when_list_is_full() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let mut msg = ExecuteMsg::AddModules {
                modules: vec![TEST_MODULE.to_string()],
            };

            // -1 because manager counts as module as well
            for i in 0..LIST_SIZE_LIMIT - 1 {
                msg = ExecuteMsg::AddModules {
                    modules: vec![format!("module_{i}")],
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
        use abstract_core::proxy::state::State;
        use cw_controllers::AdminError;

        use super::*;

        #[test]
        fn only_admin() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };
            let info = mock_info("not_admin", &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::Admin(AdminError::NotAdmin {}))
        }

        #[test]
        fn remove_module() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![Addr::unchecked(TEST_MODULE)],
                },
            )?;

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
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
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::RemoveModule {
                module: TEST_MODULE.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::NotWhitelisted(TEST_MODULE.to_string()));
        }
    }

    mod execute_action {
        use abstract_core::proxy::state::State;

        use super::*;

        #[test]
        fn only_whitelisted_can_execute() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::ModuleAction { msgs: vec![] };

            let info = mock_info("not_whitelisted", &[]);

            let res = execute(deps.as_mut(), mock_env(), info, msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ProxyError::SenderNotWhitelisted {});
        }

        #[test]
        fn forwards_action() -> ProxyTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            // stub a module
            STATE.save(
                &mut deps.storage,
                &State {
                    modules: vec![Addr::unchecked(TEST_MODULE)],
                },
            )?;

            let action: CosmosMsg = wasm_execute(
                MOCK_CONTRACT_ADDR.to_string(),
                // example garbage
                &ExecuteMsg::SetAdmin {
                    admin: TEST_MANAGER.to_string(),
                },
                vec![],
            )?
            .into();

            let msg = ExecuteMsg::ModuleAction {
                msgs: vec![action.clone()],
            };

            // execute it AS the module
            let res = execute(deps.as_mut(), mock_env(), mock_info(TEST_MODULE, &[]), msg);
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

        use abstract_core::{manager, proxy::state::State};
        use cosmwasm_std::coins;

        #[test]
        fn add_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());
            // whitelist creator
            STATE
                .save(
                    &mut deps.storage,
                    &State {
                        modules: vec![Addr::unchecked(TEST_MANAGER)],
                    },
                )
                .unwrap();

            let msg = ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register {
                    host_chain: "juno".into(),
                    base_asset: None,
                    namespace: None,
                    install_modules: vec![],
                }],
            };

            let not_whitelisted_info = mock_info(TEST_MANAGER, &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = mock_info(TEST_MANAGER, &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
            // mock enabling ibc
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_MANAGER,
                    manager::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, Addr::unchecked("ibc_client_addr")),
                )
                .build();

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "ibc_client_addr".into(),
                    msg: to_json_binary(&abstract_core::ibc_client::ExecuteMsg::Register {
                        host_chain: "juno".into(),
                        base_asset: None,
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
            mock_init(deps.as_mut());
            // whitelist creator
            STATE
                .save(
                    &mut deps.storage,
                    &State {
                        modules: vec![Addr::unchecked(TEST_MANAGER)],
                    },
                )
                .unwrap();

            let funds = coins(10, "denom");
            let msg = ExecuteMsg::IbcAction {
                msgs: vec![abstract_core::ibc_client::ExecuteMsg::SendFunds {
                    host_chain: "juno".to_owned(),
                    funds: funds.clone(),
                }],
            };

            let not_whitelisted_info = mock_info(TEST_MANAGER, &[]);
            execute(deps.as_mut(), mock_env(), not_whitelisted_info, msg.clone()).unwrap_err();

            let manager_info = mock_info(TEST_MANAGER, &[]);
            // ibc not enabled
            execute(deps.as_mut(), mock_env(), manager_info.clone(), msg.clone()).unwrap_err();
            // mock enabling ibc
            deps.querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_MANAGER,
                    manager::state::ACCOUNT_MODULES,
                    (IBC_CLIENT, Addr::unchecked("ibc_client_addr")),
                )
                .build();

            let res = execute(deps.as_mut(), mock_env(), manager_info, msg).unwrap();
            assert_that(&res.messages).has_length(1);
            assert_that!(res.messages[0]).is_equal_to(SubMsg::new(CosmosMsg::Wasm(
                cosmwasm_std::WasmMsg::Execute {
                    contract_addr: "ibc_client_addr".into(),
                    msg: to_json_binary(&abstract_core::ibc_client::ExecuteMsg::SendFunds {
                        host_chain: "juno".into(),
                        funds: funds.clone(),
                    })
                    .unwrap(),
                    funds,
                },
            )));
        }
    }
}
