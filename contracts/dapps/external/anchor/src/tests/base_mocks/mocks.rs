use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{DepsMut, Env};

use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT};
use crate::msg::ExecuteMsg;
use pandora_os::core::treasury::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg};

use crate::contract::{execute, instantiate};

pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        memory_addr: MEMORY_CONTRACT.to_string(),
    }
}

/**
 * Mocks instantiation of the contract.
 */
pub fn mock_instantiate(mut deps: DepsMut, env: Env) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.branch(), env.clone(), info.clone(), instantiate_msg())
        .expect("contract successfully handles InstantiateMsg");

    // Add one trader
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateTraders {
        to_add: Some(vec![TRADER_CONTRACT.to_string()]),
        to_remove: None,
    });

    execute(deps.branch(), env.clone(), info.clone(), msg).unwrap();

    // Set treasury addr
    let msg = ExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
        treasury_address: Some("treasury_contract_address".to_string()),
    });

    execute(deps, env.clone(), info, msg).unwrap();
}
