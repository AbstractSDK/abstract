use abstract_sdk::base::SudoEndpoint;

use crate::{state::ContractError, ApiContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
    SudoEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
{
}

#[cfg(test)]
mod tests {
    use crate::mock::{sudo, ApiMockResult};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use speculoos::prelude::*;

    #[test]
    fn endpoint() -> ApiMockResult {
        let env = mock_env();
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let sudo_msg = crate::mock::MockSudoMsg;
        let res = sudo(deps.as_mut(), env, sudo_msg)?;
        assert_that!(&res.messages.len()).is_equal_to(0);
        // confirm data is set
        assert_that!(res.data).is_equal_to(Some("mock_sudo".as_bytes().into()));

        Ok(())
    }
}
