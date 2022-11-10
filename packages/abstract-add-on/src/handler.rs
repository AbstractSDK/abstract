use abstract_sdk::{AbstractContract, Handler};

use crate::{AddOnContract, AddOnError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AddOnError>,
        ExecMsg,
        InitMsg,
        QueryMsg,
        MigrateMsg,
        Receive,
    > Handler for AddOnContract<Error, ExecMsg, InitMsg, QueryMsg, MigrateMsg, Receive>
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
