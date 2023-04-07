use crate::{state::ContractError, ApiContract};
use abstract_sdk::base::IbcCallbackEndpoint;

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
    IbcCallbackEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg, ReceiveMsg>
{
}

// to test this, add ibc client to version control

// #[cfg(test)]
// mod tests {
//     use abstract_core::{api::{ExecuteMsg, BaseExecuteMsg, self}, abstract_ica::IbcResponseMsg};
//     use abstract_sdk::base::ExecuteEndpoint;
//     use abstract_testing::prelude::{TEST_MANAGER, mocked_account_querier_builder};
//     use cosmwasm_std::{testing::{mock_dependencies, mock_env, mock_info}, DepsMut, Response};
//     use speculoos::prelude::*;
//     use crate::mock::{ApiMockResult, MockReceiveMsg, execute, MOCK_API, mock_init, MockExecMsg, MockError};

//     fn setup_with_traders(mut deps: DepsMut, traders: Vec<&str>) {
//         mock_init(deps.branch()).unwrap();

//         let _api = MOCK_API;
//         let msg = BaseExecuteMsg::UpdateTraders {
//             to_add: traders.into_iter().map(Into::into).collect(),
//             to_remove: vec![],
//         };

//         base_execute_as(deps, TEST_MANAGER, msg).unwrap();
//     }

//     fn execute_as(
//         deps: DepsMut,
//         sender: &str,
//         msg: ExecuteMsg<MockExecMsg, MockReceiveMsg>,
//     ) -> Result<Response, MockError> {
//         MOCK_API.execute(deps, mock_env(), mock_info(sender, &[]), msg)
//     }

//     fn base_execute_as(
//         deps: DepsMut,
//         sender: &str,
//         msg: BaseExecuteMsg,
//     ) -> Result<Response, MockError> {
//         execute_as(deps, sender, api::ExecuteMsg::Base(msg))
//     }

//     #[test]
//     fn endpoint() -> ApiMockResult {
//         let env = mock_env();
//         let info = mock_info("trader", &[]);
//         let mut deps = mock_dependencies();
//         deps.querier = mocked_account_querier_builder().build();

//         setup_with_traders(deps.as_mut(), vec!["trader"]);
//         let msg = IbcResponseMsg{id: "id".to_string(), msg: abstract_core::abstract_ica::StdAck::Result("all_gud".as_bytes().into())};
//         let res = execute(deps.as_mut(), env,info, ExecuteMsg::IbcCallback(msg))?;
//         assert_that!(&res.messages.len()).is_equal_to(0);
//         // confirm data is set
//         assert_that!(res.data).is_equal_to(Some("mock_ibc_receive".as_bytes().into()));
//         Ok(())
//     }
// }
