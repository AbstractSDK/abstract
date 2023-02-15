use crate::{error::AppError, state::AppContract, ReceiveEndpoint};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError> + From<abstract_sdk::AbstractSdkError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ReceiveEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
}
