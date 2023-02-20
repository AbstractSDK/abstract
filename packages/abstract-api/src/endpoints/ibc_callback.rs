use crate::{ApiContract, ApiError};
use abstract_sdk::{base::endpoints::IbcCallbackEndpoint, AbstractSdkError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > IbcCallbackEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
}
