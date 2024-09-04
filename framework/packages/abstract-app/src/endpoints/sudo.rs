use abstract_sdk::base::SudoEndpoint;

use crate::{state::ContractError, AppContract};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > SudoEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
}
