use crate::{error::ExtensionError, state::ExtensionContract};

use abstract_sdk::base::endpoints::ReceiveEndpoint;

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
}
