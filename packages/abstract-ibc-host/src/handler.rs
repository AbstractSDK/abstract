use crate::{state::ContractError, Host};
use abstract_sdk::base::Handler;
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
        ReceiveMsg,
    > Handler
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
    type Error = Error;
    type CustomExecMsg = CustomExecMsg;
    type CustomInitMsg = CustomInitMsg;
    type CustomQueryMsg = CustomQueryMsg;
    type CustomMigrateMsg = CustomMigrateMsg;
    type SudoMsg = SudoMsg;
    type ReceiveMsg = ReceiveMsg;

    fn contract(&self) -> &abstract_sdk::base::AbstractContract<Self, Self::Error> {
        &self.contract
    }
}
