use crate::{
    better_sdk::{contexts::AppInstantiateCtx, execution_stack::DepsAccess},
    state::ContractError,
    AbstractContract, AppContract, Handler,
};

impl<
        'a,
        T: DepsAccess,
        Error: ContractError,
        InitMsg,
        ExecMsg,
        QueryMsg,
        MigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > Handler
    for AppContract<'a, T, Error, InitMsg, ExecMsg, QueryMsg, MigrateMsg, ReceiveMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = MigrateMsg;
    type ReceiveMsg = ReceiveMsg;
    type SudoMsg = SudoMsg;

    type InstantiateCtx = AppInstantiateCtx<'a>;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
