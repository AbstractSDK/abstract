use abstract_sdk::base::UntaggedEndpoint;

use crate::state::{AdapterContract, ContractError};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, UntaggedMsg, SudoMsg>
    UntaggedEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, UntaggedMsg, SudoMsg>
{
}

#[cfg(test)]
mod tests {
    use abstract_std::adapter::ExecuteMsg;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use speculoos::prelude::*;

    use crate::mock::{execute, AdapterMockResult, MockUntaggedMsg};

    #[test]
    fn endpoint() -> AdapterMockResult {
        let env = mock_env();
        let info = mock_info("sender", &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let msg = MockUntaggedMsg {};
        let res = execute(deps.as_mut(), env, info, ExecuteMsg::Untagged(msg))?;
        assert_that!(&res.messages.len()).is_equal_to(0);
        // confirm data is set
        assert_that!(res.data).is_equal_to(Some("mock_receive".as_bytes().into()));
        Ok(())
    }
}
