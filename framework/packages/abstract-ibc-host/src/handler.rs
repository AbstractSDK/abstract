use crate::{state::ContractError, Host};
use abstract_sdk::base::Handler;
impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > Handler
    for Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type Error = Error;
    type CustomInitMsg = CustomInitMsg;
    type CustomExecMsg = CustomExecMsg;
    type CustomQueryMsg = CustomQueryMsg;
    type CustomMigrateMsg = CustomMigrateMsg;
    type ReceiveMsg = ReceiveMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &abstract_sdk::base::AbstractContract<Self, Self::Error> {
        &self.contract
    }
}
