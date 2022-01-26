use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Api, Decimal, MessageInfo};

use crate::contract::execute;
use crate::error::VaultError;
use crate::msg::ExecuteMsg;
use crate::tests::common::TEST_CREATOR;
use crate::tests::instantiate::mock_instantiate;

#[test]
fn unsuccessful_set_fee_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::SetFee {
        fee: pandora::fee::Fee {
            share: Decimal::percent(10u64),
        },
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(VaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

/**
 * Tests updating the fees of the contract.
 */
#[test]
fn successful_update_fee() {
    // update fees
    let info = mock_info(TEST_CREATOR, &[]);
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = ExecuteMsg::SetFee {
        fee: pandora::fee::Fee {
            share: Decimal::percent(10u64),
        },
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

/**
 * Tests updating the pool of the contract.
 */
#[test]
fn successful_update_pool() {
    // update fees
    let info = mock_info(TEST_CREATOR, &[]);
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdatePool {
        deposit_asset: Some("whale".to_string()),
        assets_to_add: vec!["whale".to_string()],
        assets_to_remove: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

/**
 * Tests unsuccessfull updating the pool of the contract.
 */
#[test]
fn unsuccessful_update_pool() {
    let info = mock_info("someone", &[]);
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdatePool {
        deposit_asset: Some("whale".to_string()),
        assets_to_add: vec!["whale".to_string()],
        assets_to_remove: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(VaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}
