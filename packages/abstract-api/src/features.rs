use crate::{ApiContract, ApiError};
use abstract_sdk::{
    feature_objects::AnsHost,
    features::{AbstractNameService, AbstractRegistryAccess, Identification},
    AbstractSdkError, AbstractSdkResult,
};
use cosmwasm_std::{Addr, Deps, StdError};

impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractNameService
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn ans_host(&self, deps: Deps) -> AbstractSdkResult<AnsHost> {
        Ok(self.base_state.load(deps.storage)?.ans_host)
    }
}

/// Retrieve identifying information about the calling OS
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > Identification
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.proxy.clone())
        } else {
            Err(StdError::generic_err("No target OS specified to execute on.").into())
        }
    }

    fn manager_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
        if let Some(target) = &self.target_os {
            Ok(target.manager.clone())
        } else {
            Err(StdError::generic_err("No OS manager specified.").into())
        }
    }

    fn os_core(&self, _deps: Deps) -> AbstractSdkResult<abstract_sdk::os::version_control::Core> {
        if let Some(target) = &self.target_os {
            Ok(target.clone())
        } else {
            Err(StdError::generic_err("No OS core specified.").into())
        }
    }
}

/// Get the version control contract
impl<
        Error: From<cosmwasm_std::StdError> + From<ApiError> + From<AbstractSdkError>,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > AbstractRegistryAccess
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg>
{
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<Addr> {
        Ok(self.state(deps.storage)?.version_control)
    }
}
