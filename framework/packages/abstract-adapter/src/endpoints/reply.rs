use abstract_sdk::base::ReplyEndpoint;

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg> ReplyEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::AbstractSdkError;
    use abstract_unit_test_utils::mock_env_validated;
    use cosmwasm_std::{testing::mock_dependencies, Binary, Reply, SubMsgResponse};

    use crate::mock::{reply, AdapterMockResult};

    #[coverage_helper::test]
    fn endpoint() -> AdapterMockResult {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        deps.querier = abstract_unit_test_utils::abstract_mock_querier(deps.api);
        let reply_msg = Reply {
            id: 1,
            #[allow(deprecated)]
            result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some("test_reply".as_bytes().into()),
                msg_responses: vec![],
            }),
            payload: Binary::default(),
            gas_used: 0,
        };
        let res = reply(deps.as_mut(), env, reply_msg)?;
        assert_eq!(res.messages.len(), 0);
        // confirm data is set
        assert_eq!(res.data, Some("test_reply".as_bytes().into()));
        Ok(())
    }

    #[coverage_helper::test]
    fn no_matching_id() -> AdapterMockResult {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        deps.querier = abstract_unit_test_utils::abstract_mock_querier(deps.api);
        let reply_msg = Reply {
            id: 0,
            #[allow(deprecated)]
            result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some("test_reply".as_bytes().into()),
                msg_responses: vec![],
            }),
            payload: Binary::default(),
            gas_used: 0,
        };
        let res = reply(deps.as_mut(), env, reply_msg);
        assert_eq!(
            res,
            Err(AbstractSdkError::MissingHandler {
                endpoint: "reply with id 0".into(),
            }
            .into())
        );
        Ok(())
    }
}
