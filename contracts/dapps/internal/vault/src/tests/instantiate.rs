use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, DepsMut, Env};
use cosmwasm_std::{Api, Decimal};
use pandora_os::core::treasury::dapp_base::msg::BaseExecuteMsg;

use crate::dapp_base::common::MEMORY_CONTRACT;
use pandora_os::core::treasury::dapp_base::state::{BaseState, BASESTATE};
use pandora_os::native::memory::item::Memory;

use crate::contract::{execute, instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{State, STATE};
use crate::tests::base_mocks::mocks::instantiate_msg as base_init_msg;
use crate::tests::common::{TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};

pub(crate) fn vault_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        base: base_init_msg(),
        token_code_id: 3u64,
        fee: Decimal::zero(),
        deposit_asset: TREASURY_CONTRACT.to_string(),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    }
}

/**
 * Mocks instantiation of the contract.
 */
pub fn mock_instantiate(mut deps: DepsMut, env: Env) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(
        deps.branch(),
        env.clone(),
        info.clone(),
        vault_instantiate_msg(),
    )
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

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env();
    mock_instantiate(deps.as_mut(), env);
    assert_eq!(
        BASESTATE.load(&deps.storage).unwrap(),
        BaseState {
            treasury_address: deps.api.addr_validate(&TREASURY_CONTRACT).unwrap(),
            traders: vec![deps.api.addr_validate(&TRADER_CONTRACT).unwrap()],
            memory: Memory {
                address: deps.api.addr_validate(&MEMORY_CONTRACT).unwrap()
            }
        }
    );
    assert_eq!(
        STATE.load(&deps.storage).unwrap(),
        State {
            liquidity_token_addr: Addr::unchecked("")
        }
    );
}
