use crate::{state::ContractError, AdapterContract};
use abstract_core::IBC_HOST;
use abstract_core::{objects::module::ModuleInfo, AbstractError};
use abstract_sdk::{base::ModuleIbcEndpoint, features::AbstractRegistryAccess};
use cosmwasm_std::Addr;

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    ModuleIbcEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn ibc_host(&self, deps: cosmwasm_std::Deps) -> Result<Addr, Self::Error> {
        let vc_query_result = self
            .abstract_registry(deps)?
            .query_module(ModuleInfo::from_id_latest(IBC_HOST)?, &deps.querier)
            .map_err(Into::<AbstractError>::into)?;

        Ok(vc_query_result.reference.unwrap_native()?)
    }
}
