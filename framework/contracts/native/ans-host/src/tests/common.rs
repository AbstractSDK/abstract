use abstract_std::ans_host::ExecuteMsg;
use abstract_unit_test_utils::mock_env_validated;
use cosmwasm_std::{testing::*, Addr, OwnedDeps, Querier, Storage};

use crate::{contract, contract::AnsHostResult};

pub(crate) fn execute_as(
    deps: &mut OwnedDeps<impl Storage, MockApi, impl Querier>,
    sender: &Addr,
    msg: ExecuteMsg,
) -> AnsHostResult {
    let env = mock_env_validated(deps.api);
    contract::execute(deps.as_mut(), env, message_info(sender, &[]), msg)
}
