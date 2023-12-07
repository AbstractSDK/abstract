use crate::{
    better_sdk::execution_stack::DepsAccess, state::ContractError, AppContract, IbcCallbackEndpoint,
};

impl<
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > IbcCallbackEndpoint
    for AppContract<
        '_,
        T,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
}
