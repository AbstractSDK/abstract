use crate::state::{AdapterContract, ContractError};
use abstract_sdk::{base::ReceiveEndpoint, features::DepsAccess};
use cosmwasm_std::{DepsMut, Env, MessageInfo};

impl<
        'a,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    > ReceiveEndpoint
    for AdapterContract<
        'a,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
}

#[cfg(test)]
mod tests {
    use crate::mock::{execute, AdapterMockResult, MockReceiveMsg};
    use abstract_core::adapter::ExecuteMsg;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use speculoos::prelude::*;

    #[test]
    fn endpoint() -> AdapterMockResult {
        let env = mock_env();
        let info = mock_info("sender", &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let msg = MockReceiveMsg;
        let res = execute(deps.as_mut(), env, info, ExecuteMsg::Receive(msg))?;
        assert_that!(&res.messages.len()).is_equal_to(0);
        // confirm data is set
        assert_that!(res.data).is_equal_to(Some("mock_receive".as_bytes().into()));
        Ok(())
    }
}
