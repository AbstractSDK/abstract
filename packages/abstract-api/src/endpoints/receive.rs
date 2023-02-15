use crate::{error::ApiError, state::ApiContract};
use abstract_sdk::{base::endpoints::ReceiveEndpoint, AbstractSdkError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
}
