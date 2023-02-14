use crate::{ApiContract, ApiError};
use abstract_sdk::base::{AbstractContract, Handler};
use abstract_sdk::AbstractSdkError;
use cosmwasm_std::Empty;

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        ExecMsg,
        InitMsg,
        QueryMsg,
        Receive,
    > Handler for ApiContract<Error, ExecMsg, InitMsg, QueryMsg, Receive>
{
    type Error = Error;

    type CustomExecMsg = ExecMsg;

    type CustomInitMsg = InitMsg;

    type CustomQueryMsg = QueryMsg;

    type CustomMigrateMsg = Empty;

    type ReceiveMsg = Receive;

    fn contract(
        &self,
    ) -> &AbstractContract<Self, Error, ExecMsg, InitMsg, QueryMsg, Empty, Receive> {
        &self.contract
    }
}
