use abstract_sdk::base::ReplyEndpoint;

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    ReplyEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
}

#[cfg(test)]
mod tests {
    use crate::mock::{reply, AdapterMockResult};
    use abstract_sdk::AbstractSdkError;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
        Reply, SubMsgResponse,
    };
    use speculoos::prelude::*;

    #[test]
    fn endpoint() -> AdapterMockResult {
        let env = mock_env();
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let reply_msg = Reply {
            id: 1,
            result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some("test_reply".as_bytes().into()),
            }),
        };
        let res = reply(deps.as_mut(), env, reply_msg)?;
        assert_that!(&res.messages.len()).is_equal_to(0);
        // confirm data is set
        assert_that!(res.data).is_equal_to(Some("test_reply".as_bytes().into()));
        Ok(())
    }

    #[test]
    fn no_matching_id() -> AdapterMockResult {
        let env = mock_env();
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let reply_msg = Reply {
            id: 0,
            result: cosmwasm_std::SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: Some("test_reply".as_bytes().into()),
            }),
        };
        let res = reply(deps.as_mut(), env, reply_msg);
        assert_that!(res).is_err().is_equal_to(
            &AbstractSdkError::MissingHandler {
                endpoint: "reply with id 0".into(),
            }
            .into(),
        );
        Ok(())
    }
}
