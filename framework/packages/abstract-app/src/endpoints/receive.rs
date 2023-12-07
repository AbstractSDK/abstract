use crate::{
    better_sdk::execution_stack::DepsAccess,
    state::{AppContract, ContractError},
    ReceiveEndpoint,
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
    > ReceiveEndpoint
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
