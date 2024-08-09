use crate::{state::ContractError, AppContract, ReplyEndpoint};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        UntaggedMsg,
        SudoMsg,
    > ReplyEndpoint
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
