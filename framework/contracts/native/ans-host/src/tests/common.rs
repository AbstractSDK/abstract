use abstract_std::ans_host::ExecuteMsg;
use cosmwasm_std::{
    testing::{mock_env, mock_info},
    DepsMut,
};

use crate::{contract, contract::AnsHostResult};

pub(crate) fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> AnsHostResult {
    contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
}
