use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
use cosmwasm_std::{Addr, Env, MemoryStorage, OwnedDeps};

use pandora::memory::item::Memory;
use pandora::treasury::dapp_base::error::BaseDAppError;
use pandora::treasury::dapp_base::msg::BaseExecuteMsg;
use pandora::treasury::dapp_base::state::{BaseState, ADMIN, BASESTATE};

use crate::contract::execute;
use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use crate::msg::ExecuteMsg;
use crate::tests::base_mocks::mocks::mock_instantiate;
use rstest::*;

type MockDeps = OwnedDeps<MemoryStorage, MockApi, MockQuerier>;

#[fixture]
fn deps() -> (MockDeps, Env) {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    mock_instantiate(deps.as_mut(), env.clone());
    (deps, env)
}

/**
 * BaseExecuteMsg::UpdateConfig
 */
#[rstest]
pub fn test_unsuccessfully_update_config_msg(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
    });

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::Admin(_)) => (),
        Ok(_) => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
        _ => panic!("Should return unauthorized Error, Admin(NotAdmin)"),
    }
}

#[rstest]
pub fn test_successfully_update_config_msg_with_treasury_address(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_equal_base_state(
        &state,
        "new_treasury_address",
        vec![TRADER_CONTRACT],
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_successfully_update_config_msg_with_all_parameters(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("new_treasury_address".to_string()),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_equal_base_state(
        &state,
        "new_treasury_address",
        vec![TRADER_CONTRACT],
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_successfully_update_config_msg_with_no_parameters(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec![TRADER_CONTRACT],
        MEMORY_CONTRACT,
    )
}

/**
 * BaseExecuteMsg::SetAdmin
 */
#[rstest]
pub fn test_unsuccessfully_set_admin_msg(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
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

#[rstest]
pub fn test_successfully_set_admin_msg(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;

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

#[rstest]
pub fn test_successfully_update_traders_add(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec![
            "new_trader_address1".to_string(),
            "new_trader_address2".to_string(),
        ]),
        to_remove: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec![
            TRADER_CONTRACT,
            "new_trader_address1",
            "new_trader_address2",
        ],
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_successfully_update_traders_add_many(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let mut new_traders: Vec<String> = vec![];
    for i in 1..=100 {
        new_traders.push(format!("new_trader_{}", i));
    }
    // let aoeu = new_traders.clone();

    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(new_traders.clone()),
        to_remove: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    let mut expected_traders = vec![TRADER_CONTRACT.to_string()];
    expected_traders.extend(new_traders);

    assert_eq!(
        &state,
        &BaseState {
            treasury_address: Addr::unchecked(TREASURY_CONTRACT.to_string()),
            traders: expected_traders
                .into_iter()
                .map(|t| Addr::unchecked(t))
                .collect(),
            memory: Memory {
                address: Addr::unchecked(MEMORY_CONTRACT.to_string())
            },
        }
    )
}

#[rstest]
pub fn test_unsuccessfully_update_traders_add_already_present(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec![TRADER_CONTRACT.to_string()]),
        to_remove: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::TraderAlreadyPresent { trader: _ }) => (),
        _ => panic!("Should return trader already present Error, TraderAlreadyPresent"),
    }

    // verify state is same
    let state = BASESTATE.load(deps.as_mut().storage).unwrap();
    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec![TRADER_CONTRACT], // should be same
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_successfully_update_traders_remove(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    // lets add some traders to start
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec![
            "new_trader_address1".to_string(),
            "new_trader_address2".to_string(),
        ]),
        to_remove: None,
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();

    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec![
            TRADER_CONTRACT,
            "new_trader_address1",
            "new_trader_address2",
        ],
        MEMORY_CONTRACT,
    );

    // now try and remove the traders
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: None,
        to_remove: Some(vec![
            "new_trader_address1".to_string(),
            TRADER_CONTRACT.to_string(),
        ]),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();
    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec!["new_trader_address2"], // only 2 is left
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_unsuccessfully_update_traders_remove_not_present(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    // now try and remove some traders that were not there
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: None,
        to_remove: Some(vec!["nonexistent".to_string(), "nonexistent2".to_string()]),
    });

    // no error
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::TraderNotPresent { trader: _ }) => (),
        _ => panic!("Should return trader not present Error, TraderNotPresent"),
    }

    // assert the same
    let state = BASESTATE.load(deps.as_mut().storage).unwrap();
    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec![TRADER_CONTRACT],
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_successfully_update_traders_replace_existing(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    // now try and remove some traders that were not there
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec!["new_trader".to_string()]),
        to_remove: Some(vec![TRADER_CONTRACT.to_string()]),
    });

    // no error
    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    let state = BASESTATE.load(deps.as_mut().storage).unwrap();
    assert_equal_base_state(
        &state,
        TREASURY_CONTRACT,
        vec!["new_trader"],
        MEMORY_CONTRACT,
    );
}

#[rstest]
pub fn test_unsuccessfully_update_traders_no_traders_left(deps: (MockDeps, Env)) {
    let (mut deps, env) = deps;
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: None,
        to_remove: Some(vec![TRADER_CONTRACT.to_string()]),
    });

    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(BaseDAppError::TraderRequired {}) => (),
        _ => panic!("Should return trader required Error, TraderRequired"),
    }
}

/// Helper function to assert that the provided state is equal to the provided state values
fn assert_equal_base_state(
    actual_state: &BaseState,
    expected_treasury: &str,
    expected_traders: Vec<&str>,
    expected_memory_addr: &str,
) {
    // we could use unwrap_or with the default values but would be less clear because we'd provide None to the method
    assert_eq!(
        actual_state,
        &BaseState {
            treasury_address: Addr::unchecked(expected_treasury.to_string()),
            traders: expected_traders
                .into_iter()
                .map(|t| Addr::unchecked(t))
                .collect(),
            memory: Memory {
                address: Addr::unchecked(expected_memory_addr.to_string())
            },
        }
    )
}
