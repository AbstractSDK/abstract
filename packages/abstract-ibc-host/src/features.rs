use crate::{Host, HostError};
use abstract_sdk::base::features::{AbstractNameService, Identification, ModuleIdentification};
use cosmwasm_std::{Deps, StdError, StdResult};

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > AbstractNameService
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn ans_host(&self, deps: Deps) -> StdResult<abstract_sdk::feature_objects::AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > Identification
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn proxy_address(&self, _deps: Deps) -> StdResult<cosmwasm_std::Addr> {
        self.target()
            .map_err(|e| StdError::generic_err(e.to_string()))
            .map(ToOwned::to_owned)
    }
    fn manager_address(&self, _deps: Deps) -> StdResult<cosmwasm_std::Addr> {
        Err(StdError::generic_err(
            "manager address not available on stateless ibc deployment",
        ))
    }

    fn os_core(&self, _deps: Deps) -> StdResult<abstract_sdk::os::version_control::Core> {
        Err(StdError::generic_err(
            "OS core not available on stateless ibc deployment",
        ))
    }

    fn os_id(&self, _deps: Deps) -> StdResult<u32> {
        Err(StdError::generic_err(
            "os_id not available on stateless ibc deployment",
        ))
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    > ModuleIdentification
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
{
    fn module_id(&self) -> &'static str {
        self.contract.info().0
    }
}
