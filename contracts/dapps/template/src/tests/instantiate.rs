use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Api;

use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR, TRADER_CONTRACT, TREASURY_CONTRACT};
use dao_os::memory::item::Memory;
use dao_os::treasury::dapp_base::state::{BaseState, BASESTATE};

use crate::contract::instantiate;
use crate::tests::base_mocks::mocks::instantiate_msg;

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

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
}
