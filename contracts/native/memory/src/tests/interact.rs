use std::convert::TryInto;

use cosmwasm_std::testing::{mock_env, mock_info};
use cw_asset::AssetInfo;

use crate::contract::execute;
use crate::error::MemoryError;
use crate::tests::common::TEST_CREATOR;

use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use abstract_os::memory::*;

/**
 * Test unallowed address update
 */
#[test]
fn unauthorized_memory_update() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // Try adding an asset to the memory
    let env = mock_env();
    let asset_info = AssetInfo::Native("asset_1".to_string());
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), asset_info.into())],
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
        to_add: vec![(
            "project/contract".to_string().try_into().unwrap(),
            "contract_address".to_string(),
        )],
        to_remove: vec![],
    };

    let res = execute(deps.as_mut(), env, info, msg);

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
    let asset_info = AssetInfo::Native("asset_1".to_string());
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), asset_info.into())],
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
        to_add: vec![(
            "project/contract".to_string().try_into().unwrap(),
            "contract_address".to_string(),
        )],
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
        to_remove: vec!["project/contract".to_string().try_into().unwrap()],
    };

    let res = execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }
}
