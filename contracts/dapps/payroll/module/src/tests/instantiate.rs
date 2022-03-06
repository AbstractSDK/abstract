use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, DepsMut};
use cosmwasm_std::{Api, Decimal};

use crate::dapp_base::common::MEMORY_CONTRACT;
use pandora_os::memory::item::Memory;
use pandora_os::treasury::dapp_base::state::{BaseState, BASESTATE};

use crate::contract::instantiate;
use crate::msg::InstantiateMsg;
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
pub fn mock_instantiate(deps: DepsMut) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, vault_instantiate_msg())
        .expect("contract successfully handles InstantiateMsg");
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    mock_instantiate(deps.as_mut());
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
