use abstract_sdk::features::DepsAccess;

use crate::{
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
