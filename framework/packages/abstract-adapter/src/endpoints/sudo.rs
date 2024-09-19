use abstract_sdk::base::SudoEndpoint;

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg> SudoEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    use crate::mock::{sudo, AdapterMockResult};

    #[test]
    fn endpoint() -> AdapterMockResult {
        let env = mock_env();
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::abstract_mock_querier(deps.api);
        let sudo_msg = crate::mock::MockSudoMsg {};
        let res = sudo(deps.as_mut(), env, sudo_msg)?;
        assert_eq!(res.messages.len(), 0);
        // confirm data is set
        assert_eq!(res.data, Some("mock_sudo".as_bytes().into()));

        Ok(())
    }
}
