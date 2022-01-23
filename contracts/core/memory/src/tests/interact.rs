use cosmwasm_std::testing::{mock_env, mock_info};

use crate::contract::execute;
use crate::error::MemoryError;
use crate::tests::common::TEST_CREATOR;

use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use pandora::memory::msg::*;

/**
 * Test unallowed address update
 */
#[test]
fn unauthorized_memory_update() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // Try adding an asset to the memory
    let env = mock_env();
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), "asset_1".to_string())],
        to_remove: vec![],
    };

    let info = mock_info("some_address", &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Err(MemoryError::Admin(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }

    // Try adding a contract to the memory
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![("contract".to_string(), "contract_address".to_string())],
        to_remove: vec![],
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(MemoryError::Admin(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}

/**
 * Test allowed memory update
 */
#[test]
fn authorized_memory_update() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // Try adding an asset to the memory
    let env = mock_env();
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), "asset_1".to_string())],
        to_remove: vec![],
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }

    // Try adding a contract to the memory
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![("contract".to_string(), "contract_address".to_string())],
        to_remove: vec![],
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }

    // Try removing a contract from the memory
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![],
        to_remove: vec!["contract".to_string()],
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }
}
