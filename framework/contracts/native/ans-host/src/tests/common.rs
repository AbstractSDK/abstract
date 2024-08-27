use abstract_std::ans_host::ExecuteMsg;
use cosmwasm_std::{testing::*, Addr, DepsMut};

use crate::{contract, contract::AnsHostResult};

pub(crate) fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> AnsHostResult {
    contract::execute(
        deps,
        mock_env(),
        message_info(sender, &[]),
        msg,
    )
}
