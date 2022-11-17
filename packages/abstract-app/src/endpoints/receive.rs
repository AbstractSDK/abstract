use crate::{error::AppError, state::AppContract};

use crate::ReceiveEndpoint;

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
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
