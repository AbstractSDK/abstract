use abstract_sdk::features::DepsAccess;

use crate::{state::ContractError, AppContract, ReplyEndpoint};

impl<
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > ReplyEndpoint
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
