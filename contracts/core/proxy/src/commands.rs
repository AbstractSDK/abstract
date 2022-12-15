use cosmwasm_std::{
    wasm_execute, CosmosMsg, DepsMut, Empty, MessageInfo, Order, Response, StdError,
};

use abstract_sdk::os::ibc_client::ExecuteMsg as IbcClientMsg;
use abstract_sdk::os::objects::proxy_asset::UncheckedProxyAsset;
use abstract_sdk::os::proxy::state::{ADMIN, ANS_HOST, STATE, VAULT_ASSETS};
use abstract_sdk::os::IBC_CLIENT;

use crate::contract::ProxyResult;
use crate::error::ProxyError;
use crate::queries::*;

const LIST_SIZE_LIMIT: usize = 15;

/// Executes actions forwarded by whitelisted contracts
/// This contracts acts as a proxy contract for the dApps
pub fn execute_module_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotWhitelisted {});
    }

    Ok(Response::new().add_messages(msgs))
}

/// Executes IBC actions forwarded by whitelisted contracts
/// Calls the messages on the IBC client (ensuring permission)
pub fn execute_ibc_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<IbcClientMsg>,
) -> ProxyResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .modules
        .contains(&deps.api.addr_validate(msg_info.sender.as_str())?)
    {
        return Err(ProxyError::SenderNotWhitelisted {});
    }
    let manager_address = ADMIN.get(deps.as_ref())?.unwrap();
    let ibc_client_address = abstract_sdk::os::manager::state::OS_MODULES
        .query(&deps.querier, manager_address, IBC_CLIENT)?
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "ibc_client not found on manager. Add it under the {} name.",
                IBC_CLIENT
            ))
        })?;
    let client_msgs: Result<Vec<_>, _> = msgs
        .into_iter()
        .map(|execute_msg| wasm_execute(&ibc_client_address, &execute_msg, vec![]))
        .collect();
    Ok(Response::new().add_messages(client_msgs?))
}

/// Update the stored vault asset information
pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<UncheckedProxyAsset>,
    to_remove: Vec<String>,
) -> ProxyResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let ans_host = &ANS_HOST.load(deps.storage)?;
    // Check the vault size to be within the size limit to prevent running out of gas when doing lookups
    let current_vault_size = VAULT_ASSETS
        .keys(deps.storage, None, None, Order::Ascending)
        .count();
    let delta: i128 = to_add.len() as i128 - to_remove.len() as i128;
    if current_vault_size as i128 + delta > LIST_SIZE_LIMIT as i128 {
        return Err(ProxyError::AssetsLimitReached {});
    }

    for new_asset in to_add.into_iter() {
        let checked_asset = new_asset.check(deps.as_ref(), ans_host)?;

        VAULT_ASSETS.save(deps.storage, checked_asset.asset.clone(), &checked_asset)?;
    }

    for asset_id in to_remove {
        VAULT_ASSETS.remove(deps.storage, asset_id.into());
    }

    // Check validity of new configuration
    let validity_result = query_proxy_asset_validity(deps.as_ref())?;
    if validity_result.missing_dependencies.is_some()
        || validity_result.unresolvable_assets.is_some()
    {
        return Err(ProxyError::BadUpdate(format!("{:?}", validity_result)));
    }

    Ok(Response::new().add_attribute("action", "update_proxy_assets"))
}

/// Add a contract to the whitelist
pub fn add_module(deps: DepsMut, msg_info: MessageInfo, module: String) -> ProxyResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if state.modules.len() >= LIST_SIZE_LIMIT {
        return Err(ProxyError::ModuleLimitReached {});
    }

    let module_addr = deps.api.addr_validate(&module)?;

    if state.modules.contains(&module_addr) {
        return Err(ProxyError::AlreadyWhitelisted(module));
    }

    // Add contract to whitelist.
    state.modules.push(module_addr);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", module))
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
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", module))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::testing::{
        mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
    };
    use cosmwasm_std::{Addr, OwnedDeps, Storage};
    use speculoos::prelude::*;

    use abstract_os::proxy::{ExecuteMsg, InstantiateMsg};

    use crate::contract::{execute, instantiate};

    use super::*;

    const TEST_MODULE: &str = "module";
    const TEST_CREATOR: &str = "creator";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_CREATOR, &[]);
        let msg = InstantiateMsg {
            os_id: 0,
            ans_host_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let _res = instantiate(deps, mock_env(), info, msg).unwrap();
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let info = mock_info(TEST_CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    fn load_modules(storage: &dyn Storage) -> Vec<Addr> {
        STATE.load(storage).unwrap().modules
    }

    mod add_module {
        use cosmwasm_std::testing::mock_dependencies;
        use cosmwasm_std::Addr;
        use cw_controllers::AdminError;

        use super::*;

        #[test]
        fn only_admin_can_add_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
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

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            let res = execute_as_admin(&mut deps, msg);
            assert_that(&res).is_ok();

            let actual_modules = load_modules(&deps.storage);
            assert_that(&actual_modules).has_length(1);
            assert_that(&actual_modules).contains(&Addr::unchecked(TEST_MODULE));
        }

        #[test]
        fn fails_adding_previously_added_module() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut());

            let msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
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

            let mut msg = ExecuteMsg::AddModule {
                module: TEST_MODULE.to_string(),
            };

            for i in 0..LIST_SIZE_LIMIT {
                msg = ExecuteMsg::AddModule {
                    module: format!("module_{}", i),
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
        use cosmwasm_std::testing::mock_dependencies;
        use cosmwasm_std::Addr;
        use cw_controllers::AdminError;

        use abstract_os::proxy::state::State;

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
        use super::*;
        use abstract_os::proxy::state::State;

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
                    admin: TEST_CREATOR.to_string(),
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
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: &String) -> Result<Response, ProxyError> {
    let admin_addr = deps.api.addr_validate(admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
