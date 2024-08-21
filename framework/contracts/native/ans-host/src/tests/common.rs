use abstract_std::ans_host::ExecuteMsg;
use cosmwasm_std::{testing::*, DepsMut};

use crate::{contract, contract::AnsHostResult};

pub(crate) fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> AnsHostResult {
    contract::execute(
        deps,
        mock_env(),
        message_info(&MockApi::default().addr_make(sender), &[]),
        msg,
    )
}
