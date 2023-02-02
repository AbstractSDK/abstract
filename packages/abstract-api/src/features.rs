use crate::{ApiContract, ApiError};
use abstract_sdk::base::features::ModuleIdentification;
use abstract_sdk::{
    base::features::{AbstractNameService, AbstractRegistryAccess, Identification},
    feature_objects::AnsHost,
};
use cosmwasm_std::{Addr, Deps, StdError, StdResult};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractNameService
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn ans_host(&self, deps: Deps) -> StdResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling OS
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > Identification
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn proxy_address(&self, _deps: Deps) -> StdResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.proxy.clone())
        } else {
            Err(StdError::generic_err(
                "No target OS specified to execute on.",
            ))
        }
    }

    fn manager_address(&self, _deps: Deps) -> StdResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.manager.clone())
        } else {
            Err(StdError::generic_err("No OS manager specified."))
        }
    }

    fn os_core(&self, _deps: Deps) -> StdResult<abstract_sdk::os::version_control::Core> {
        if let Some(target) = &self.target_os {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No OS core specified."))
        }
    }
}

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ModuleIdentification
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn module_id(&self) -> &'static str {
        self.contract.info().0
    }
}

/// Get the version control contract
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractRegistryAccess
    for ApiContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn abstract_registry(&self, deps: Deps) -> StdResult<Addr> {
        Ok(self.state(deps.storage)?.version_control)
    }
}
