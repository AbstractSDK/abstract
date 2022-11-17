use abstract_sdk::base::endpoints::IbcCallbackEndpoint;

use crate::{ExtensionContract, ExtensionError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > IbcCallbackEndpoint
    for ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
}
