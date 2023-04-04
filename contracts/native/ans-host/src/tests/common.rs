use crate::contract;
use crate::contract::AnsHostResult;
use abstract_core::ans_host::ExecuteMsg;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::DepsMut;

pub(crate) const TEST_CREATOR: &str = "creator";

pub(crate) fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> AnsHostResult {
    contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
}
