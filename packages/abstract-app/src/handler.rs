use crate::{AbstractContract, Handler};

use crate::{AppContract, AppError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        ExecMsg,
        InitMsg,
        QueryMsg,
        MigrateMsg,
        Receive,
    > Handler for AppContract<Error, ExecMsg, InitMsg, QueryMsg, MigrateMsg, Receive>
{
    fn contract(
        &self,
    ) -> &AbstractContract<Self, Error, ExecMsg, InitMsg, QueryMsg, MigrateMsg, Receive> {
        &self.contract
    }

    type Error = Error;

    type CustomExecMsg = ExecMsg;

    type CustomInitMsg = InitMsg;

    type CustomQueryMsg = QueryMsg;

    type CustomMigrateMsg = MigrateMsg;

    type ReceiveMsg = Receive;
}
