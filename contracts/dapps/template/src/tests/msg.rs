use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;

use dao_os::memory::item::Memory;
use dao_os::treasury::dapp_base::error::BaseDAppError;
use dao_os::treasury::dapp_base::msg::BaseExecuteMsg;
use dao_os::treasury::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::contract::execute;
use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::msg::ExecuteMsg;
use crate::tests::base_mocks::mocks::mock_instantiate;

/**
 * BaseExecuteMsg::UpdateConfig
 */
#[test]
pub fn test_unsuccessfully_update_config_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: None,
        memory: None,
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::Admin(_)) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
    }
}

#[test]
pub fn test_successfully_update_config_msg_with_treasury_address() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
        trader: None,
        memory: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked("new_treasury_address".to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
            memory: Memory {
                address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
            },
        }
    )
}

#[test]
pub fn test_successfully_update_config_msg_with_trader() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: Some("new_trader_address".to_string()),
        memory: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            trader: Addr::unchecked("new_trader_address".to_string()),
            memory: Memory {
                address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
            },
        }
    )
}

#[test]
pub fn test_successfully_update_config_msg_with_memory() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: None,
        memory: Some("new_memory_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
            memory: Memory {
                address: Addr::unchecked("new_memory_address".to_string())
            },
        }
    )
}

#[test]
pub fn test_successfully_update_config_msg_with_all_parameters() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
        trader: Some("new_trader_address".to_string()),
        memory: Some("new_memory_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked("new_treasury_address".to_string()),
            trader: Addr::unchecked("new_trader_address".to_string()),
            memory: Memory {
                address: Addr::unchecked("new_memory_address".to_string())
            },
        }
    )
}

#[test]
pub fn test_successfully_update_config_msg_with_no_parameters() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
        trader: None,
        memory: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_eq!(
        state,
        BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            trader: Addr::unchecked(TRADER_CONTRACT.to_string()),
            memory: Memory {
                address: Addr::unchecked(&MEMORY_CONTRACT.to_string())
            },
        }
    )
}

/**
 * BaseExecuteMsg::SetAdmin
 */
#[test]
pub fn test_unsuccessfully_set_admin_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();
    let msg = ExecuteMsg::Base(BaseExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::Admin(_)) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
    }
}

#[test]
pub fn test_successfully_set_admin_msg() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    // check original admin
    let admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(admin, Addr::unchecked(TEST_CREATOR.to_string()));

    // set new admin
    let msg = ExecuteMsg::Base(BaseExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    });
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // check new admin
    let admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(admin, Addr::unchecked("new_admin".to_string()));
}
