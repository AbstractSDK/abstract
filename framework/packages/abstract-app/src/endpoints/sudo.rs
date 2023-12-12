use abstract_sdk::{base::SudoEndpoint, features::DepsAccess};

use crate::{state::ContractError, AppContract};

impl<
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > SudoEndpoint
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
