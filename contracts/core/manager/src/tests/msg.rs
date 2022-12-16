use cosmwasm_std::testing::{mock_env, mock_info};

use crate::dapp_base::common::TRADER_CONTRACT;
use abstract_sdk::os::modules::dapp_base::error::BaseDAppError;

use crate::contract::execute;
use crate::error::TerraswapError;
use crate::tests::base_mocks::mocks::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;
use abstract_sdk::os::dapps::terraswap::ExecuteMsg;

use abstract_sdk::os::modules::apis::terraswap::ExecuteMsg;

/**
 * ExecuteMsg::ProvideLiquidity
 */
#[test]
pub fn test_provide_liquidity_unauthorized_msg() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    mock_instantiate(deps.as_mut(), env.clone());
    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info("unauthorized", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(TerraswapError::BaseDAppError(BaseDAppError::Unauthorized {})) => (),
        Ok(_) => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
        _ => panic!("Should return unauthorized Error, DAppError::Unauthorized"),
    }
}

#[test]
pub fn test_unsuccessfully_provide_liquidity_nonexisting_asset_msg() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    mock_instantiate(deps.as_mut(), env.clone());

    let msg = ExecuteMsg::ProvideLiquidity {
        pool_id: "asset".to_string(),
        main_asset_id: "".to_string(),
        amount: Default::default(),
    };

    let info = mock_info(TRADER_CONTRACT, &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg);

    match res {
        Err(TerraswapError::Std(_)) => (),
        Ok(_) => panic!("Should return NotFound Err"),
        _ => panic!("Should return NotFound Err"),
    }
}
