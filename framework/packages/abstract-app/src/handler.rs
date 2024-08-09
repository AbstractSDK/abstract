use crate::{state::ContractError, AbstractContract, AppContract, Handler};

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, MigrateMsg, UntaggedMsg, SudoMsg> Handler
    for AppContract<Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, UntaggedMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type UntaggedMsg = UntaggedMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
