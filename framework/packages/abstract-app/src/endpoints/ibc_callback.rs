use abstract_sdk::features::AbstractRegistryAccess;
use abstract_std::{
    objects::module::{ModuleInfo, ModuleVersion},
    IBC_CLIENT,
};
use cosmwasm_std::Addr;

use crate::{state::ContractError, AppContract, IbcCallbackEndpoint};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > IbcCallbackEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    fn ibc_client_addr(&self, deps: cosmwasm_std::Deps) -> Result<Addr, Self::Error> {
        let registry_query_result = self
            .abstract_registry(deps)?
            .query_module(
                ModuleInfo::from_id(
                    IBC_CLIENT,
                    ModuleVersion::from(abstract_ibc_client::contract::CONTRACT_VERSION),
                )?,
                &deps.querier,
            )
            .map_err(Into::<abstract_std::AbstractError>::into)?;

        Ok(registry_query_result.reference.unwrap_native()?)
    }
}
