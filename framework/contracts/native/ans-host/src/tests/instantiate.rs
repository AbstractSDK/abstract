use abstract_std::ans_host::*;
use abstract_testing::{mock_env_validated, prelude::AbstractMockAddrs};
use cosmwasm_std::{testing::*, MessageInfo, OwnedDeps};
use speculoos::prelude::*;

use crate::{
    contract::instantiate,
    tests::{common::execute_as, mock_querier::mock_dependencies},
};

pub(crate) fn instantiate_msg(info: &MessageInfo) -> InstantiateMsg {
    InstantiateMsg {
        admin: info.sender.to_string(),
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_init<Q: cosmwasm_std::Querier>(deps: &mut OwnedDeps<MockStorage, MockApi, Q>) {
    let abstr = AbstractMockAddrs::new(deps.api);
    let info = message_info(&abstr.owner, &[]);
    let env = mock_env_validated(deps.api);
    let msg = InstantiateMsg {
        admin: info.sender.to_string(),
    };

    let _res = instantiate(deps.as_mut(), env, info, msg)
        .expect("contract successfully handles InstantiateMsg");
}

/**
 * Tests successful instantiation of the contract.
 */
#[coverage_helper::test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);
    let env = mock_env_validated(deps.api);
    let abstr = AbstractMockAddrs::new(deps.api);

    let info = message_info(&abstr.owner, &[]);
    let msg = instantiate_msg(&info);
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[coverage_helper::test]
fn successful_update_ownership() {
    let mut deps = mock_dependencies(&[]);
    mock_init(&mut deps);
    let abstr = AbstractMockAddrs::new(deps.api);

    let new_admin = deps.api.addr_make("new_admin");
    // First update to transfer
    let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
        new_owner: new_admin.to_string(),
        expiry: None,
    });

    let transfer_res = execute_as(&mut deps, &abstr.owner, transfer_msg).unwrap();
    assert_eq!(0, transfer_res.messages.len());

    // Then update and accept as the new owner
    let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
    let accept_res = execute_as(&mut deps, &new_admin, accept_msg).unwrap();
    assert_eq!(0, accept_res.messages.len());

    assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
        .is_some()
        .is_equal_to(new_admin)
}
