use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{Addr, Api};

use crate::dapp_base::common::{MEMORY_CONTRACT, TEST_CREATOR};
use abstract_os::modules::dapp_base::state::{BaseState, BASESTATE};
use abstract_os::native::memory::item::Memory;

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
            proxy_address: Addr::unchecked(""),
            traders: vec![],
            memory: Memory {
                address: deps.api.addr_validate(&MEMORY_CONTRACT).unwrap()
            }
        }
    );
}
