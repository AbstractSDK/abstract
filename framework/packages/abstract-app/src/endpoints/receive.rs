use crate::{
    state::{AppContract, ContractError},
    UntaggedEndpoint,
};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        UntaggedMsg,
        SudoMsg,
    > UntaggedEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        UntaggedMsg,
        SudoMsg,
    >
{
}
