use abstract_std::ans_host::*;
use abstract_testing::{mock_env_validated, prelude::AbstractMockAddrs};
use cosmwasm_std::testing::*;
use cw_asset::AssetInfo;

use crate::{
    contract::execute,
    error::AnsHostError,
    tests::{instantiate::mock_init, mock_querier::mock_dependencies},
};

/**
 * Test disallowed address update
 */
#[coverage_helper::test]
fn unauthorized_ans_host_update() {
    let mut deps = mock_dependencies(&[]);
    mock_init(&mut deps);

    // Try adding an asset to the ans_host
    let env = mock_env_validated(deps.api);
    let asset_info = AssetInfo::Native("asset_1".to_string());
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), asset_info.into())],
        to_remove: vec![],
    };

    let info = message_info(&deps.api.addr_make("some_address"), &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Err(AnsHostError::Ownership(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }

    // Try adding a contract to the ans_host
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![(
            "project:contract".try_into().unwrap(),
            deps.api.addr_make("contract_address").to_string(),
        )],
        to_remove: vec![],
    };

    let res = execute(deps.as_mut(), env, info, msg);

    match res {
        Err(AnsHostError::Ownership(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}

/**
 * Test allowed ans_host update
 */
#[coverage_helper::test]
fn authorized_ans_host_update() {
    let mut deps = mock_dependencies(&[]);
    mock_init(&mut deps);
    let abstr = AbstractMockAddrs::new(deps.api);

    // Try adding an asset to the ans_host
    let env = mock_env_validated(deps.api);
    let asset_info = AssetInfo::Native("asset_1".to_string());
    let msg = ExecuteMsg::UpdateAssetAddresses {
        to_add: vec![("asset".to_string(), asset_info.into())],
        to_remove: vec![],
    };

    let info = message_info(&abstr.owner, &[]);
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }

    // Try adding a contract to the ans_host
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![(
            "project:contract".try_into().unwrap(),
            deps.api.addr_make("contract_address").to_string(),
        )],
        to_remove: vec![],
    };

    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }

    // Try removing a contract from the ans_host
    let msg = ExecuteMsg::UpdateContractAddresses {
        to_add: vec![],
        to_remove: vec!["project:contract".try_into().unwrap()],
    };

    let res = execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => (),
        _ => panic!("Should not return Err"),
    }
}
