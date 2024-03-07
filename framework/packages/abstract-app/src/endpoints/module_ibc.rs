use abstract_core::{objects::module::ModuleInfo, AbstractError, IBC_HOST};
use abstract_sdk::{base::ModuleIbcEndpoint, features::AbstractRegistryAccess, AbstractSdkError};
use cosmwasm_std::Addr;

use crate::{state::ContractError, AppContract};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > ModuleIbcEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    // This can't be used because accounts don't install the ibc_host on their accounts so this is never available
    // fn ibc_host(&self, deps: cosmwasm_std::Deps) -> Result<cw_orch::prelude::Addr, AbstractSdkError> {
    //     let ibc_client = self.modules(deps).module_address(IBC_HOST)?;
    //     Ok(ibc_client)
    // }
    fn ibc_host(&self, deps: cosmwasm_std::Deps) -> Result<Addr, AbstractSdkError> {
        let vc_query_result = self
            .abstract_registry(deps)?
            .query_module(ModuleInfo::from_id_latest(IBC_HOST)?, &deps.querier)
            .map_err(|err| {
                let err: AbstractError = err.into();
                err
            })?;

        Ok(vc_query_result.reference.unwrap_native()?)
    }
}
