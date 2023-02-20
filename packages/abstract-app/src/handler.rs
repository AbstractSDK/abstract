use crate::{AbstractContract, AppContract, AppError, Handler};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        InitMsg,
        ExecMsg,
        QueryMsg,
        MigrateMsg,
        Receive,
    > Handler for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, Receive>
{
    type Error = Error;
    type CustomExecMsg = ExecMsg;
    type CustomInitMsg = InitMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type ReceiveMsg = Receive;

    fn contract(
        &self,
    ) -> &AbstractContract<Self, Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, Receive> {
        &self.contract
    }
}
