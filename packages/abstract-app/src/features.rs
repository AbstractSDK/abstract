use crate::{AppContract, AppError};
use abstract_sdk::{
    base::features::{AbstractNameService, Identification},
    feature_objects::AnsHost,
};
use cosmwasm_std::{Addr, Deps, StdResult};
impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > AbstractNameService
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn ans_host(&self, deps: Deps) -> StdResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Identification
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    fn proxy_address(&self, deps: Deps) -> StdResult<Addr> {
        Ok(self.base_state.load(deps.storage)?.proxy_address)
    }
}
