use abstract_sdk::{base::IbcCallbackEndpoint, features::AbstractRegistryAccess};
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    AbstractError, IBC_CLIENT,
};
use cosmwasm_std::{Addr, Deps};

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
    IbcCallbackEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, SudoMsg>
{
    fn ibc_client_addr(&self, deps: Deps) -> Result<Addr, Self::Error> {
        let registry_query_result = self
            .abstract_registry(deps)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_CLIENT,
                    ModuleVersion::from(abstract_ibc_client::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<AbstractError>::into)?;

        Ok(registry_query_result.reference.unwrap_native()?)
    }
}
