use abstract_sdk::{base::IbcCallbackEndpoint, features::AbstractRegistryAccess};
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    AbstractError, IBC_CLIENT,
};
use cosmwasm_std::{Addr, Deps};

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
    IbcCallbackEndpoint
    for AdapterContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    fn ibc_client(&self, deps: Deps) -> Result<Addr, Self::Error> {
        let vc_query_result = self
            .abstract_registry(deps)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_CLIENT,
                    ModuleVersion::from(abstract_ibc_client::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<AbstractError>::into)?;

        Ok(vc_query_result.reference.unwrap_native()?)
    }
}
