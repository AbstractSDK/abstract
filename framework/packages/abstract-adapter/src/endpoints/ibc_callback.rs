use abstract_sdk::{base::IbcCallbackEndpoint, features::AbstractRegistryAccess, AbstractSdkError};
use abstract_std::{objects::module::ModuleInfo, AbstractError, IBC_CLIENT};
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
                ModuleInfo::from_id_latest(IBC_CLIENT).map_err(|err| {
                    let err: AbstractSdkError = err.into();
                    err
                })?,
                &deps.querier,
            )
            .map_err(|err| {
                let err: AbstractError = err.into();
                let err: AbstractSdkError = err.into();
                err
            })?;

        Ok(vc_query_result.reference.unwrap_native().map_err(|err| {
            let err: AbstractSdkError = err.into();
            err
        })?)
    }
}
