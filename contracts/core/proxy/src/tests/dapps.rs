use crate::contract::{execute, instantiate, query};
use crate::error::ProxyError;
use crate::tests::instantiate::execute_as_admin;
use abstract_sdk::os::proxy::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::from_binary;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cw_controllers::AdminError;
use speculoos::prelude::*;

use super::common::TEST_CREATOR;

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        os_id: 0,
        ans_host_address: "".into(),
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_that(&config.modules).is_empty();
}

#[test]
fn test_update_admin() -> Result<(), ProxyError> {
    let mut deps = mock_dependencies();
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::SetAdmin {
        admin: String::from("addr0001"),
    };
    let info = mock_info("addr0001", &[]);

    // Call as non-admin, should fail
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone());

    assert_that(&res)
        .is_err()
        .is_equal_to(&ProxyError::Admin(AdminError::NotAdmin {}));

    // Call as admin
    let res = execute_as_admin(&mut deps, msg.clone());

    assert_that(&res).is_ok();

    Ok(())
}

#[test]
fn test_add_dapp() {
    let mut deps = mock_dependencies();
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddModule {
        module: "addr420".to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(1, config.modules.len());
    assert_eq!("addr420", config.modules[0]);
}

#[test]
fn test_remove_dapp() {
    let mut deps = mock_dependencies();
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddModule {
        module: "addr420".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    assert_that(&res).is_ok();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(1, config.modules.len());
    // now remove dapp again.
    let msg = ExecuteMsg::RemoveModule {
        module: "addr420".to_string(),
    };
    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }
    // get dapp list and assert
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
    assert_that(&config.modules).is_empty();
}
