use crate::{state::ContractError, AdapterContract};
use abstract_sdk::{base::ModuleIbcEndpoint, features::AbstractRegistryAccess};
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    AbstractError, IBC_HOST,
};
use cosmwasm_std::Addr;

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg> ModuleIbcEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn ibc_host(
        &self,
        deps: cosmwasm_std::Deps,
        env: &cosmwasm_std::Env,
    ) -> Result<Addr, Self::Error> {
        let registry_query_result = self
            .abstract_registry(deps, env)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_HOST,
                    ModuleVersion::from(abstract_ibc_host::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<AbstractError>::into)?;

        Ok(registry_query_result.reference.unwrap_native()?)
    }
}
