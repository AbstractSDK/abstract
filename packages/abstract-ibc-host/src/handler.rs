use crate::{Host, HostError};
use abstract_sdk::base::Handler;
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Handler
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    type Error = Error;

    type CustomExecMsg = CustomExecMsg;

    type CustomInitMsg = CustomInitMsg;

    type CustomQueryMsg = CustomQueryMsg;

    type CustomMigrateMsg = CustomMigrateMsg;

    type ReceiveMsg = ReceiveMsg;

    fn contract(
        &self,
    ) -> &abstract_sdk::base::AbstractContract<
        Self,
        Self::Error,
        Self::CustomExecMsg,
        Self::CustomInitMsg,
        Self::CustomQueryMsg,
        Self::CustomMigrateMsg,
        Self::ReceiveMsg,
    > {
        &self.contract
    }
}
