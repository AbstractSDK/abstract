use crate::{Host, HostError};
use abstract_sdk::base::Handler;
impl<
        Error: From<cosmwasm_std::StdError> + From<HostError> + From<abstract_sdk::AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Handler
    for Host<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
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
        Self::CustomInitMsg,
        Self::CustomExecMsg,
        Self::CustomQueryMsg,
        Self::CustomMigrateMsg,
        Self::ReceiveMsg,
    > {
        &self.contract
    }
}
