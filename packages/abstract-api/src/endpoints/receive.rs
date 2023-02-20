use crate::{error::ApiError, state::ApiContract};
use abstract_sdk::{base::endpoints::ReceiveEndpoint, AbstractSdkError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
}
