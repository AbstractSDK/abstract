use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::DepsMut;

use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use pandora::treasury::dapp_base::msg::BaseInstantiateMsg;

use crate::contract::instantiate;

pub(crate) fn instantiate_msg() -> BaseInstantiateMsg {
    BaseInstantiateMsg {
        memory_addr: MEMORY_CONTRACT.to_string(),
        treasury_address: TREASURY_CONTRACT.to_string(),
        trader: TRADER_CONTRACT.to_string(),
    }
}

/**
 * Mocks instantiation of the contract.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, instantiate_msg())
        .expect("contract successfully handles InstantiateMsg");
}
