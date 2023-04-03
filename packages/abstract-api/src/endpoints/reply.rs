use abstract_sdk::base::ReplyEndpoint;

use crate::{ApiContract, ApiError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<abstract_sdk::AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ReplyEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
}
