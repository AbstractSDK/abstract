use abstract_sdk::base::ReceiveEndpoint;

use crate::{state::ContractError, Host};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    >
{
}
