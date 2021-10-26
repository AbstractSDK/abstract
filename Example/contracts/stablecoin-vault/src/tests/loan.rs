use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};
use cosmwasm_std::{Api, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::instantiate::mock_instantiate;
use crate::state::{State, STATE};
use cw20::MinterResponse;

use terraswap::asset::AssetInfo;
use terraswap::token::InstantiateMsg as TokenInstantiateMsg;
use white_whale::fee::*;
use white_whale::ust_vault::msg::VaultQueryMsg as QueryMsg;
use white_whale::ust_vault::msg::*;

use crate::tests::common::{ARB_CONTRACT, TEST_CREATOR};

use crate::tests::mock_querier::mock_dependencies;

const INSTANTIATE_REPLY_ID: u8 = 1u8;


/**
 * Mocks call for loan. Should update balance of caller if successfull. 
 */
pub fn mock_flash_loan(deps: DepsMut) {
    
    let info = mock_info(TEST_CREATOR, &[]);
    let _res =
        instantiate(deps, mock_env(), info, msg).expect("contract successfully handles InitMsg");
    // Set mock value on profit check
    // Update balances of caller and vault
    
}

