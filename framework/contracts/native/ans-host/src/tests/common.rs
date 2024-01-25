use abstract_core::ans_host::ExecuteMsg;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::DepsMut;

use crate::contract;
use crate::contract::AnsHostResult;

pub(crate) fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> AnsHostResult {
    contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
}
